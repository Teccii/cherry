use std::{
    fmt,
    fmt::Write as _,
    io::Write,
    ops::*
};
use rand::Rng;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct T_f32(pub f32, pub f32);

impl T_f32 {
    #[inline(always)]
    pub fn scale(self, phase: f32) -> f32 {
        (self.0 * (TOTAL_PHASE as f32 - phase) + self.1 * phase)
    }
}

impl fmt::Display for T_f32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T({}, {}", self.0.round() as i16, self.1.round() as i16)
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_tapered_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<T_f32> for T_f32 {
            type Output = T_f32;

            #[inline(always)]
            fn $fn(self, rhs: T_f32) -> Self::Output {
                T_f32(self.0.$fn(rhs.0), self.1.$fn(rhs.1))
            }
        }
    )*};
}

macro_rules! impl_tapered_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<T_f32> for T_f32 {
            #[inline(always)]
            fn $fn(&mut self, rhs: T_f32) {
                self.0.$fn(rhs.0);
                self.1.$fn(rhs.1);
            }
        }
    )*};
}

macro_rules! impl_tapered_f32_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<f32> for T_f32 {
            type Output = T_f32;

            #[inline(always)]
            fn $fn(self, rhs: f32) -> Self::Output {
                T_f32(self.0.$fn(rhs), self.1.$fn(rhs))
            }
        }

        impl $trait<T_f32> for f32 {
            type Output = T_f32;

            #[inline(always)]
            fn $fn(self, rhs: T_f32) -> Self::Output {
                T_f32(self.$fn(rhs.0), self.$fn(rhs.1))
            }
        }
    )*};
}

macro_rules! impl_tapered_f32_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<f32> for T_f32 {
            #[inline(always)]
            fn $fn(&mut self, rhs: f32) {
                self.0.$fn(rhs);
                self.1.$fn(rhs);
            }
        }
    )*};
}

/*----------------------------------------------------------------*/

impl_tapered_ops! {
    Add, add;
    Sub, sub;
}

impl_tapered_assign_ops! {
    AddAssign, add_assign;
    SubAssign, sub_assign;
}

impl_tapered_f32_ops! {
    Mul, mul;
    Div, div;
}

impl_tapered_f32_assign_ops! {
    MulAssign, mul_assign;
    DivAssign, div_assign;
}

impl Mul<i16> for T_f32 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: i16) -> Self::Output {
        T_f32(self.0 * rhs as f32, self.1 * rhs as f32)
    }
}

impl<const MAX: usize, const SIZE: usize> Mul<IndicesPair<MAX, SIZE>> for T_f32 {
    type Output = IndexTable_f32<SIZE>;
    
    fn mul(self, indices: IndicesPair<MAX, SIZE>) -> Self::Output {
        let mut result = IndexTable_f32::default();
        
        for i in indices.white {
            result[i] += self;
        }
        
        for i in indices.black {
            result[i] -= self;
        }

        result
    }
}

impl Mul<FilePair> for T_f32 {
    type Output = FileTable_f32;

    fn mul(self, rhs: FilePair) -> Self::Output {
        let mut result = FileTable_f32::default();
        
        for sq in rhs.white {
            result[sq.file() as usize] += self;
        }
        
        for sq in rhs.black {
            result[sq.file() as usize] -= self;
        }
        
        result
    }
}

impl Mul<RankPair> for T_f32 {
    type Output = RankTable_f32;

    fn mul(self, rhs: RankPair) -> Self::Output {
        let mut result = RankTable_f32::default();

        for sq in rhs.white {
            result[sq.rank() as usize] += self;
        }

        for sq in rhs.black {
            result[sq.rank().flip() as usize] -= self;
        }

        result
    }
}

impl Mul<SquarePair> for T_f32 {
    type Output = SquareTable_f32;
    
