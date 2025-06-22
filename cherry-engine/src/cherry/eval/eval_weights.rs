use std::{fmt, ops::*};
use arrayvec::ArrayVec;
use cozy_chess::*;
use crate::*;

/*----------------------------------------------------------------*/

#[macro_export]
macro_rules! table {
    ($(($mg:expr, $eg:expr),)*) => {
        IndexTable::new([$(T($mg, $eg),)*])
    }
}

/*----------------------------------------------------------------*/

macro_rules! def_tapered {
    ($name:ident, $ty:ty) => {
        #[derive(Debug, Copy, Clone, PartialEq, Default)]
        pub struct $name(pub $ty, pub $ty);

        impl $name {
            #[inline(always)]
            pub fn new_mg(score: $ty) -> $name {
                $name(score, 0 as $ty)
            }

            #[inline(always)]
            pub fn new_eg(score: $ty) -> $name {
                $name(0 as $ty, score)
            }

            #[inline(always)]
            pub const fn mg(self) -> $name {
                $name(self.0, 0 as $ty)
            }

            #[inline(always)]
            pub const fn eg(self) -> $name {
                $name(0 as $ty, self.1)
            }

            pub const ZERO: $name = $name(0 as $ty, 0 as $ty);
        }

        /*----------------------------------------------------------------*/
        
        macro_rules! impl_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<$name> for $name {
                    type Output = $name;
    
                    #[inline(always)]
                    fn $fn(self, rhs: $name) -> Self::Output {
                        $name(self.0.$fn(rhs.0), self.1.$fn(rhs.1))
                    }
                }
            }
        }
        
        macro_rules! impl_assign_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<$name> for $name {
                    #[inline(always)]
                    fn $fn(&mut self, rhs: $name) {
                        self.0.$fn(rhs.0);
                        self.1.$fn(rhs.1);
                    }
                }
            }
        }
        
        macro_rules! impl_type_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<$ty> for $name {
                    type Output = $name;
    
                    #[inline(always)]
                    fn $fn(self, rhs: $ty) -> Self::Output {
                        $name(self.0.$fn(rhs), self.1.$fn(rhs))
                    }
                }

                impl $trait<$name> for $ty {
                    type Output = $name;

                    #[inline(always)]
                    fn $fn(self, rhs: $name) -> Self::Output {
                        $name(self.$fn(rhs.0), self.$fn(rhs.1))
                    }
                }
            }
        }
        
        macro_rules! impl_type_assign_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<$ty> for $name {
                    #[inline(always)]
                    fn $fn(&mut self, rhs: $ty) {
                        self.0.$fn(rhs);
                        self.1.$fn(rhs);
                    }
                }
            }
        }

        /*----------------------------------------------------------------*/
        
        impl_ops!(Add, add);
        impl_ops!(Sub, sub);
        impl_ops!(Mul, mul);
        impl_ops!(Div, div);
        
        impl_assign_ops!(AddAssign, add_assign);
        impl_assign_ops!(SubAssign, sub_assign);
        impl_assign_ops!(MulAssign, mul_assign);
        impl_assign_ops!(DivAssign, div_assign);

        impl_type_ops!(Add, add);
        impl_type_ops!(Sub, sub);
        impl_type_ops!(Mul, mul);
        impl_type_ops!(Div, div);

        impl_type_assign_ops!(AddAssign, add_assign);
        impl_type_assign_ops!(SubAssign, sub_assign);
        impl_type_assign_ops!(MulAssign, mul_assign);
        impl_type_assign_ops!(DivAssign, div_assign);
    };
}

/*----------------------------------------------------------------*/

def_tapered!(T, i16);
def_tapered!(T_f32, f32);

/*----------------------------------------------------------------*/

impl T {
    #[inline(always)]
    pub fn scale(self, phase: u16) -> Score {
        let phase = (phase * TAPER_SCALE + TOTAL_PHASE / 2) / TOTAL_PHASE;
        let score = (self.0 as i32 * (TAPER_SCALE - phase) as i32 + self.1 as i32 * phase as i32) / TAPER_SCALE as i32;

        Score(score as i16)
    }
}

impl Eq for T { }

impl fmt::Display for T {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T({}, {})", self.0, self.1)
    }
}

/*----------------------------------------------------------------*/

impl T_f32 {
    #[inline(always)]
    pub fn scale(self, phase: u16) -> f32 {
        (self.0 * (TOTAL_PHASE as f32 - phase as f32) + self.1 * phase as f32) / TOTAL_PHASE as f32
    }

    pub fn sqrt(&self) -> T_f32 {
        T_f32(self.0.sqrt(), self.1.sqrt())
    }
}

impl From<T> for T_f32 {
    fn from(value: T) -> Self {
        T_f32(value.0 as f32, value.1 as f32)
    }
}

impl fmt::Display for T_f32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T({}, {})", self.0.round() as i16, self.1.round() as i16)
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_tapered_f32_i16_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<i16> for T_f32 {
            type Output = Self;

            fn $fn(self, rhs: i16) -> Self::Output {
                T_f32(self.0.$fn(rhs as f32), self.1.$fn(rhs as f32))
            }
        }
    )*}
}

