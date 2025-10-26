use core::{fmt, num::NonZeroU16};
use crate::*;

/*----------------------------------------------------------------*/

/*
Bit Layout:
bits 0-5: Source square
bits 6-11: Target square
bits 12-15: Move Flag
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl MoveFlag {
    #[inline]
    pub const fn promotion(piece: Piece) -> Option<MoveFlag> {
        match piece {
            Piece::Knight => Some(MoveFlag::PromotionKnight),
            Piece::Bishop => Some(MoveFlag::PromotionBishop),
            Piece::Rook => Some(MoveFlag::PromotionRook),
            Piece::Queen => Some(MoveFlag::PromotionQueen),
            _ => None,
        }
    }

    #[inline]
    pub const fn capture_promotion(piece: Piece) -> Option<MoveFlag> {
        match piece {
            Piece::Knight => Some(MoveFlag::CapturePromotionKnight),
            Piece::Bishop => Some(MoveFlag::CapturePromotionBishop),
            Piece::Rook => Some(MoveFlag::CapturePromotionRook),
            Piece::Queen => Some(MoveFlag::CapturePromotionQueen),
            _ => None,
        }
    }
}

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
    pub const fn from_bits(bits: u16) -> Move {
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
        if !self.is_promotion() {
            return None;
        }

        const PIECE_LOOKUP: [Piece; 4] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight];

        Some(PIECE_LOOKUP[((self.bits.get() & 0x3000) >> 12) as usize])
    }

    #[inline]
    pub fn is_castling(self) -> bool {
        let flag = self.flag();

        flag == MoveFlag::ShortCastling || flag == MoveFlag::LongCastling
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
    pub const fn is_capture_promotion(self) -> bool {
        (self.bits.get() & 0xC000) == 0xC000
    }

    #[inline]
    pub const fn is_tactic(self) -> bool {
        (self.bits.get() & 0xC000) != 0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn display(self, board: &Board, frc: bool) -> Move {
        if frc {
            return self;
        }

        let (from, mut to, flag) = (self.from(), self.to(), self.flag());

        let stm = board.stm();
        let our_backrank = Rank::First.relative_to(stm);
        let castle_src = Square::new(File::E, our_backrank);

        if from == castle_src && from == board.king(stm) {
            let rights = board.castle_rights(stm);

            if Some(to) == rights.short.map(|f| Square::new(f, our_backrank)) {
                to = Square::new(File::G, our_backrank);
            } else if Some(to) == rights.long.map(|f| Square::new(f, our_backrank)) {
                to = Square::new(File::C, our_backrank);
            }
        }

        Move::new(from, to, flag)
    }

    #[inline]
    pub fn parse(board: &Board, frc: bool, mv: &str) -> Option<Move> {
        let from = mv.get(0..2)?.parse::<Square>().ok()?;
        let mut to = mv.get(2..4)?.parse::<Square>().ok()?;
        let promotion = if let Some(s) = mv.get(4..5) {
            let piece = s.parse::<Piece>().ok()?;

            Some(piece).filter(|p|
                matches!(p, Piece::Knight | Piece::Bishop | Piece::Rook | Piece::Queen)
            )
        } else {
            None
        };

        let is_capture = board.piece_on(to).is_some();
        let flag = match board.piece_on(from)? {
            Piece::Pawn => Move::parse_pawn_flag(board, from, to, is_capture, promotion)?,
            Piece::King => Move::parse_king_flag(board, frc, from, &mut to, is_capture),
            _ if is_capture => MoveFlag::Capture,
            _ => MoveFlag::Normal,
        };

        Some(Move::new(from, to, flag))
    }

    #[inline]
    fn parse_pawn_flag(
        board: &Board,
        from: Square,
        to: Square,
        is_capture: bool,
        promotion: Option<Piece>
    ) -> Option<MoveFlag> {
        if let Some(promotion) = promotion {
            if is_capture {
                MoveFlag::capture_promotion(promotion)
            } else {
                MoveFlag::promotion(promotion)
            }
        } else if is_capture {
            Some(MoveFlag::Capture)
        } else if let Some(ep) = board.ep_square() && to == ep {
            Some(MoveFlag::EnPassant)
        } else if from.rank() == Rank::Second.relative_to(board.stm())
            && to.rank() == Rank::Fourth.relative_to(board.stm()) {
            Some(MoveFlag::DoublePush)
        } else {
            Some(MoveFlag::Normal)
        }
    }

    #[inline]
    fn parse_king_flag(
        board: &Board,
        chess960: bool,
        from: Square,
        to: &mut Square,
        is_capture: bool,
    ) -> MoveFlag {
        let stm = board.stm();
        if chess960 && is_capture {
            let rights = board.castle_rights(stm);
            let our_backrank = Rank::First.relative_to(stm);

            return if Some(*to) == rights.short.map(|f| Square::new(f, our_backrank)) {
                MoveFlag::ShortCastling
            } else if Some(*to) == rights.long.map(|f| Square::new(f, our_backrank)) {
                MoveFlag::LongCastling
            } else {
                MoveFlag::Capture
            }
        }

        if is_capture {
            return MoveFlag::Capture;
        }

        let our_backrank = Rank::First.relative_to(stm);
        let castle_src = Square::new(File::E, our_backrank);

        if from == castle_src {
            let rights = board.castle_rights(stm);
            let castle_short = Square::new(File::G, our_backrank);
            let castle_long = Square::new(File::C, our_backrank);

            if let Some(rook) = rights.short && *to == castle_short {
                *to = Square::new(rook, our_backrank);
                return MoveFlag::ShortCastling;
            } else if let Some(rook) = rights.long && *to == castle_long {
                *to = Square::new(rook, our_backrank);
                return MoveFlag::LongCastling;
            }
        }

        MoveFlag::Normal
    }
}

impl fmt::Display for Move {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.from(), self.to())?;

        if let Some(piece) = self.promotion() {
            write!(f, "{}", piece)?;
        }

        Ok(())
    }
}