    fn mul(self, rhs: SquarePair) -> Self::Output {
        let mut result = SquareTable_f32::default();
        
        for sq in rhs.white {
            result[sq as usize] += self;
        }
        
        for sq in rhs.black {
            result[sq.flip_rank() as usize] -= self;
        }
        
        result
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct IndexTable_f32<const COUNT: usize>(pub [T_f32; COUNT]);

impl<const COUNT: usize> IndexTable_f32<COUNT> {
    #[inline(always)]
    pub const fn new(table: [T_f32; COUNT]) -> Self {
        Self(table)
    }
}

impl<const COUNT: usize> Default for IndexTable_f32<COUNT> {
    fn default() -> Self {
        Self([T_f32::default(); COUNT])
    }
}

impl<const COUNT: usize> fmt::Display for IndexTable_f32<COUNT> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for i in self.0 {
            write!(f, " {},", i)?;
        }
        write!(f, "]")
    }
}

impl<const COUNT: usize> Index<usize> for IndexTable_f32<COUNT> {
    type Output = T_f32;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const COUNT: usize> IndexMut<usize> for IndexTable_f32<COUNT> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_table_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait for IndexTable_f32<COUNT> {
            type Output = Self;

            fn $fn(self, rhs: Self) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < COUNT {
                    result.0[i] = T_f32(self.0[i].0.$fn(rhs.0[i].0), self.0[i].1.$fn(rhs.0[i].1));
                    i += 1;
                }

                result
            }
        }
    )*};
}

macro_rules! impl_table_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait for IndexTable_f32<COUNT> {
            fn $fn(&mut self, rhs: Self) {
                let mut i = 0;

                while i < COUNT {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs.0[i].0), self.0[i].1.$op(rhs.0[i].1));
                    i += 1;
                }
            }
        }
    )*};
}

macro_rules! impl_table_tapered_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait<T_f32> for IndexTable_f32<COUNT> {
            type Output = Self;

            fn $fn(self, rhs: T_f32) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < COUNT {
                    result[i] = T_f32(self.0[i].0.$fn(rhs.0), self.0[i].1.$fn(rhs.1));
                    i += 1;
                }

                result
            }
        }
    )*};
}

macro_rules! impl_table_tapered_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait<T_f32> for IndexTable_f32<COUNT> {
            fn $fn(&mut self, rhs: T_f32) {
                let mut i = 0;

                while i < COUNT {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs.0), self.0[i].1.$op(rhs.1));
                    i += 1;
                }
            }
        }
    )*};
}


macro_rules! impl_table_f32_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait<f32> for IndexTable_f32<COUNT> {
            type Output = Self;

            fn $fn(self, rhs: f32) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < COUNT {
                    result.0[i] = T_f32(self.0[i].0.$fn(rhs), self.0[i].1.$fn(rhs));
                    i += 1;
                }

                result
            }
        }
    )*};
}

macro_rules! impl_table_f32_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait<f32> for IndexTable_f32<COUNT> {
            fn $fn(&mut self, rhs: f32) {
                let mut i = 0;

                while i < COUNT {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs), self.0[i].1.$op(rhs));
                    i += 1;
                }
            }
        }
    )*};
}

/*----------------------------------------------------------------*/

impl_table_ops! {
    Add, add;
    Sub, sub;
}

impl_table_assign_ops! {
    AddAssign, add_assign, add;
    SubAssign, sub_assign, sub;
}

impl_table_tapered_ops! {
    Add, add;
    Sub, sub;
    Mul, mul;
    Div, div;
}

impl_table_tapered_assign_ops! {
    AddAssign, add_assign, add;
    SubAssign, sub_assign, sub;
}

impl_table_f32_ops! {
    Mul, mul;
    Div, div;
}

impl_table_f32_assign_ops! {
    MulAssign, mul_assign, mul;
    DivAssign, div_assign, div;
}

impl<const MAX: usize, const SIZE: usize> Mul<IndicesPair<MAX, SIZE>> for IndexTable_f32<SIZE> {
    type Output = T_f32;