macro_rules! impl_tapered_f32_i16_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<i16> for T_f32 {
            fn $fn(&mut self, rhs: i16) {
                self.0.$fn(rhs as f32);
                self.1.$fn(rhs as f32);
            }
        }
    )*}
}

/*----------------------------------------------------------------*/

impl_tapered_f32_i16_ops! {
    Mul, mul;
    Div, div;
}

impl_tapered_f32_i16_assign_ops! {
    MulAssign, mul_assign;
    DivAssign, div_assign;
}

/*----------------------------------------------------------------*/

macro_rules! def_table {
    ($name:ident, $tapered_ty:ident, $ty:ty) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct $name<const COUNT: usize>(pub [$tapered_ty; COUNT]);
        
        impl<const COUNT: usize> $name<COUNT> {
            #[inline(always)]
            pub const fn new(table: [$tapered_ty; COUNT]) -> Self {
                Self(table)
            }
        }

        impl<const COUNT: usize> Default for $name<COUNT> {
            fn default() -> Self {
                Self::new([$tapered_ty::default(); COUNT])
            }
        }
        
        impl<const COUNT: usize> fmt::Display for $name<COUNT> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "[")?;
                
                for i in 0..COUNT {
                    write!(f, "{}, ", self[i])?;
                }
                
                write!(f, "]")
            }
        }

        /*----------------------------------------------------------------*/
        
        impl<const COUNT: usize> Index<usize> for $name<COUNT> {
            type Output = $tapered_ty;

            #[inline(always)]
            fn index(&self, index: usize) -> &Self::Output {
                &self.0[index]
            }
        }

        impl<const COUNT: usize> IndexMut<usize> for $name<COUNT> {
            #[inline(always)]
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.0[index]
            }
        }

        /*----------------------------------------------------------------*/
        
        macro_rules! impl_ops {
            ($trait:ident, $fn:ident) => {
                impl<const COUNT: usize> $trait for $name<COUNT> {
                    type Output = Self;

                    fn $fn(self, rhs: Self) -> Self::Output {
                        let mut result = self;

                        for i in 0..COUNT {
                            result[i] = $tapered_ty(self[i].0.$fn(rhs[i].0), self[i].1.$fn(rhs[i].1));
                        }

                        result
                    }
                }
            };
        }
        
        macro_rules! impl_assign_ops {
            ($trait:ident, $fn:ident, $op:ident) => {
                impl<const COUNT: usize> $trait for $name<COUNT> {
                    fn $fn(&mut self, rhs: Self) {
                        for i in 0..COUNT {
                            self[i] = $tapered_ty(self[i].0.$op(rhs[i].0), self[i].1.$op(rhs[i].1));
                        }
                    }
                }
            };
        }
        
        macro_rules! impl_tapered_ops {
            ($trait:ident, $fn:ident) => {
                impl<const COUNT: usize> $trait<$tapered_ty> for $name<COUNT> {
                    type Output = Self;
        
                    fn $fn(self, rhs: $tapered_ty) -> Self::Output {
                        let mut result = self;
        
                        for i in 0..COUNT {
                            result[i] = $tapered_ty(self[i].0.$fn(rhs.0), self[i].1.$fn(rhs.1));
                        }
        
                        result
                    }
                }

                impl<const COUNT: usize> $trait<$name<COUNT>> for $tapered_ty {
                    type Output = $name<COUNT>;

                    fn $fn(self, rhs: $name<COUNT>) -> Self::Output {
                        let mut result = rhs;

                        for i in 0..COUNT {
                            result[i] = $tapered_ty(self.0.$fn(rhs[i].0), self.1.$fn(rhs[i].1));
                        }

                        result
                    }
                }
            };
        }
        
        macro_rules! impl_tapered_assign_ops {
            ($trait:ident, $fn:ident, $op:ident) => {
                impl<const COUNT: usize> $trait<$tapered_ty> for $name<COUNT> {
                    fn $fn(&mut self, rhs: $tapered_ty) {
                        for i in 0..COUNT {
                            self[i] = $tapered_ty(self[i].0.$op(rhs.0), self[i].1.$op(rhs.1));
                        }
                    }
                }
            };
        }
        
        macro_rules! impl_type_ops {
            ($trait:ident, $fn:ident) => {
                impl<const COUNT: usize> $trait<$ty> for $name<COUNT> {
                    type Output = Self;
        
                    fn $fn(self, rhs: $ty) -> Self::Output {
                        let mut result = self;
        
                        for i in 0..COUNT {
                            result[i] = $tapered_ty(self[i].0.$fn(rhs), self[i].1.$fn(rhs));
                        }
        
                        result
                    }
                }
            };
        }
        
        macro_rules! impl_type_assign_ops {
            ($trait:ident, $fn:ident, $op:ident) => {
                impl<const COUNT: usize> $trait<$ty> for $name<COUNT> {
                    fn $fn(&mut self, rhs: $ty) {
                        for i in 0..COUNT {
                            self[i] = $tapered_ty(self[i].0.$op(rhs), self[i].1.$op(rhs));
                        }
                    }
                }
            };
        }

        /*----------------------------------------------------------------*/
        
        impl_ops!(Add, add);
        impl_ops!(Sub, sub);
        impl_ops!(Mul, mul);
        impl_ops!(Div, div);
        
        impl_assign_ops!(AddAssign, add_assign, add);
        impl_assign_ops!(SubAssign, sub_assign, sub);
        impl_assign_ops!(MulAssign, mul_assign, mul);
        impl_assign_ops!(DivAssign, div_assign, div);
        
        impl_tapered_ops!(Add, add);
        impl_tapered_ops!(Sub, sub);
        impl_tapered_ops!(Mul, mul);
        impl_tapered_ops!(Div, div);
        
        impl_tapered_assign_ops!(AddAssign, add_assign, add);
        impl_tapered_assign_ops!(SubAssign, sub_assign, sub);
        impl_tapered_assign_ops!(MulAssign, mul_assign, mul);
        impl_tapered_assign_ops!(DivAssign, div_assign, div);

        impl_type_ops!(Add, add);
        impl_type_ops!(Sub, sub);
        impl_type_ops!(Mul, mul);
        impl_type_ops!(Div, div);

        impl_type_assign_ops!(AddAssign, add_assign, add);
        impl_type_assign_ops!(SubAssign, sub_assign, sub);
        impl_type_assign_ops!(MulAssign, mul_assign, mul);
        impl_type_assign_ops!(DivAssign, div_assign, div);
    }
}

