use std::ops::BitAnd;
use crate::*;

/*----------------------------------------------------------------*/

/*
Bit Layout:
bits 0-4: Piece Index,
bits 5-7: Piece Type,
bit 8: Color
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Place(u8);

impl Place {
    #[inline]
    pub const fn new(raw: u8) -> Place {
        Place(raw)
    }

    #[inline]
    pub const fn from_piece(piece: Piece, color: Color, index: PieceIndex) -> Place {
        Place::new((piece.bits() << 4) | index.into_inner() | color.msb())
    }

    #[inline]
    pub const fn into_inner(self) -> u8 {
        self.0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn color(self) -> Option<Color> {
        if self.is_empty() {
            return None;
        }

        Color::try_index(((self.0 & 0x80) != 0) as usize)
    }

    #[inline]
    pub const fn piece(self) -> Option<Piece> {
        if self.is_empty() {
            return None;
        }

        match (self.0 >> 4) & 0b111 {
            0b010 => Some(Piece::Pawn),
            0b011 => Some(Piece::Knight),
            0b101 => Some(Piece::Bishop),
            0b110 => Some(Piece::Rook),
            0b111 => Some(Piece::Queen),
            0b001 => Some(Piece::King),
            _ => None,
        }
    }

    #[inline]
    pub const fn index(self) -> Option<PieceIndex> {
        if self.is_empty() {
            return None;
        }

        Some(PieceIndex::new(self.0 & 0b1111))
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /*----------------------------------------------------------------*/

    pub const EMPTY: Place = Place(0);
    pub const SLIDER_BIT: u8 = 0b100 <<4;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Byteboard {
    pub(crate) inner: Vec512,
}

impl Byteboard {
    #[inline]
    pub fn new(raw: Vec512) -> Byteboard {
        Byteboard { inner: raw }
    }

    #[inline]
    pub fn into_mailbox(self) -> [Place; Square::COUNT] {
        unsafe { core::mem::transmute::<Vec512, [Place; Square::COUNT]>(self.inner) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub(crate) fn get(&self, sq: Square) -> Place {
        Place::new(Vec512::permute8(Vec512::splat8(sq as u8), self.inner).into_u32() as u8)
    }

    #[inline]
    pub(crate) fn set(&mut self, sq: Square, place: Place) {
        self.inner = Vec512::mask_splat8(self.inner, sq.bitboard().0, place.into_inner());
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn empty(&self) -> Bitboard {
        Bitboard(self.inner.zero8())
    }

    #[inline]
    pub fn occupied(&self) -> Bitboard {
        Bitboard(self.inner.nonzero8())
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn colors(&self, color: Color) -> Bitboard {
        let black = self.inner.msb8();

        match color {
            Color::White => self.occupied() ^ black,
            Color::Black => Bitboard(black),
        }
    }

    #[inline]
    pub fn pieces(&self, piece: Piece) -> Bitboard {
        Bitboard(Vec512::eq8(
            self.inner & Vec512::splat8(0b1110000),
            Vec512::splat8(piece.bits() << 4)
        ))
    }

    #[inline]
    pub fn color_pieces(&self, piece: Piece, color: Color) -> Bitboard {
        Bitboard(Vec512::mask_eq8(
            self.colors(color).0,
            self.inner & Vec512::splat8(0b1110000),
            Vec512::splat8(piece.bits() << 4)
        ))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn minors(&self) -> Bitboard {
        self.pieces(Piece::Knight) | self.pieces(Piece::Bishop)
    }

    #[inline]
    pub fn color_minors(&self, color: Color) -> Bitboard {
        self.colors(color) & self.minors()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn majors(&self) -> Bitboard {
        self.pieces(Piece::Rook) | self.pieces(Piece::Queen)
    }

    #[inline]
    pub fn color_majors(&self, color: Color) -> Bitboard {
        self.colors(color) & self.majors()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn diag_sliders(&self) -> Bitboard {
        self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)
    }

    #[inline]
    pub fn color_diag_sliders(&self, color: Color) -> Bitboard {
        self.colors(color) & self.diag_sliders()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn orth_sliders(&self) -> Bitboard {
        self.majors()
    }

    #[inline]
    pub fn color_orth_sliders(&self, color: Color) -> Bitboard {
        self.colors(color) & self.orth_sliders()
    }
}

impl Default for Byteboard {
    #[inline]
    fn default() -> Self {
        Byteboard { inner: Vec512::zero() }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Wordboard {
    pub(crate) inner: [Vec512; 2],
}

impl Wordboard {
    #[inline]
    pub const fn new(a: Vec512, b: Vec512) -> Wordboard {
        Wordboard { inner: [a, b] }
    }

    #[inline]
    pub fn into_mailbox(self) -> [PieceMask; Square::COUNT] {
        unsafe { ::core::mem::transmute::<[Vec512; 2], [PieceMask; Square::COUNT]>(self.inner) }
    }

    #[inline]
    pub fn all(&self) -> Bitboard {
        Bitboard(Vec512::interleave64(
            self.inner[0].nonzero16() as u64,
            self.inner[1].nonzero16() as u64
        ))
    }

    #[inline]
    pub fn for_mask(&self, mask: PieceMask) -> Bitboard {
        let mask = Vec512::splat16(mask.into_inner());

        Bitboard(Vec512::interleave64(
            Vec512::test16(self.inner[0], mask) as u64,
            Vec512::test16(self.inner[1], mask) as u64
        ))
    }

    #[inline]
    pub fn get(&self, sq: Square) -> PieceMask {
        let index = sq as u16;
        let (vec, index) = if index < 32 {
            (self.inner[0], index)
        } else {
            (self.inner[1], index - 32)
        };

        PieceMask::new(Vec512::permute16(Vec512::splat16(index), vec).into_u32() as u16)
    }
}

impl Default for Wordboard {
    #[inline]
    fn default() -> Self {
        Wordboard { inner: [Vec512::zero(); 2] }
    }
}


impl BitAnd for Wordboard {
    type Output = Wordboard;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Wordboard {
            inner: [
                self.inner[0] & rhs.inner[0],
                self.inner[1] & rhs.inner[1],
            ]
        }
    }
}