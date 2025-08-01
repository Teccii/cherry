use std::{fmt, ops::*};
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline]
    pub const fn shift<D: Direction>(self, steps: i8) -> Bitboard {
        /*
        For some reason, `shl` takes an `isize` as a parameter but then panics if you try to shift
        by a negative number. This makes no sense. It should just do `shr` if it's negative, or just
        not take in an `isize` if it's not supposed to be negative...
        */

        let mask = horizontal_shift_mask(D::DX * steps);
        let shift = (D::DX + D::DY * 8) * steps;

        Bitboard(if shift > 0 {
            (self.0 << shift) & mask
        } else {
            (self.0 >> -shift) & mask
        })
    }
    
    #[inline]
    pub const fn smear<D: Direction>(self) -> Bitboard {
        let mut result = self;

        result.0 |= result.shift::<D>(1).0;
        result.0 |= result.shift::<D>(2).0;
        result.0 |= result.shift::<D>(4).0;

        result
    }
    
    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn flip_files(self) -> Bitboard {
        const K1: u64 = 0x5555555555555555;
        const K2: u64 = 0x3333333333333333;
        const K4: u64 = 0x0F0F0F0F0F0F0F0F;

        let mut result = self.0;
        result = ((result >> 1) & K1) | ((result & K1) << 1);
        result = ((result >> 2) & K2) | ((result & K2) << 2);
        result = ((result >> 4) & K4) | ((result & K4) << 4);

        Bitboard(result)
    }

    #[inline]
    pub const fn flip_ranks(self) -> Bitboard {
        Bitboard(self.0.swap_bytes())
    }

    #[inline]
    pub const fn relative_to(self, color: Color) -> Bitboard {
        match color {
            Color::White => self,
            Color::Black => self.flip_ranks()
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn next_square(self) -> Square {
        Square::index(self.0.trailing_zeros() as usize)
    }
    
    #[inline]
    pub const fn try_next_square(self) -> Option<Square> {
        Square::try_index(self.0.trailing_zeros() as usize)
    }

    #[inline]
    pub const fn next_square_back(self) -> Square {
        Square::index(63 - self.0.leading_zeros() as usize)
    }

    #[inline]
    pub const fn try_next_square_back(self) -> Option<Square> {
        Square::try_index(63 - self.0.leading_zeros() as usize)
    }
    
    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn is_superset(self, rhs: Bitboard) -> bool {
        rhs.is_subset(self)
    }
    
    #[inline]
    pub const fn is_subset(self, rhs: Bitboard) -> bool {
        self.0 & rhs.0 == self.0
    }
    
    #[inline]
    pub const fn is_disjoint(self, rhs: Bitboard) -> bool {
        self.0 & rhs.0 == 0
    }
    
    #[inline]
    pub const fn invert(self) -> Bitboard {
        Bitboard(!self.0)
    }

    #[inline]
    pub const fn intersection(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }

    #[inline]
    pub const fn union(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 | rhs.0)
    }

    /*----------------------------------------------------------------*/
    
    #[inline]
    pub const fn has(self, sq: Square) -> bool {
        !self.is_disjoint(sq.bitboard())
    }

    #[inline]
    pub const fn popcnt(self) -> usize {
        self.0.count_ones() as usize
    }
    
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn iter(self) -> BitboardIter {
        BitboardIter(self)
    }

    #[inline]
    pub const fn iter_subsets(self) -> BitboardSubsetIter {
        BitboardSubsetIter {
            set: self,
            subset: Bitboard::EMPTY,
            finished: false,
        }
    }
    
    /*----------------------------------------------------------------*/

    pub const EMPTY: Bitboard = Bitboard(0);
    pub const FULL: Bitboard = Bitboard(u64::MAX);
    pub const CORNER: Bitboard = Bitboard(0x8100000000000081);
    pub const EDGES: Bitboard = Bitboard(0xFF818181818181FF);
    pub const CENTER: Bitboard = Bitboard(0x0000001818000000);
    pub const BIG_CENTER: Bitboard = Bitboard(0x00003c3c3c3c0000);
    pub const DARK_SQUARES: Bitboard = Bitboard(0xAA55AA55AA55AA55);
    pub const LIGHT_SQUARES: Bitboard = Bitboard(0x55AA55AA55AA55AA);
}

/*----------------------------------------------------------------*/