/*----------------------------------------------------------------*/

def_table!(IndexTable, T, i16);
def_table!(IndexTable_f32, T_f32, f32);

pub type FileTable = IndexTable<{File::NUM}>;
pub type RankTable = IndexTable<{Rank::NUM}>;
pub type SquareTable = IndexTable<{Square::NUM}>;

pub type FileTable_f32 = IndexTable_f32<{File::NUM}>;
pub type RankTable_f32 = IndexTable_f32<{Rank::NUM}>;
pub type SquareTable_f32 = IndexTable_f32<{Square::NUM}>;

/*----------------------------------------------------------------*/

impl<const COUNT: usize> IndexTable_f32<COUNT> {
    pub fn sqrt(&self) -> Self {
        let mut result = Self::default();

        for i in 0..COUNT {
            result[i] = result[i].sqrt();
        }

        result
    }
}

impl<const COUNT: usize> From<IndexTable<COUNT>> for IndexTable_f32<COUNT> {
    fn from(table: IndexTable<COUNT>) -> Self {
        let mut result = IndexTable_f32::default();

        for i in 0..COUNT {
            result[i] = T_f32::from(table[i]);
        }

        result
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_table_f32_i16_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait<i16> for IndexTable_f32<COUNT> {
            type Output = Self;

            fn $fn(self, rhs: i16) -> Self::Output {
                let mut result = self;

                for i in 0..COUNT {
                    result[i] = T_f32(self[i].0.$fn(rhs as f32), self[i].1.$fn(rhs as f32));
                }

                result
            }
        }
    )*};
}

macro_rules! impl_table_f32_i16_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait<i16> for IndexTable_f32<COUNT> {
            fn $fn(&mut self, rhs: i16) {
                for i in 0..COUNT {
                    self[i] = T_f32(self[i].0.$op(rhs as f32), self[i].1.$op(rhs as f32));
                }
            }
        }
    )*};
}

/*----------------------------------------------------------------*/

impl_table_f32_i16_ops! {
    Mul, mul;
    Div, div;
}

impl_table_f32_i16_assign_ops! {
    MulAssign, mul_assign, mul;
    DivAssign, div_assign, div;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct FilePair {
    pub white: BitBoard,
    pub black: BitBoard,
}

impl fmt::Display for FilePair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}, {:?}]", self.white, self.black)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct RankPair {
    pub white: BitBoard,
    pub black: BitBoard,
}

impl fmt::Display for RankPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}, {:?}]", self.white, self.black)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct SquarePair {
    pub white: BitBoard,
    pub black: BitBoard,
}

impl fmt::Display for SquarePair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}, {:?}]", self.white, self.black)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct IndicesPair<const MAX: usize, const SIZE: usize> {
    pub white: ArrayVec<usize, MAX>,
    pub black: ArrayVec<usize, MAX>
}