    fn mul(self, rhs: IndicesPair<MAX, SIZE>) -> Self::Output {
        let mut score = T_f32(0.0, 0.0);

        for i in rhs.white {
            score += self[i];
        }

        for i in rhs.black {
            score -= self[i];
        }

        score
    }
}

/*----------------------------------------------------------------*/

macro_rules! def_special_tables {
    ($($name:ident, $count:expr;)*) => {$(
        #[derive(Debug, Copy, Clone)]
        pub struct $name(pub [T_f32; $count]);

        impl $name {
            #[inline(always)]
            pub const fn new(table: [T_f32; $count]) -> Self {
                Self(table)
            }
        }
        
        impl Default for $name {
            fn default() -> Self {
                Self([T_f32::default(); $count])
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "[")?;

                for i in self.0 {
                    write!(f, " {},", i)?;
                }

                write!(f, "]")
            }
        }

        impl Index<usize> for $name {
            type Output = T_f32;

            #[inline(always)]
            fn index(&self, index: usize) -> &Self::Output {
                &self.0[index]
            }
        }

        impl IndexMut<usize> for $name {
            #[inline(always)]
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.0[index]
            }
        }
    )*}
}

macro_rules! impl_special_table_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for FileTable_f32 {
            type Output = Self;

            fn $fn(self, rhs: Self) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < File::NUM {
                    result.0[i] = T_f32(self.0[i].0.$fn(rhs.0[i].0), self.0[i].1.$fn(rhs.0[i].1));
                    i += 1;
                }

                result
            }
        }
    
        impl $trait for RankTable_f32 {
            type Output = Self;

            fn $fn(self, rhs: Self) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < Rank::NUM {
                    result.0[i] = T_f32(self.0[i].0.$fn(rhs.0[i].0), self.0[i].1.$fn(rhs.0[i].1));
                    i += 1;
                }

                result
            }
        }
    
        impl $trait for SquareTable_f32 {
            type Output = Self;

            fn $fn(self, rhs: Self) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < Square::NUM {
                    result.0[i] = T_f32(self.0[i].0.$fn(rhs.0[i].0), self.0[i].1.$fn(rhs.0[i].1));
                    i += 1;
                }

                result
            }
        }
    )*};
}

macro_rules! impl_special_table_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl $trait for FileTable_f32 {
            fn $fn(&mut self, rhs: Self) {
                let mut i = 0;

                while i < File::NUM {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs.0[i].0), self.0[i].1.$op(rhs.0[i].1));
                    i += 1;
                }
            }
        }

        impl $trait for RankTable_f32 {
            fn $fn(&mut self, rhs: Self) {
                let mut i = 0;

                while i < Rank::NUM {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs.0[i].0), self.0[i].1.$op(rhs.0[i].1));
                    i += 1;
                }
            }
        }

        impl $trait for SquareTable_f32 {
            fn $fn(&mut self, rhs: Self) {
                let mut i = 0;

                while i < File::NUM {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs.0[i].0), self.0[i].1.$op(rhs.0[i].1));
                    i += 1;
                }
            }
        }
    )*};
}

macro_rules! impl_special_table_tapered_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<T_f32> for FileTable_f32 {
            type Output = Self;

            fn $fn(self, rhs: T_f32) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < File::NUM {
                    result[i] = T_f32(self.0[i].0.$fn(rhs.0), self.0[i].1.$fn(rhs.1));
                    i += 1;
                }

                result
            }
        }

        impl $trait<T_f32> for RankTable_f32 {
            type Output = Self;

            fn $fn(self, rhs: T_f32) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < Rank::NUM {
                    result[i] = T_f32(self.0[i].0.$fn(rhs.0), self.0[i].1.$fn(rhs.1));
                    i += 1;
                }

                result
            }
        }

        impl $trait<T_f32> for SquareTable_f32 {
            type Output = Self;

            fn $fn(self, rhs: T_f32) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < Square::NUM {
                    result[i] = T_f32(self.0[i].0.$fn(rhs.0), self.0[i].1.$fn(rhs.1));
                    i += 1;
                }

                result
            }
        }
    )*};
}

