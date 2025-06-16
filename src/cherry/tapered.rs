use std::ops::*;
use super::*;

/*----------------------------------------------------------------*/

#[macro_export]
macro_rules! table {
    ($(($mg:expr, $eg:expr),)*) => {
        IndexTable::new([$(T($mg, $eg),)*])
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(Default)]
pub struct T(pub i16, pub i16);

impl T {
    #[inline(always)]
    pub fn new_mg(score: i16) -> T {
        T(score, 0)
    }

    #[inline(always)]
    pub fn new_eg(score: i16) -> T {
        T(0, score)
    }

    #[inline(always)]
    pub fn scale(self, phase: u16) -> Score {
        let phase = (phase * TAPER_SCALE + TOTAL_PHASE / 2) / TOTAL_PHASE;
        let score = (self.0 as i32 * (TAPER_SCALE - phase) as i32 + self.1 as i32 * phase as i32) / TAPER_SCALE as i32;

        Score(score as i16)
    }

    #[inline(always)]
    pub const fn mg(self) -> T {
        T(self.0, 0)
    }

    #[inline(always)]
    pub const fn eg(self) -> T {
        T(0, self.1)
    }

    pub const ZERO: T = T(0, 0);
}

/*----------------------------------------------------------------*/

macro_rules! impl_tapered_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<T> for T {
            type Output = T;

            #[inline(always)]
            fn $fn(self, rhs: T) -> Self::Output {
                T(self.0.$fn(rhs.0), self.1.$fn(rhs.1))
            }
        }
    )*};
}

macro_rules! impl_tapered_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<T> for T {
            #[inline(always)]
            fn $fn(&mut self, rhs: T) {
                self.0.$fn(rhs.0);
                self.1.$fn(rhs.1);
            }
        }
    )*};
}

macro_rules! impl_tapered_i16_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<i16> for T {
            type Output = T;

            #[inline(always)]
            fn $fn(self, rhs: i16) -> Self::Output {
                T(self.0.$fn(rhs), self.1.$fn(rhs))
            }
        }

        impl $trait<T> for i16 {
            type Output = T;

            #[inline(always)]
            fn $fn(self, rhs: T) -> Self::Output {
                T(self.$fn(rhs.0), self.$fn(rhs.1))
            }
        }
    )*};
}

macro_rules! impl_tapered_i16_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<i16> for T {
            #[inline(always)]
            fn $fn(&mut self, rhs: i16) {
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

impl_tapered_i16_ops! {
    Mul, mul;
    Div, div;
}

impl_tapered_i16_assign_ops! {
    MulAssign, mul_assign;
    DivAssign, div_assign;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IndexTable<const COUNT: usize>(pub [T; COUNT]);

impl<const COUNT: usize> IndexTable<COUNT> {
    #[inline(always)]
    pub const fn new(table: [T; COUNT]) -> Self {
        Self(table)
    }
}

impl<const COUNT: usize> Index<usize> for IndexTable<COUNT> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const COUNT: usize> IndexMut<usize> for IndexTable<COUNT> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_table_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait for IndexTable<COUNT> {
            type Output = Self;

            fn $fn(self, rhs: Self) -> Self::Output {
                let mut result = self;
                let mut i = 0;
                
                while i < COUNT {
                    result.0[i] = T(self.0[i].0.$fn(rhs.0[i].0), self.0[i].1.$fn(rhs.0[i].1));
                    i += 1;
                }
                
                result
            }
        }
    )*};
}

macro_rules! impl_table_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait for IndexTable<COUNT> {
            fn $fn(&mut self, rhs: Self) {
                let mut i = 0;
                
                while i < COUNT {
                    self.0[i] = T(self.0[i].0.$op(rhs.0[i].0), self.0[i].1.$op(rhs.0[i].1));
                    i += 1;
                }
            }
        }
    )*};
}

macro_rules! impl_table_tapered_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait<T> for IndexTable<COUNT> {
            type Output = Self;
            
            fn $fn(self, rhs: T) -> Self::Output {
                let mut result = self;
                let mut i = 0;
                
                while i < COUNT {
                    result[i] = T(self.0[i].0.$fn(rhs.0), self.0[i].1.$fn(rhs.1));
                    i += 1;
                }
                
                result
            }
        }
    )*};
}

macro_rules! impl_table_tapered_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait<T> for IndexTable<COUNT> {
            fn $fn(&mut self, rhs: T) {
                let mut i = 0;
                
                while i < COUNT {
                    self.0[i] = T(self.0[i].0.$op(rhs.0), self.0[i].1.$op(rhs.1));
                    i += 1;
                }
            }
        }
    )*};
}


macro_rules! impl_table_i16_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait<i16> for IndexTable<COUNT> {
            type Output = Self;

            fn $fn(self, rhs: i16) -> Self::Output {
                let mut result = self;
                let mut i = 0;
                
                while i < COUNT {
                    result.0[i] = T(self.0[i].0.$fn(rhs), self.0[i].1.$fn(rhs));
                    i += 1;
                }
                
                result
            }
        }
    )*};
}

macro_rules! impl_table_i16_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait<i16> for IndexTable<COUNT> {
            fn $fn(&mut self, rhs: i16) {
                let mut i = 0;
                
                while i < COUNT {
                    self.0[i] = T(self.0[i].0.$op(rhs), self.0[i].1.$op(rhs));
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

impl_table_i16_ops! {
    Mul, mul;
    Div, div;
}

impl_table_i16_assign_ops! {
    MulAssign, mul_assign, mul;
    DivAssign, div_assign, div;
}