impl<const MAX: usize, const SIZE: usize> fmt::Display for IndicesPair<MAX, SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}, {:?}]", self.white, self.black)
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_table_pair_ops {
    ($table:ident, $file_table:ident, $rank_table:ident, $square_table:ident, $tapered_ty:ident) => {
        impl<const MAX: usize, const COUNT: usize> Mul<IndicesPair<MAX, COUNT>> for $table<COUNT> {
            type Output = $tapered_ty;
            
            fn mul(self, rhs: IndicesPair<MAX, COUNT>) -> Self::Output {
                let mut result = $tapered_ty::ZERO;
                    
                for i in rhs.white {
                    result += self[i];
                }
                    
                for i in rhs.black {
                    result -= self[i];
                }
                    
                result
            }
        }
        
        impl Mul<FilePair> for $file_table {
            type Output = $tapered_ty;
            
            fn mul(self, rhs: FilePair) -> Self::Output {
                let mut result = $tapered_ty::ZERO;
                    
                for sq in rhs.white {
                    result += self[sq.file() as usize];
                }
                    
                for sq in rhs.black {
                    result -= self[sq.file() as usize];
                }
                    
                result
            }
        }
        
        impl Mul<RankPair> for $rank_table {
            type Output = $tapered_ty;
            
            fn mul(self, rhs: RankPair) -> Self::Output {
                let mut result = $tapered_ty::ZERO;
                    
                for sq in rhs.white {
                    result += self[sq.rank() as usize];
                }
                    
                for sq in rhs.black {
                    result -= self[sq.rank().flip() as usize];
                }
                    
                result
            }
        }
        
        impl Mul<SquarePair> for $square_table {
            type Output = $tapered_ty;
            
            fn mul(self, rhs: SquarePair) -> Self::Output {
                let mut result = $tapered_ty::ZERO;
                    
                for sq in rhs.white {
                    result += self[sq as usize];
                }
                    
                for sq in rhs.black {
                    result -= self[sq.flip_rank() as usize];
                }
                    
                result
            }
        }

        /*----------------------------------------------------------------*/

        impl Mul<FilePair> for $tapered_ty {
            type Output = $file_table;

            fn mul(self, rhs: FilePair) -> Self::Output {
                let mut result = $file_table::default();

                for sq in rhs.white {
                    result[sq.file() as usize] += self;
                }

                for sq in rhs.black {
                    result[sq.file() as usize] -= self;
                }

                result
            }
        }

        impl Mul<RankPair> for $tapered_ty {
            type Output = $rank_table;

            fn mul(self, rhs: RankPair) -> Self::Output {
                let mut result = $rank_table::default();

                for sq in rhs.white {
                    result[sq.rank() as usize] += self;
                }

                for sq in rhs.black {
                    result[sq.rank().flip() as usize] -= self;
                }

                result
            }
        }

        impl Mul<SquarePair> for $tapered_ty {
            type Output = $square_table;

            fn mul(self, rhs: SquarePair) -> Self::Output {
                let mut result = $square_table::default();

                for sq in rhs.white {
                    result[sq as usize] += self;
                }

                for sq in rhs.black {
                    result[sq.flip_rank() as usize] -= self;
                }

                result
            }
        }

        impl<const MAX: usize, const COUNT: usize> Mul<IndicesPair<MAX, COUNT>> for $tapered_ty {
            type Output = $table<COUNT>;

            fn mul(self, rhs: IndicesPair<MAX, COUNT>) -> Self::Output {
                let mut result = $table::default();

                for i in rhs.white {
                    result[i] += self;
                }

                for i in rhs.black {
                    result[i] -= self;
                }

                result
            }
        }
    }
}

impl_table_pair_ops!(IndexTable, FileTable, RankTable, SquareTable, T);
impl_table_pair_ops!(IndexTable_f32, FileTable_f32, RankTable_f32, SquareTable_f32, T_f32);

/*----------------------------------------------------------------*/

pub const PAWN_PHASE: u16 = 0;
pub const KNIGHT_PHASE: u16 = 1;
pub const BISHOP_PHASE: u16 = 1;
pub const ROOK_PHASE: u16 = 2;
pub const QUEEN_PHASE: u16 = 4;

pub const TOTAL_PHASE: u16 = 16 * PAWN_PHASE
    + 4 * KNIGHT_PHASE
    + 4 * BISHOP_PHASE
    + 4 * ROOK_PHASE
    + 2 * QUEEN_PHASE;

pub const TAPER_SCALE: u16 = 256;

/*----------------------------------------------------------------*/

macro_rules! apply_weights {
    ($weights:expr, $trace:expr, $elem:ident) => {
        $weights.$elem.clone() * $trace.$elem.clone()
    };
    ($weights:expr, $trace:expr, $elem:ident, $($elems:ident),*) => {
        $weights.$elem.clone() * $trace.$elem.clone() + apply_weights!($weights, $trace, $($elems),*)
    }
}