macro_rules! impl_special_table_tapered_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl $trait<T_f32> for FileTable_f32 {
            fn $fn(&mut self, rhs: T_f32) {
                let mut i = 0;

                while i < File::NUM {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs.0), self.0[i].1.$op(rhs.1));
                    i += 1;
                }
            }
        }

        impl $trait<T_f32> for RankTable_f32 {
            fn $fn(&mut self, rhs: T_f32) {
                let mut i = 0;

                while i < Rank::NUM {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs.0), self.0[i].1.$op(rhs.1));
                    i += 1;
                }
            }
        }

        impl $trait<T_f32> for SquareTable_f32 {
            fn $fn(&mut self, rhs: T_f32) {
                let mut i = 0;

                while i < Square::NUM {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs.0), self.0[i].1.$op(rhs.1));
                    i += 1;
                }
            }
        }
    )*};
}


macro_rules! impl_special_table_f32_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<f32> for FileTable_f32 {
            type Output = Self;

            fn $fn(self, rhs: f32) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < File::NUM {
                    result.0[i] = T_f32(self.0[i].0.$fn(rhs), self.0[i].1.$fn(rhs));
                    i += 1;
                }

                result
            }
        }

        impl $trait<f32> for RankTable_f32 {
            type Output = Self;

            fn $fn(self, rhs: f32) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < Rank::NUM {
                    result.0[i] = T_f32(self.0[i].0.$fn(rhs), self.0[i].1.$fn(rhs));
                    i += 1;
                }

                result
            }
        }

        impl $trait<f32> for SquareTable_f32 {
            type Output = Self;

            fn $fn(self, rhs: f32) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < Square::NUM {
                    result.0[i] = T_f32(self.0[i].0.$fn(rhs), self.0[i].1.$fn(rhs));
                    i += 1;
                }

                result
            }
        }
    )*};
}

macro_rules! impl_special_table_f32_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl $trait<f32> for FileTable_f32 {
            fn $fn(&mut self, rhs: f32) {
                let mut i = 0;

                while i < File::NUM {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs), self.0[i].1.$op(rhs));
                    i += 1;
                }
            }
        }

        impl $trait<f32> for RankTable_f32 {
            fn $fn(&mut self, rhs: f32) {
                let mut i = 0;

                while i < Rank::NUM {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs), self.0[i].1.$op(rhs));
                    i += 1;
                }
            }
        }

        impl $trait<f32> for SquareTable_f32 {
            fn $fn(&mut self, rhs: f32) {
                let mut i = 0;

                while i < Square::NUM {
                    self.0[i] = T_f32(self.0[i].0.$op(rhs), self.0[i].1.$op(rhs));
                    i += 1;
                }
            }
        }
    )*};
}

/*----------------------------------------------------------------*/

def_special_tables! {
    FileTable_f32, File::NUM;
    RankTable_f32, Rank::NUM;
    SquareTable_f32, Square::NUM;
}

impl_special_table_ops! {
    Add, add;
    Sub, sub;
}

impl_special_table_assign_ops! {
    AddAssign, add_assign, add;
    SubAssign, sub_assign, sub;
}

impl_special_table_tapered_ops! {
    Add, add;
    Sub, sub;
    Mul, mul;
    Div, div;
}

impl_special_table_tapered_assign_ops! {
    AddAssign, add_assign, add;
    SubAssign, sub_assign, sub;
}

impl_special_table_f32_ops! {
    Mul, mul;
    Div, div;
}

impl_special_table_f32_assign_ops! {
    MulAssign, mul_assign, mul;
    DivAssign, div_assign, div;
}

impl Mul<FilePair> for FileTable_f32 {
    type Output = T_f32;

    fn mul(self, rhs: FilePair) -> Self::Output {
        let mut score = T_f32(0.0, 0.0);

        for sq in rhs.white {
            score += self.0[sq.file() as usize];
        }

        for sq in rhs.black {
            score += self.0[sq.file() as usize];
        }

        score
    }
}

impl Mul<RankPair> for RankTable_f32 {
    type Output = T_f32;

