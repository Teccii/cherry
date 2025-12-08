use core::{ops::*, ptr};

use crate::*;

/*----------------------------------------------------------------*/

/*
Bit Layout:
- Bits 0-4: Piece Index
- Bits 5-7: Piece Type
- Bit 8: Piece Color
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Place(pub u8);

impl Place {
    #[inline]
    pub const fn from_piece(piece: Piece, color: Color, index: PieceIndex) -> Place {
        Place(index.0 | (piece.bits() << 4) | ((color as u8) << 7))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn index(self) -> Option<PieceIndex> {
        if self.is_empty() {
            return None;
        }

        Some(PieceIndex(self.0 & 0b1111))
    }

    #[inline]
    pub const fn piece(self) -> Option<Piece> {
        if self.is_empty() {
            return None;
        }

        Piece::from_bits((self.0 >> 4) & 0b111)
    }

    #[inline]
    pub const fn color(self) -> Option<Color> {
        if self.is_empty() {
            return None;
        }

        Color::try_index((self.0 >> 7) as usize)
    }

    /*----------------------------------------------------------------*/

    pub const EMPTY: Place = Place(0);
    pub const INDEX_MASK: u8 = 0xF;
    pub const PIECE_MASK: u8 = 0x70;
    pub const COLOR_MASK: u8 = 0x80;
    pub const SLIDER_BIT: u8 = 0b100 << 4;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Byteboard(pub u8x64);

impl Byteboard {
    #[inline]
    pub fn as_mailbox(&self) -> &[Place; Square::COUNT] {
        unsafe { &*ptr::from_ref(&self.0).cast() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get(&self, sq: Square) -> Place {
        self.as_mailbox()[sq]
    }

    #[inline]
    pub fn set(&mut self, sq: Square, place: Place) {
        self.0 = u8x64::blend(
            self.0,
            u8x64::splat(place.0),
            Mask64::from_bitmask(sq.bitboard().0),
        );
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn empty(&self) -> Bitboard {
        Bitboard(self.0.zero().to_bitmask())
    }

    #[inline]
    pub fn occupied(&self) -> Bitboard {
        Bitboard(self.0.nonzero().to_bitmask())
    }

    #[inline]
    pub fn colors(&self, color: Color) -> Bitboard {
        let black = self.0.msb().to_bitmask();
        match color {
            Color::White => self.occupied() ^ black,
            Color::Black => Bitboard(black),
        }
    }

    #[inline]
    pub fn pieces(&self, piece: Piece) -> Bitboard {
        Bitboard(
            u8x64::eq(
                self.0 & u8x64::splat(Place::PIECE_MASK),
                u8x64::splat(piece.bits() << 4),
            )
            .to_bitmask(),
        )
    }

    #[inline]
    pub fn color_pieces(&self, color: Color, piece: Piece) -> Bitboard {
        self.colors(color) & self.pieces(piece)
    }
}

impl Deref for Byteboard {
    type Target = u8x64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Byteboard {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Wordboard(pub u16x64);

impl Wordboard {
    #[inline]
    pub fn as_mailbox(&self) -> &[PieceMask; Square::COUNT] {
        unsafe { &*ptr::from_ref(&self.0).cast() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get(&self, sq: Square) -> PieceMask {
        self.as_mailbox()[sq]
    }

    #[inline]
    pub fn all(&self) -> Bitboard {
        Bitboard(self.0.nonzero().to_bitmask())
    }

    #[inline]
    pub fn for_mask(&self, mask: PieceMask) -> Bitboard {
        Bitboard(u16x64::test(self.0, u16x64::splat(mask.0)).to_bitmask())
    }
}

impl Deref for Wordboard {
    type Target = u16x64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Wordboard {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct IndexToPiece(pub [Option<Piece>; PieceIndex::COUNT]);

impl IndexToPiece {
    #[inline]
    pub fn valid(&self) -> PieceMask {
        PieceMask(
            u8x16::neq(
                unsafe { u8x16::load(self.0.as_ptr()) },
                u8x16::splat(Self::INVALID_PIECE),
            )
            .to_bitmask(),
        )
    }

    #[inline]
    pub fn mask_eq(&self, piece: Piece) -> PieceMask {
        PieceMask(
            u8x16::eq(
                unsafe { u8x16::load(self.0.as_ptr()) },
                u8x16::splat(piece as u8),
            )
            .to_bitmask(),
        )
    }

    const INVALID_PIECE: u8 = unsafe { core::mem::transmute::<Option<Piece>, u8>(None) };
}

impl Default for IndexToPiece {
    #[inline]
    fn default() -> Self {
        IndexToPiece([None; PieceIndex::COUNT])
    }
}

impl Index<PieceIndex> for IndexToPiece {
    type Output = Option<Piece>;

    #[inline]
    fn index(&self, index: PieceIndex) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl IndexMut<PieceIndex> for IndexToPiece {
    #[inline]
    fn index_mut(&mut self, index: PieceIndex) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct IndexToSquare(pub [Option<Square>; PieceIndex::COUNT]);

impl IndexToSquare {
    #[inline]
    pub fn valid(&self) -> PieceMask {
        PieceMask(
            u8x16::neq(
                unsafe { u8x16::load(self.0.as_ptr()) },
                u8x16::splat(Self::INVALID_SQUARE),
            )
            .to_bitmask(),
        )
    }

    #[inline]
    pub fn mask_eq(&self, sq: Square) -> PieceMask {
        PieceMask(
            u8x16::eq(
                unsafe { u8x16::load(self.0.as_ptr()) },
                u8x16::splat(sq as u8),
            )
            .to_bitmask(),
        )
    }

    const INVALID_SQUARE: u8 = unsafe { core::mem::transmute::<Option<Square>, u8>(None) };
}

impl Default for IndexToSquare {
    #[inline]
    fn default() -> Self {
        IndexToSquare([None; PieceIndex::COUNT])
    }
}

impl Index<PieceIndex> for IndexToSquare {
    type Output = Option<Square>;

    #[inline]
    fn index(&self, index: PieceIndex) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl IndexMut<PieceIndex> for IndexToSquare {
    #[inline]
    fn index_mut(&mut self, index: PieceIndex) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}