macro_rules! weights {
    ($($elem:ident : $ty:ty | $ty_f32:ty | $trace_ty:ty = $default:expr,)*) => {
        #[derive(Debug, Copy, Clone)]
        pub struct EvalWeights {
            $(pub $elem: $ty),*
        }
        
        impl Default for EvalWeights {
            fn default() -> Self {
                EvalWeights { $($elem: $default),* }
            }
        }
        
        impl fmt::Display for EvalWeights {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                $(writeln!(f, "{}: {},", stringify!($elem), self.$elem)?;)*
                
                Ok(())
            }
        }

        /*----------------------------------------------------------------*/

        macro_rules! impl_weight_tapered_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<T> for EvalWeights {
                    type Output = EvalWeights;

                    fn $fn(self, rhs: T) -> Self::Output {
                        EvalWeights { $($elem: self.$elem.$fn(rhs)),* }
                    }
                }
            }
        }

        macro_rules! impl_weight_i16_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<i16> for EvalWeights {
                    type Output = EvalWeights;

                    fn $fn(self, rhs: i16) -> Self::Output {
                        EvalWeights { $($elem: self.$elem.$fn(rhs)),* }
                    }
                }
            }
        }

        /*----------------------------------------------------------------*/

        impl_weight_tapered_ops!(Add, add);
        impl_weight_tapered_ops!(Sub, sub);
        impl_weight_tapered_ops!(Mul, mul);
        impl_weight_tapered_ops!(Div, div);

        impl_weight_i16_ops!(Add, add);
        impl_weight_i16_ops!(Sub, sub);
        impl_weight_i16_ops!(Mul, mul);
        impl_weight_i16_ops!(Div, div);

        /*----------------------------------------------------------------*/
        
        #[derive(Debug, Copy, Clone, Default)]
        pub struct EvalWeights_f32 {
            $(pub $elem: $ty_f32),*
        }

        impl From<EvalWeights> for EvalWeights_f32 {
            fn from(weights: EvalWeights) -> Self {
                let mut result = EvalWeights_f32::default();
                $(result.$elem = <$ty_f32>::from(weights.$elem);)*
                result
            }
        }
        
        impl fmt::Display for EvalWeights_f32 {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                $(writeln!(f, "{}: {},", stringify!($elem), self.$elem)?;)*
                
                Ok(())
            }
        }

        /*----------------------------------------------------------------*/

        macro_rules! impl_weightf32_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<EvalWeights_f32> for EvalWeights_f32 {
                    type Output = EvalWeights_f32;

                    fn $fn(self, rhs: EvalWeights_f32) -> Self::Output {
                        EvalWeights_f32 { $($elem: self.$elem.$fn(rhs.$elem)),* }
                    }
                }
            }
        }

        macro_rules! impl_weightf32_tapered_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<T_f32> for EvalWeights_f32 {
                    type Output = EvalWeights_f32;

                    fn $fn(self, rhs: T_f32) -> Self::Output {
                        EvalWeights_f32 { $($elem: self.$elem.$fn(rhs)),* }
                    }
                }
            }
        }

        macro_rules! impl_weightf32_f32_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<f32> for EvalWeights_f32 {
                    type Output = EvalWeights_f32;

                    fn $fn(self, rhs: f32) -> Self::Output {
                        EvalWeights_f32 { $($elem: self.$elem.$fn(rhs)),* }
                    }
                }
            }
        }

        /*----------------------------------------------------------------*/

        impl_weightf32_ops!(Add, add);
        impl_weightf32_ops!(Sub, sub);

        impl_weightf32_tapered_ops!(Mul, mul);
        impl_weightf32_tapered_ops!(Div, div);

        impl_weightf32_f32_ops!(Mul, mul);
        impl_weightf32_f32_ops!(Div, div);

        /*----------------------------------------------------------------*/
        
        #[cfg(feature = "trace")]
        #[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
        pub struct EvalTrace {
            pub phase: u16,
            pub stm: i16,
            $(pub $elem: $trace_ty),*
        }
        
        #[cfg(feature = "trace")]
        impl EvalTrace {
            pub fn apply_weights(&self, weights: &EvalWeights) -> Score {
                let score = apply_weights!(weights, self, $($elem),*);
                score.scale(self.phase)
            }

            pub fn apply_weights_f32(&self, weights: &EvalWeights_f32) -> f32 {
                let score = apply_weights!(weights, self, $($elem),*);
                score.scale(self.phase)
            }
        }
        
        #[cfg(feature = "trace")]
        impl fmt::Display for EvalTrace {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                $(writeln!(f, "{}: {},", stringify!($elem), self.$elem)?;)*
                Ok(())
            }
        }
    }
}