    fn mul(self, rhs: RankPair) -> Self::Output {
        let mut score = T_f32(0.0, 0.0);

        for sq in rhs.white {
            score += self.0[sq.rank() as usize];
        }

        for sq in rhs.black {
            score += self.0[sq.rank().flip() as usize];
        }

        score
    }
}

impl Mul<SquarePair> for SquareTable_f32 {
    type Output = T_f32;

    fn mul(self, rhs: SquarePair) -> Self::Output {
        let mut score = T_f32(0.0, 0.0);

        for sq in rhs.white {
            score += self.0[sq as usize];
        }

        for sq in rhs.black {
            score += self.0[sq.flip_rank() as usize];
        }

        score
    }
}

/*----------------------------------------------------------------*/

macro_rules! apply_weights {
    ($weights:expr, $trace:expr, $elem:ident) => {
        $weights.$elem.clone() * $trace.$elem.clone()
    };
    ($weights:expr, $trace:expr, $elem:ident, $($elems:ident),*) => {
        $weights.$elem.clone() * $trace.$elem.clone() + apply_weights!($weights, $trace, $($elems),*)
    }
}

macro_rules! write_weights {
    ($fmt:expr, $weights:expr, $elem:ident) => {
        write!($fmt, "{}: {}\n", stringify!($elem), $weights.$elem)?;
    };
    ($fmt:expr, $weights:expr, $elem:ident, $($elems:ident),*) => {
        write!($fmt, "{}: {}\n", stringify!($elem), $weights.$elem)?;
        write_weights!($fmt, $weights, $($elems),*);
    }
}

/*----------------------------------------------------------------*/

fn sigmoid(x: f32, k: f32) -> f32 {
    1.0 / (1.0 + (k * -x).exp())
}

