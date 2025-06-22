use std::{fmt, ops::*};
use crate::{Direction, File, Rank, Square};

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline(always)]
    pub const fn shift<D: Direction>(self, steps: usize) -> Bitboard {
        /*
        For some reason, `shl` takes an `isize` as a parameter but then panics if you try to shift
        by a negative number. This makes no sense. It should just do `shr` if it's negative, or just
        not take in an `isize` if it's not supposed to be negative...
        */

        let mut result = self;
        let mut i = 0;

        while i < steps {
            result = if D::SHIFT > 0 {
                Bitboard((result.0 & D::MASK.0) << D::SHIFT)
            } else {
                Bitboard((result.0 & D::MASK.0) >> -D::SHIFT)
            };
            
            i += 1;
        }

        result
    }
    
    #[inline(always)]
    pub const fn smear<D: Direction>(self) -> Bitboard {
        let mut result = self;

        result.0 |= result.shift::<D>(1).0;
        result.0 |= result.shift::<D>(2).0;
        result.0 |= result.shift::<D>(4).0;

        result
    }
    
    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn next_square(self) -> Square {
        Square::index(self.0.trailing_zeros() as usize)
    }
    
    #[inline(always)]
    pub const fn try_next_square(self) -> Option<Square> {
        Square::try_index(self.0.trailing_zeros() as usize)
    }
    
    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn is_superset(self, rhs: Bitboard) -> bool {
        rhs.is_subset(self)
    }
    
    #[inline(always)]
    pub const fn is_subset(self, rhs: Bitboard) -> bool {
        self.0 & rhs.0 == self.0
    }
    
    #[inline(always)]
    pub const fn is_disjoint(self, rhs: Bitboard) -> bool {
        self.0 & rhs.0 == 0
    }

    /*----------------------------------------------------------------*/
    
    #[inline(always)]
    pub const fn has(self, sq: Square) -> bool {
        self.is_disjoint(sq.bitboard())
    }

    #[inline(always)]
    pub const fn popcnt(self) -> usize {
        self.0.count_ones() as usize
    }
    
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /*----------------------------------------------------------------*/

    pub const EMPTY: Bitboard = Bitboard(0);
    pub const FULL: Bitboard = Bitboard(u64::MAX);
    
    pub const EDGES: Bitboard = Bitboard(0xFF818181818181FF);
    pub const DARK_SQUARES: Bitboard = Bitboard(0xAA55AA55AA55AA55);
    pub const LIGHT_SQUARES: Bitboard = Bitboard(0x55AA55AA55AA55AA);
}

/*----------------------------------------------------------------*/

impl From<u64> for Bitboard {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Bitboard(value)
    }
}

impl Deref for Bitboard {
    type Target = u64;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bitboard {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/*----------------------------------------------------------------*/

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

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl Shl<usize> for Bitboard {
    type Output = Bitboard;
    
    #[inline(always)]
    fn shl(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl Shr<usize> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn shr(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

impl ShlAssign<usize> for Bitboard {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: usize) {
        self.0 <<= rhs;
    }
}

impl ShrAssign<usize> for Bitboard {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: usize) {
        self.0 >>= rhs;
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_bb_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<Bitboard> for Bitboard {
            type Output = Bitboard;
            
            #[inline(always)]
            fn $fn(self, rhs: Bitboard) -> Self::Output {
                Bitboard(self.0.$fn(rhs.0))
            }
        }
    
        impl $trait<u64> for Bitboard {
            type Output = Bitboard;
            
            #[inline(always)]
            fn $fn(self, rhs: u64) -> Self::Output {
                Bitboard(self.0.$fn(rhs))
            }
        }
    
        impl $trait<Bitboard> for u64 {
            type Output = Bitboard;
            
            #[inline(always)]
            fn $fn(self, rhs: Bitboard) -> Self::Output {
                Bitboard(self.$fn(rhs.0))
            }
        }
    )*}
}

macro_rules! impl_bb_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<Bitboard> for Bitboard {
            #[inline(always)]
            fn $fn(&mut self, rhs: Bitboard) {
                self.0.$fn(rhs.0);
            }
        }
    
        impl $trait<u64> for Bitboard {
            #[inline(always)]
            fn $fn(&mut self, rhs: u64) {
                self.0.$fn(rhs);
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

/*----------------------------------------------------------------*/

pub struct BitboardIter(Bitboard);

impl Iterator for BitboardIter {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let sq = self.0.try_next_square();
        
        if let Some(sq) = sq {
            self.0 ^= sq.bitboard();
        }
        
        sq
    }
}