weights! {
    bishop_pair: T | T_f32 | i16 = BISHOP_PAIR,

    pawn_value: T | T_f32 | i16 = PAWN_VALUE,
    knight_value: T | T_f32 | i16 = KNIGHT_VALUE,
    bishop_value: T | T_f32 | i16 = BISHOP_VALUE,
    rook_value: T | T_f32 | i16 = ROOK_VALUE,
    queen_value: T | T_f32 | i16 = QUEEN_VALUE,

    pawn_psqt: SquareTable | SquareTable_f32 | SquarePair = PAWN_PSQT,
    knight_psqt: SquareTable | SquareTable_f32 | SquarePair = KNIGHT_PSQT,
    bishop_psqt: SquareTable | SquareTable_f32 | SquarePair = BISHOP_PSQT,
    rook_psqt: SquareTable | SquareTable_f32 | SquarePair = ROOK_PSQT,
    queen_psqt: SquareTable | SquareTable_f32 | SquarePair = QUEEN_PSQT,
    king_psqt: SquareTable | SquareTable_f32 | SquarePair = KING_PSQT,

    knight_mobility: IndexTable<9> | IndexTable_f32<9> | IndicesPair<{Square::NUM}, 9> = KNIGHT_MOBILITY,
    bishop_mobility: IndexTable<14> | IndexTable_f32<14> | IndicesPair<{Square::NUM}, 14> = BISHOP_MOBILITY,
    rook_mobility: IndexTable<15> | IndexTable_f32<15> | IndicesPair<{Square::NUM}, 15> = ROOK_MOBILITY,
    queen_mobility: IndexTable<28> | IndexTable_f32<28> | IndicesPair<{Square::NUM}, 28> = QUEEN_MOBILITY,

    rook_open_file: FileTable | FileTable_f32 | FilePair = ROOK_OPEN_FILE,
    rook_semiopen_file: FileTable | FileTable_f32 | FilePair = ROOK_SEMIOPEN_FILE,
    queen_open_file: FileTable | FileTable_f32 | FilePair = QUEEN_OPEN_FILE,
    queen_semiopen_file: FileTable | FileTable_f32 | FilePair = QUEEN_SEMIOPEN_FILE,

    knight_attack: T | T_f32 | i16 = KNIGHT_ATTACK,
    bishop_attack: T | T_f32 | i16 = BISHOP_ATTACK,
    rook_attack: T | T_f32 | i16 = ROOK_ATTACK,
    queen_attack: T | T_f32 | i16 = QUEEN_ATTACK,

    pawn_minor_threat: T | T_f32 | i16 = PAWN_MINOR_THREAT,
    pawn_major_threat: T | T_f32 | i16 = PAWN_MAJOR_THREAT,
    minor_major_threat: T | T_f32 | i16 = MINOR_MAJOR_THREAT,

    passed_pawn: RankTable | RankTable_f32 | RankPair = PASSED_PAWN,
    phalanx: RankTable | RankTable_f32 | RankPair = PHALANX,
    backwards_pawn: T | T_f32 | i16 = BACKWARDS_PAWN,
    isolated_pawn: T | T_f32 | i16 = ISOLATED_PAWN,
    doubled_pawn: T | T_f32 | i16 = DOUBLED_PAWN,
    support: T | T_f32 | i16 = SUPPORT,

    center_control: T | T_f32 | i16 = CENTER_CONTROL,
}

/*----------------------------------------------------------------*/

pub const KNIGHT_MOBILITY: IndexTable<9> = table![
    (-62, -81), (-53, -56), (-12, -31),
    (-4, -16),  (3, 5),     (13, 11),
    (22, 17),   (28, 20),   (33, 25),
];
pub const BISHOP_MOBILITY: IndexTable<14> = table![
    (-48,-59), (-20,-23),
    (16, -3), (26, 13),
    (38, 24), (51, 42),
    (55, 54), (63, 57),
    (63, 65), (68, 73),
    (81, 78), (81, 86),
    (91, 88), (98, 97),
];

pub const ROOK_MOBILITY: IndexTable<15> = table![
    (-60,-78), (-20,-17), (2, 23),
    (3, 39),   (3, 70),   (11, 99),
    (22,103),  (31,121),  (40,134),
    (40,139),  (41,158),  (48,164),
    (57,168),  (57,169),  (62,172),
];

pub const QUEEN_MOBILITY: IndexTable<28> = table![
    (-30,-48), (-12,-30), (-8, -7),  ( -9, 19),
    (20, 40),  (23, 55),  (23, 59),  (35, 75),
    (38, 78),  (53, 96),  (64, 96),  (65,100),
    (65,121),  (66,127),  (67,131),  (67,133),
    (72,136),  (72,141),  (77,147),  (79,150),
    (93,151),  (108,168), (108,168), (108,171),
    (110,182), (114,182), (114,192), (116,219),
];

pub const ROOK_OPEN_FILE: FileTable = table![
    (10, -1), (7, 1), (7, -3), (8, 6), (11, 7), (14, 1), (26, -3), (28, -10),
];
pub const ROOK_SEMIOPEN_FILE: FileTable = table![
    (8, 12), (1, 12), (11, 0), (0, 11), (-1, 9), (7, -3), (15, -8), (26, -8),
];
pub const QUEEN_OPEN_FILE: FileTable = table![
    (5, 18), (-4, 31), (-15, 36), (-13, 45), (-12, 39), (-30, 43), (-24, 40), (3, 22),
];
pub const QUEEN_SEMIOPEN_FILE: FileTable = table![
    (5, 17), (-3, 17), (2, 12), (0, 22), (1, 11), (1, 9), (15, -12), (9, 10),
];

/*----------------------------------------------------------------*/