impl From<u64> for Bitboard {
    #[inline]
    fn from(value: u64) -> Self {
        Bitboard(value)
    }
}

impl Deref for Bitboard {
    type Target = u64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bitboard {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = BitboardIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitboardIter(self)
    }
}

/*----------------------------------------------------------------*/

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bitboard({:#016X})", self.0)
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &rank in Rank::ALL.iter().rev() {
            write!(f, "\n")?;
            
            for &file in &File::ALL{
                if self.has(Square::new(file, rank)) {
                    write!(f, "\tx")?;
                } else {
                    write!(f, "\t.")?;
                }
            }
        }
        
        Ok(())
    }
}

/*----------------------------------------------------------------*/

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_bb_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<Bitboard> for Bitboard {
            type Output = Bitboard;
            
            #[inline]
            fn $fn(self, rhs: Bitboard) -> Self::Output {
                Bitboard(self.0.$fn(rhs.0))
            }
        }
    
        impl $trait<u64> for Bitboard {
            type Output = Bitboard;
            
            #[inline]
            fn $fn(self, rhs: u64) -> Self::Output {
                Bitboard(self.0.$fn(rhs))
            }
        }
    
        impl $trait<Bitboard> for u64 {
            type Output = Bitboard;
            
            #[inline]
            fn $fn(self, rhs: Bitboard) -> Self::Output {
                Bitboard(self.$fn(rhs.0))
            }
        }

        impl $trait<Square> for Bitboard {
            type Output = Bitboard;

            #[inline]
            fn $fn(self, rhs: Square) -> Self::Output {
                Bitboard(self.0.$fn(rhs.bitboard().0))
            }
        }
    )*}
}

macro_rules! impl_bb_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<Bitboard> for Bitboard {
            #[inline]
            fn $fn(&mut self, rhs: Bitboard) {
                self.0.$fn(rhs.0);
            }
        }
    
        impl $trait<u64> for Bitboard {
            #[inline]
            fn $fn(&mut self, rhs: u64) {
                self.0.$fn(rhs);
            }
        }

        impl $trait<Square> for Bitboard {
            #[inline]
            fn $fn(&mut self, rhs: Square) {
                self.0.$fn(rhs.bitboard().0);
            }
        }
    )*}
}

macro_rules! impl_bb_shift_ops {
    ($($ty:ty,)*) => {$(
        impl Shl<$ty> for Bitboard {
            type Output = Bitboard;

            #[inline]
            fn shl(self, rhs: $ty) -> Self::Output {
                Bitboard(self.0 << rhs)
            }
        }

        impl Shr<$ty> for Bitboard {
            type Output = Bitboard;

            #[inline]
            fn shr(self, rhs: $ty) -> Self::Output {
                Bitboard(self.0 >> rhs)
            }
        }

        impl ShlAssign<$ty> for Bitboard {
            #[inline]
            fn shl_assign(&mut self, rhs: $ty) {
                self.0 <<= rhs;
            }
        }

        impl ShrAssign<$ty> for Bitboard {
            #[inline]
            fn shr_assign(&mut self, rhs: $ty) {
                self.0 >>= rhs;
            }
        }
    )*}
}

/*----------------------------------------------------------------*/

impl_bb_ops! {
    BitAnd, bitand;
    BitOr, bitor;
    BitXor, bitxor; 
}

impl_bb_assign_ops! {
    BitAndAssign, bitand_assign;
    BitOrAssign, bitor_assign;
    BitXorAssign, bitxor_assign;
}

impl_bb_shift_ops! {
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize,
}

/*----------------------------------------------------------------*/

pub struct BitboardIter(Bitboard);

impl Iterator for BitboardIter {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let sq = self.0.try_next_square();
        
        if let Some(sq) = sq {
            self.0 ^= sq.bitboard();
        }
        
        sq
    }
}

impl DoubleEndedIterator for BitboardIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let sq = self.0.try_next_square_back();
        
        if let Some(sq) = sq {
            self.0 ^= sq.bitboard();
        }
        
        sq
    }
}

/*----------------------------------------------------------------*/

pub struct BitboardSubsetIter {
    set: Bitboard,
    subset: Bitboard,
    finished: bool
}

impl Iterator for BitboardSubsetIter {
    type Item = Bitboard;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let current = self.subset;
        self.subset.0 = self.subset.0.wrapping_sub(self.set.0) & self.set.0;
        self.finished = self.subset.is_empty();

        Some(current)
    }
}