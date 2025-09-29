use std::num::NonZeroU16;
use crate::*;

/*----------------------------------------------------------------*/

/*
Bit Layout:
bits 0-5: Source square
bits 6-11: Target square
bits 12-15: Move Flag
*/
#[derive(Debug, Copy, Clone)]
pub struct Move { bits: NonZeroU16 }

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveFlag {
    Normal = 0x0000,
    DoublePush = 0x1000,
    LongCastling  = 0x2000,
    ShortCastling = 0x3000,
    PromotionQueen  = 0x4000,
    PromotionRook   = 0x5000,
    PromotionBishop = 0x6000,
    PromotionKnight = 0x7000,
    Capture = 0x8000,
    EnPassant = 0x9000,
    CapturePromotionQueen  = 0xC000,
    CapturePromotionRook   = 0xD000,
    CapturePromotionBishop = 0xE000,
    CapturePromotionKnight = 0xF000,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MoveParseError;

impl Move {
    #[inline]
    pub const fn new(from: Square, to: Square, flag: MoveFlag) -> Move {
        let mut bits = 0;

        bits |= from as u16;
        bits |= (to as u16) << 6;
        bits |= flag as u16;

        Move { bits: NonZeroU16::new(bits).unwrap() }
    }

    #[inline]
    pub fn from_bits(bits: u16) -> Move {
        assert_ne!(bits, 0);

        Move { bits: NonZeroU16::new(bits).unwrap() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn bits(self) -> u16 {
        self.bits.get()
    }

    #[inline]
    pub const fn from(self) -> Square {
        Square::index((self.bits.get() & 0b111111) as usize)
    }

    #[inline]
    pub const fn to(self) -> Square {
        Square::index(((self.bits.get() >> 6) & 0b111111) as usize)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn flag(self) -> MoveFlag {
        unsafe { std::mem::transmute::<u16, MoveFlag>(self.bits.get() & 0xF000) }
    }

    #[inline]
    pub const fn promotion(self) -> Option<Piece> {
        if self.is_promotion() {
            return None;
        }

        const PIECE_LOOKUP: [Piece; 4] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight];

        Some(PIECE_LOOKUP[((self.bits.get() & 0x3000) >> 12) as usize])
    }

    #[inline]
    pub fn is_castling(self) -> bool {
        let flag = self.flag();

        flag == MoveFlag::ShortCastling || flag == MoveFlag::ShortCastling
    }

    #[inline]
    pub const fn is_capture(self) -> bool {
        (self.bits.get() & 0x8000) != 0
    }

    #[inline]
    pub fn is_en_passant(self) -> bool {
        self.flag() == MoveFlag::EnPassant
    }

    #[inline]
    pub const fn is_promotion(self) -> bool {
        (self.bits.get() & 0x4000) != 0
    }

    #[inline]
    pub const fn is_tactic(self) -> bool {
        (self.bits.get() & 0xC000) != 0
    }
}