pub const PASSED_PAWN: RankTable = table![
    (0, 0),
    (-10, 8),
    (-13, 13),
    (-7, 19),
    (1, 27),
    (17, 35),
    (31, 57),
    (0, 0),
];
pub const PHALANX: RankTable = table![
    (0, 0),
    (0, -1),
    (8, 4), 
    (6, 7),
    (6, 12),
    (5, 33),
    (40, 26),
    (0, 0),
];

pub const BACKWARDS_PAWN: T = T(-1, -2);
pub const ISOLATED_PAWN: T = T(-9, -1);
pub const DOUBLED_PAWN: T = T(-5, -3);
pub const SUPPORT: T = T(3, 1);

/*----------------------------------------------------------------*/

pub const PAWN_MINOR_THREAT: T = T(21, 37);
pub const PAWN_MAJOR_THREAT: T = T(22, 56);
pub const MINOR_MAJOR_THREAT: T = T(55, 86);

pub const KNIGHT_ATTACK: T = T(12, -2);
pub const BISHOP_ATTACK: T = T(6, -2);
pub const ROOK_ATTACK: T = T(6, 2);
pub const QUEEN_ATTACK: T = T(4, 18);

pub const CENTER_CONTROL: T = T(2, 0);

/*----------------------------------------------------------------*/

pub const BISHOP_PAIR: T = T(29, 130);

pub const PAWN_VALUE: T = T(100, 182);
pub const KNIGHT_VALUE: T = T(275, 371);
pub const BISHOP_VALUE: T = T(300, 443);
pub const ROOK_VALUE: T = T(550, 643);
pub const QUEEN_VALUE: T =  T(1850, 1743);

pub const PAWN_PSQT: SquareTable = table![
    (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),    (0, 0),  (0, 0),   (0, 0),
    (25, 51), (14, 56), (17, 52), (0, 54),  (13, 54), (28, 50), (30, 45), (22, 43),
    (17, 46), (5, 49),  (2, 45),  (-5, 46), (1, 46),  (15, 44), (12, 42), (14, 41),
    (24, 48), (16, 47), (18, 40), (11, 36), (16, 41), (20, 43), (24, 41), (21, 39),
    (35, 50), (21, 50), (19, 44), (28, 31), (30, 39), (35, 38), (31, 43), (30, 41),
    (41, 63), (33, 54), (37, 49), (37, 44), (41, 49), (46, 40), (59, 48), (40, 45),
    (91, 63), (50, 64), (41, 64), (57, 50), (15, 59), (41, 53), (31, 61), (13, 55),
    (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),   (0, 0),
];

pub const KNIGHT_PSQT: SquareTable = table![
    (-23, -18), (-15, -12), (-2, 27),   (-11, -23), (-11, -23), (-2, -27),  (-15, -12), (-23, -18),
    (-2, -20),  (-4, -22),  (11, 18),   (16, 27),   (16, 27),   (11, 18),   (-4, -22),  (-2, -20),
    (-9, -29),  (14, 22),   (10, 35),   (20, 39),   (20, 39),   (10, 35),   (14, 22),   (-9, -29),
    (-12, -33), (35, 28),   (19, 52),   (22, 41),   (22, 41),   (19, 52),   (35, 28),   (-12, -33),
    (-31, -37), (13, 35),   (22, 47),   (26, 49),   (26, 49),   (22, 47),   (13, 35),   (-31, -37),
    (-20, -45), (18, 40),   (24, 59),   (33, 51),   (33, 51),   (24, 59),   (18, 40),   (-20, -45),
    (-15, -45), (-20, -48), (34, 34),   (24, 40),   (24, 40),   (34, 34),   (-20, -48), (-15, -45),
    (-23, -13), (-5, -31),  (-11, -37), (-4, -44),  (-4, -44),  (-11, -37), (-5, 31),   (-23, -13),
];


pub const BISHOP_PSQT: SquareTable = table![
    (-28, -36), (43, 33),  (23, 35), (23, 28), (23, 29), (32, 35),  (45, 22),  (-22, -25),
    (32, 35),   (23, 32),  (33, 36), (17, 41), (26, 36), (24, 33),  (36, 26),  (34, 34),
    (20, 36),   (29, 32),  (23, 43), (28, 42), (21, 41), (23, 39),  (12, 32),  (33, 35),
    (19, 39),   (21, 37),  (31, 46), (24, 43), (21, 48), (15, 43),  (28, 40),  (27, 31),
    (30, 38),   (30, 43),  (20, 42), (31, 45), (26, 53), (28, 43),  (18, 46),  (13, 47),
    (25, 44),   (29, 47),  (23, 51), (25, 42), (32, 44), (22, 52),  (31, 44),  (31, 40),
    (13, 45),   (16, 53),  (30, 38), (13, 45), (8, 48),  (35, 43),  (-20, 54), (3, 57),
    (-6, -47),  (-15, 52), (-6, 51), (-4, 48), (-3, 52), (-10, 45), (9, 50),   (0, -45),
];

