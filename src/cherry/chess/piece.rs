use core::ops::*;

use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PieceIndex(u8);

impl PieceIndex {
    #[inline]
    pub const fn new(i: u8) -> PieceIndex {
        assert!(i < 16);

        PieceIndex(i)
    }

    #[inline]
    pub const fn into_inner(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn into_mask(self) -> PieceMask {
        PieceMask::new(1 << self.0)
    }

    pub const COUNT: usize = 16;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct PieceMask(u16);

impl PieceMask {
    #[inline]
    pub const fn new(raw: u16) -> PieceMask {
        PieceMask(raw)
    }

    #[inline]
    pub const fn into_inner(self) -> u16 {
        self.0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn has(self, index: PieceIndex) -> bool {
        (self.0 >> index.0) & 1 == 1
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
    pub const fn msb(self) -> PieceIndex {
        PieceIndex::new(self.0.leading_zeros() as u8)
    }

    #[inline]
    pub const fn lsb(self) -> PieceIndex {
        PieceIndex::new(self.0.trailing_zeros() as u8)
    }

    /*----------------------------------------------------------------*/

    pub const EMPTY: PieceMask = PieceMask(0);
    pub const KING: PieceMask = PieceMask(1);
}

impl IntoIterator for PieceMask {
    type Item = PieceIndex;
    type IntoIter = PieceMaskIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        PieceMaskIter(self)
    }
}

impl Not for PieceMask {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        PieceMask(!self.into_inner())
    }
}

macro_rules! impl_piece_mask_ops {
    ($($trait:ident, $fn:ident;)*) => {
        $(
            impl $trait for PieceMask {
                type Output = Self;

                #[inline]
                fn $fn(self, rhs: Self) -> Self::Output {
                    PieceMask(self.into_inner().$fn(rhs.into_inner()))
                }
            }
        )*
    }
}

macro_rules! impl_piece_mask_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {
        $(
            impl $trait for PieceMask {
                #[inline]
                fn $fn(&mut self, rhs: Self) {
                    self.0.$fn(rhs.0);
                }
            }
        )*
    }
}

impl_piece_mask_ops! {
    BitAnd, bitand;
    BitOr, bitor;
    BitXor, bitxor;
}

impl_piece_mask_assign_ops! {
    BitAndAssign, bitand_assign;
    BitOrAssign, bitor_assign;
    BitXorAssign, bitxor_assign;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct PieceMaskIter(PieceMask);

impl Iterator for PieceMaskIter {
    type Item = PieceIndex;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        let index = self.0.lsb();
        self.0 &= PieceMask::new(self.0.into_inner().wrapping_sub(1));

        Some(index)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.popcnt(), Some(self.0.popcnt()))
    }
}

impl ExactSizeIterator for PieceMaskIter {
    #[inline]
    fn len(&self) -> usize {
        self.0.popcnt()
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct IndexToSquare {
    inner: [Option<Square>; PieceIndex::COUNT],
}

impl IndexToSquare {
    #[inline]
    pub fn into_inner(self) -> [Option<Square>; PieceIndex::COUNT] {
        self.inner
    }

    #[inline]
    pub fn valid(&self) -> PieceMask {
        PieceMask::new(Vec128::neq8(
            unsafe { Vec128::load(self.inner.as_ptr()) },
            Vec128::splat8(Self::INVALID_SQUARE),
        ))
    }

    #[inline]
    pub fn mask_eq(&self, sq: Square) -> PieceMask {
        PieceMask::new(Vec128::eq8(
            unsafe { Vec128::load(self.inner.as_ptr()) },
            Vec128::splat8(sq as u8),
        ))
    }

    const INVALID_SQUARE: u8 = unsafe { ::core::mem::transmute::<Option<Square>, u8>(None) };
}

impl Default for IndexToSquare {
    #[inline]
    fn default() -> Self {
        IndexToSquare {
            inner: [None; PieceIndex::COUNT],
        }
    }
}

impl Index<PieceIndex> for IndexToSquare {
    type Output = Option<Square>;

    #[inline]
    fn index(&self, index: PieceIndex) -> &Self::Output {
        &self.inner[index.into_inner() as usize]
    }
}

impl IndexMut<PieceIndex> for IndexToSquare {
    #[inline]
    fn index_mut(&mut self, index: PieceIndex) -> &mut Self::Output {
        &mut self.inner[index.into_inner() as usize]
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
#[repr(C, align(16))]
pub struct IndexToPiece {
    inner: [Option<Piece>; PieceIndex::COUNT],
}

impl IndexToPiece {
    #[inline]
    pub fn into_inner(self) -> [Option<Piece>; PieceIndex::COUNT] {
        self.inner
    }

    #[inline]
    pub fn valid(&self) -> PieceMask {
        PieceMask::new(Vec128::neq8(
            unsafe { Vec128::load(self.inner.as_ptr()) },
            Vec128::splat8(Self::INVALID_PIECE),
        ))
    }

    #[inline]
    pub fn mask_eq(&self, piece: Piece) -> PieceMask {
        PieceMask::new(Vec128::eq8(
            unsafe { Vec128::load(self.inner.as_ptr()) },
            Vec128::splat8(piece as u8),
        ))
    }

    const INVALID_PIECE: u8 = unsafe { ::core::mem::transmute::<Option<Piece>, u8>(None) };
}

impl Default for IndexToPiece {
    #[inline]
    fn default() -> Self {
        IndexToPiece {
            inner: [None; PieceIndex::COUNT],
        }
    }
}

impl Index<PieceIndex> for IndexToPiece {
    type Output = Option<Piece>;

    #[inline]
    fn index(&self, index: PieceIndex) -> &Self::Output {
        &self.inner[index.into_inner() as usize]
    }
}

impl IndexMut<PieceIndex> for IndexToPiece {
    #[inline]
    fn index_mut(&mut self, index: PieceIndex) -> &mut Self::Output {
        &mut self.inner[index.into_inner() as usize]
    }
}