macro_rules! tuner {
    ($($elem:ident: $ty_f32:ty,)*) => {
        #[derive(Debug, Copy, Clone, Default)]
        pub struct Weights {
            $(pub $elem: $ty_f32),*
        }

        impl Weights {
            #[inline(always)]
            pub fn new($($elem: $ty_f32),*) -> Weights {
                Weights { $($elem),* }
            }

            #[inline(always)]
            pub fn apply(&self, trace: &EvalTrace) -> f32 {
                let score: T_f32 = apply_weights!(self, trace, $($elem),*);
                score.scale(trace.phase as f32)
            }
        }

        impl fmt::Display for Weights {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write_weights!(f, self, $($elem),*);
                Ok(())
            }
        }
        
        macro_rules! impl_weight_op {
            ($trait:ident, $fn:ident) => {
                impl $trait for Weights {
                    type Output = Self;
                    
                    fn $fn(self, rhs: Self) -> Self {
                        Weights { $($elem: self.$elem.$fn(rhs.$elem)),* }
                    }
                }
            }
        }
        
        macro_rules! impl_weight_assign_op {
            ($trait:ident, $fn:ident) => {
                impl $trait for Weights {
                    fn $fn(&mut self, rhs: Self) {
                        $(self.$elem.$fn(rhs.$elem);)*
                    }
                }
            }
        }
        
        macro_rules! impl_weight_f32_op {
            ($trait:ident, $fn:ident) => {
                impl $trait<f32> for Weights {
                    type Output = Self;
                    
                    fn $fn(self, rhs: f32) -> Self {
                        Weights { $($elem: self.$elem.$fn(rhs)),* }
                    }
                }
            }
        }
        
        impl_weight_op!(Add, add);
        impl_weight_op!(Sub, sub);
        impl_weight_assign_op!(AddAssign, add_assign);
        impl_weight_assign_op!(SubAssign, sub_assign);
        impl_weight_f32_op!(Mul, mul);

        #[derive(Debug, Clone)]
        pub struct Tuner {
            weights: Weights,
            gradient: Weights,
            learning_rate: f32,
            regression_factor: f32,
            decay_factor: f32,
        }
        
        impl Tuner {
            pub fn new(learning_rate: f32, regression_factor: f32, decay_factor: f32) -> Tuner {
                Tuner {
                    weights: Weights::default(),
                    gradient: Weights::default(),
                    learning_rate,
                    regression_factor,
                    decay_factor
                }
            }
            
            pub fn error(&self, training_data: &[TrainingData]) -> f32 {
                let mut error = 0.0;
                let mut sum = 0.0;
                
                for data in training_data {
                    let pred = sigmoid(self.feed_forward(&data.trace), self.regression_factor);
                    let diff = data.result - pred;
                    
                    error += data.weight * diff * diff;
                    sum += data.weight;
                }
                
                error / sum
            }
            
            pub fn back_prop(&mut self, data: &TrainingData) {
                let pred = sigmoid(self.feed_forward(&data.trace), self.regression_factor);
                let grad = pred - data.result;
                let sig_grad = data.weight * grad * pred * (1.0 - pred) * self.learning_rate * self.regression_factor;
                
                let mg = data.trace.phase as f32 / TOTAL_PHASE as f32;
                let eg = 1.0 - mg;
                
                let tune = T_f32(mg * sig_grad, eg * sig_grad);
                let delta = Weights::new($(tune * data.trace.$elem.clone()),*);
                self.gradient = self.gradient * self.decay_factor - delta * self.learning_rate;
                self.weights += self.gradient;
            }
            
            pub fn feed_forward(&self, trace: &EvalTrace) -> f32 {
                self.weights.apply(trace)
            }
        }

        impl fmt::Display for Tuner {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.weights)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrainingData {
    pub trace: EvalTrace,
    pub result: f32,
    pub weight: f32,
}

tuner! {
    bishop_pair: T_f32,
    knight_mobility: IndexTable_f32<9>,
    bishop_mobility: IndexTable_f32<14>,
    rook_mobility: IndexTable_f32<15>,
    queen_mobility: IndexTable_f32<28>,

    passed_pawn: RankTable_f32,
    backwards_pawn: T_f32,
    isolated_pawn: T_f32,
    doubled_pawn: T_f32,
    phalanx: T_f32,
    support: T_f32,

    pawn_minor_threat: T_f32,
    pawn_major_threat: T_f32,
    minor_major_threat: T_f32,

    space_restrict_piece: T_f32,
    space_restrict_empty: T_f32,
    space_center_control: T_f32,

    knight_attack: T_f32,
    bishop_attack: T_f32,
    rook_attack: T_f32,
    queen_attack: T_f32,

    rook_open_file: FileTable_f32,
    rook_semiopen_file: FileTable_f32,
    queen_open_file: FileTable_f32,
    queen_semiopen_file: FileTable_f32,

    pawn_value: T_f32,
    knight_value: T_f32,
    bishop_value: T_f32,
    rook_value: T_f32,
    queen_value: T_f32,

    pawn_psqt: SquareTable_f32,
    knight_psqt: SquareTable_f32,
    bishop_psqt: SquareTable_f32,
    rook_psqt: SquareTable_f32,
    queen_psqt: SquareTable_f32,
    king_psqt: SquareTable_f32,
}

pub fn tune(out_path: &str, training_data: &[TrainingData], iters: u64, batch_size: u64) {
    let mut tuner = Box::new(Tuner::new(0.005, 1.0, 0.3));

    for i in 0..(iters / batch_size) {
        let mut rng = rand::rng();

        for _ in 0..batch_size {
            let i = rng.random_range(0..training_data.len());
            let data = &training_data[i];
            tuner.back_prop(data);
        }

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(out_path)
            .unwrap();

        let mut out = String::new();
        
        writeln!(&mut out, "--------------------------------").unwrap();
        writeln!(&mut out, "{}", tuner).unwrap();
        writeln!(&mut out, "Error: {}", tuner.error(training_data)).unwrap();
        writeln!(&mut out, "Iteration: {}", i * batch_size).unwrap();

        print!("{}", out);
        write!(&mut file, "{}", out).unwrap();
    }
}