pub const ROOK_PSQT: SquareTable = table![
    (4, 35), (11, 34),  (6, 41),  (13, 34), (13, 30), (14, 33), (19, 29), (6, 37),
    (3, 40), (0, 40),   (5, 41),  (14, 28), (12, 26), (12, 39), (19, 34), (16, 34),
    (0, 48), (-1, 51),  (8, 47),  (10, 41), (17, 32), (18, 42), (17, 42), (13, 42),
    (-1, 59), (1, 59),  (4, 60),  (12, 47), (21, 48), (10, 58), (27, 53), (9, 54),
    (14, 61), (15, 64), (30, 58), (34, 50), (22, 55), (45, 55), (27, 65), (24, 62),
    (20, 64), (21, 68), (39, 60), (30, 56), (52, 50), (46, 60), (41, 64), (41, 59),
    (32, 62), (22, 66), (29, 70), (32, 57), (45, 53), (57, 58), (19, 70), (12, 70),
    (22, 59), (17, 67), (2, 69),  (11, 58), (18, 53), (30, 60), (28, 65), (15, 67),
];

pub const QUEEN_PSQT: SquareTable = table![
    (24, 35),  (31, 30),  (29, 30), (33, 49),  (33, 44), (16, 34), (18, 33),  (31, 26),
    (22, 61),  (35, 41),  (30, 53), (31, 47),  (31, 59), (32, 47), (43, 18),  (50, 30),
    (20, 71),  (29, 66),  (28, 69), (26, 72),  (29, 63), (34, 81), (38, 71),  (50, 47),
    (23, 97),  (16, 85),  (22, 90), (21, 91),  (22, 84), (27, 85), (31, 85),  (43, 75),
    (21, 98),  (25, 101), (31, 87), (12, 100), (23, 97), (33, 91), (32, 91),  (46, 66),
    (26, 102), (20, 103), (35, 94), (25, 90),  (33, 98), (35, 94), (31, 107), (41, 88),
    (26, 94),  (24, 102), (31, 97), (34, 96),  (39, 96), (45, 94), (31, 104), (45, 97),
    (39, 75),  (40, 78),  (29, 86), (16, 80),  (43, 83), (56, 78), (68, 87),  (57, 82),
];

pub const KING_PSQT: SquareTable = table![
    (271, 1),   (327, 45),  (271, 85),  (198, 76),  (198, 76),  (271, 85),  (327, 45),  (271, 1),
    (278, 53),  (303, 100), (234, 133), (179, 135), (179, 135), (234, 133), (303, 100), (278, 53),
    (195, 88),  (258, 130), (169, 169), (120, 175), (120, 175), (169, 169), (258, 130), (195, 88),
    (164, 103), (190, 156), (138, 172), (98, 172),  (98, 172),  (138, 172), (190, 156), (164, 103),
    (154, 96),  (179, 166), (105, 199), (70, 199),  (70, 199),  (105, 199), (179, 166), (154, 96),
    (123, 92),  (145, 172), (81, 184),  (31, 191),  (31, 191),  (81, 184),  (145, 172), (123, 92),
    (88, 47),   (120, 121), (65, 116),  (33, 131),  (33, 131),  (65, 116),  (120, 121), (88, 47),
    (59, 11),   (89, 59),   (45, 73),   (-1, 78),   (-1, 78),   (45, 73),   (89, 59),   (59, 11),
];

pub const fn calc_piece_table(table: SquareTable, value: T) -> SquareTable {
    let mut result = table;
    let mut i = 0;
    
    while i < Square::NUM {
        result.0[i] = T(value.0 + result.0[i].0, value.1 + result.0[i].1);
        i += 1;
    }
    
    result
}

pub const PAWN_TABLE: SquareTable = calc_piece_table(PAWN_PSQT, PAWN_VALUE);
pub const KNIGHT_TABLE: SquareTable = calc_piece_table(KNIGHT_PSQT, KNIGHT_VALUE);
pub const BISHOP_TABLE: SquareTable = calc_piece_table(BISHOP_PSQT, BISHOP_VALUE);
pub const ROOK_TABLE: SquareTable = calc_piece_table(ROOK_PSQT, ROOK_VALUE);
pub const QUEEN_TABLE: SquareTable = calc_piece_table(QUEEN_PSQT, QUEEN_VALUE);
pub const KING_TABLE: SquareTable = KING_PSQT;

/*----------------------------------------------------------------*/

pub const fn king_zone(sq: Square, color: Color) -> BitBoard {
    const fn calc_zone(sq: Square, color: Color) -> BitBoard {
        let moves = BitBoard(get_king_moves(sq).0 | sq.bitboard().0);
        
        match color {
            Color::White => BitBoard(moves.0 | (moves.0 << 8)),
            Color::Black => BitBoard(moves.0 | (moves.0 >> 8)),
        }
    }
    
    const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];
        let mut i = 0;
        
        while i < Color::NUM {
            let mut j = 0;
            while j < Square::NUM {
                table[i][j] = calc_zone(Square::index_const(j), Color::index_const(i));
                
                j += 1;
            }
            
            i += 1;
        }
        
        table
    };
    
    TABLE[color as usize][sq as usize]
}