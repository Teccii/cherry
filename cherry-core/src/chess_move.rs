use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU16;
use std::str::FromStr;
use crate::*;
/*----------------------------------------------------------------*/

/*
Bit Layout:
bits 0-5: Source square
bits 6-11: Target square
bits 12-13: Promotion Piece - 2
bits 14-15: Special Flag: None (0), Promotion (1), En Passant (2), Castling (3)
*/
#[derive(Debug, Copy, Clone)]
pub struct Move { bits: NonZeroU16 }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MoveFlag {
    None,
    Promotion,
    EnPassant,
    Castling,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MoveParseError;

impl MoveFlag {
    #[inline(always)]
    pub const fn index(i: usize) -> MoveFlag {
        match i {
            0 => MoveFlag::None,
            1 => MoveFlag::Promotion,
            2 => MoveFlag::EnPassant,
            3 => MoveFlag::Castling,
            _ => panic!("MoveFlag::index(): Index out of bounds")
        }
    }

    #[inline(always)]
    pub const fn try_index(i: usize) -> Option<MoveFlag> {
        match i {
            0 => Some(MoveFlag::None),
            1 => Some(MoveFlag::Promotion),
            2 => Some(MoveFlag::EnPassant),
            3 => Some(MoveFlag::Castling),
            _ => None
        }
    }
}

impl Move {
    #[inline(always)]
    pub const fn new(from: Square, to: Square, flag: MoveFlag) -> Move {
        let mut bits = 0;

        bits |= from as u16;
        bits |= (to as u16) << 6;
        bits |= (flag as u16) << 14;

        Move { bits: NonZeroU16::new(bits).unwrap() }
    }

    #[inline(always)]
    pub const fn new_promotion(from: Square, to: Square, piece: Piece) -> Move {
        let mut bits = 0;

        bits |= from as u16;
        bits |= (to as u16) << 6;
        bits |= (piece as u16 - Piece::Knight as u16) << 12;
        bits |= (MoveFlag::Promotion as u16) << 14;

        Move { bits: NonZeroU16::new(bits).unwrap() }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn bits(self) -> u16 { self.bits.get() }

    #[inline(always)]
    pub const fn from(self) -> Square {
        Square::index((self.bits.get() & 0b111111) as usize)
    }

    #[inline(always)]
    pub const fn to(self) -> Square {
        Square::index(((self.bits.get() >> 6) & 0b111111) as usize)
    }

    #[inline(always)]
    pub const fn promotion(self) -> Option<Piece> {
        match self.flag() {
            MoveFlag::Promotion => Some(Piece::index(Piece::Knight as usize + ((self.bits.get() >> 12) & 0b11) as usize)),
            _ => None
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn set_src(&mut self, sq: Square) {
        let no_from = self.bits.get() & !0b111111;

        self.bits = NonZeroU16::new(no_from | sq as u16).unwrap();
    }

    #[inline(always)]
    pub const fn set_dest(&mut self, sq: Square) {
        let no_to = self.bits.get() & !0b111111000000;

        self.bits = NonZeroU16::new(no_to | ((sq as u16) << 6)).unwrap();
    }

    #[inline(always)]
    pub const fn set_promotion(&mut self, promotion: Option<Piece>) {
        let no_promotion = self.bits.get() & !0b11000000000000;
        let promotion = if let Some(p) = promotion {
            (p as u16 - Piece::Knight as u16) << 12
        } else {
            0
        };

        self.bits = NonZeroU16::new(no_promotion | promotion).unwrap();
    }

    #[inline(always)]
    pub const fn set_flag(&mut self, flag: MoveFlag) {
        let no_flag = self.bits.get() & !0b1100000000000000;

        self.bits = NonZeroU16::new(no_flag | ((flag as u16) << 14)).unwrap();
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn flag(self) -> MoveFlag {
        MoveFlag::index(((self.bits.get() >> 14) & 0b11) as usize)
    }

    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        self.flag() as u8 == MoveFlag::Promotion as u8
    }

    #[inline(always)]
    pub const fn is_en_passant(self) -> bool {
        self.flag() as u8 == MoveFlag::EnPassant as u8
    }

    #[inline(always)]
    pub const fn is_castling(self) -> bool {
        self.flag() as u8 == MoveFlag::Castling as u8
    }

    /*----------------------------------------------------------------*/

    pub fn parse(board: &Board, chess960: bool, mv: &str) -> Result<Move, MoveParseError> {
        let mut mv = mv.parse::<Move>()?;
        if chess960 {
            return Ok(mv);
        }

        let back_rank = Rank::First.relative_to(board.stm());
        let castle_src = Square::new(File::E, back_rank);
        let castle_short = Square::new(File::G, back_rank);
        let castle_long = Square::new(File::C, back_rank);
        let from = mv.from();

        if board.king(board.stm()) == from && from == castle_src {
            let rights = board.castle_rights(board.stm());
            let to = mv.to();

            if let Some(rook) = rights.short && to == castle_short {
                mv.set_dest(Square::new(rook, back_rank));
            } else if let Some(rook) = rights.long && to == castle_long {
                mv.set_dest(Square::new(rook, back_rank));
            }
        }

        Ok(mv)
    }
    
    pub fn display(self, board: &Board, chess960: bool) -> impl fmt::Display {
        if chess960 {
            return self;
        }

        let mut mv = self;
        let back_rank = Rank::First.relative_to(board.stm());
        let rights = board.castle_rights(board.stm());
        let castle_short = rights.short.map(|f| Square::new(f, back_rank));
        let castle_long = rights.long.map(|f| Square::new(f, back_rank));

        if board.king(board.stm()) == mv.from() {
            let to = mv.to();

            if Some(to) == castle_short {
                mv.set_dest(Square::new(File::G, back_rank));
            } else if Some(to) == castle_long {
                mv.set_dest(Square::new(File::C, back_rank))
            }
        }

        mv
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.from() == other.from()
        && self.to() == other.to()
        && self.promotion() == other.promotion()
    }
}

impl Eq for Move { }

impl Hash for Move {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from().hash(state);
        self.to().hash(state);
        self.promotion().hash(state);
    }
}

impl FromStr for Move {
    type Err = MoveParseError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse(s: &str) -> Option<Move> {
            let from = s.get(0..2)?.parse::<Square>().ok()?;
            let to = s.get(2..4)?.parse::<Square>().ok()?;
            let promotion = if let Some(s) = s.get(4..5) {
                let piece = s.parse::<Piece>().ok()?;
                
                Some(piece).filter(|p|
                    matches!(p, Piece::Knight | Piece::Bishop | Piece::Rook | Piece::Queen)
                )
            } else {
                None
            };

            Some(if let Some(promotion) = promotion {
                Move::new_promotion(from, to, promotion)
            } else {
                Move::new(from, to, MoveFlag::None)
            })
        }

        parse(s).ok_or(MoveParseError)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.from(), self.to())?;

        if let Some(piece) = self.promotion() {
            write!(f, "{}", piece)?;
        }

        Ok(())
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct PieceMoves {
    pub piece: Piece,
    pub from: Square,
    pub to: Bitboard,
    pub flag: MoveFlag,
}

impl PieceMoves {
    pub fn has(self, mv: Move) -> bool {
        let is_promotion = self.piece == Piece::Pawn
            && matches!(mv.to().rank(), Rank::First | Rank::Eighth);

        self.from == mv.from()
            && self.to.has(mv.to())
            && is_promotion == mv.is_promotion()

    }

    pub const fn len(self) -> usize {
        if matches!(self.piece, Piece::Pawn) {
            const PROMOTION_MASK: u64 = 0xFF000000000000FF;

            Bitboard(self.to.0 & !PROMOTION_MASK).popcnt()
                + Bitboard(self.to.0 & PROMOTION_MASK).popcnt() * 4
        } else {
            self.to.popcnt()
        }
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.to.is_empty()
    }
}

impl IntoIterator for PieceMoves {
    type Item = Move;
    type IntoIter = PieceMovesIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        PieceMovesIter {
            moves: self,
            promotion: 0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PieceMovesIter {
    pub moves: PieceMoves,
    pub promotion: usize
}

impl Iterator for PieceMovesIter {
    type Item = Move;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let from = self.moves.from;
        let to = self.moves.to.try_next_square()?;
        let is_promotion = self.moves.piece == Piece::Pawn
            && matches!(to.rank(), Rank::First | Rank::Eighth);

        if is_promotion {
            let piece = match self.promotion {
                0 => Piece::Queen,
                1 => Piece::Rook,
                2 => Piece::Bishop,
                3 => Piece::Knight,
                _ => unreachable!()
            };

            self.promotion += 1;

            if self.promotion > 3 {
                self.moves.to ^= to.bitboard();
                self.promotion = 0;
            }

            Some(Move::new_promotion(from, to, piece))
        } else {
            self.moves.to ^= to.bitboard();

            Some(Move::new(from, to, self.moves.flag))
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }
}

impl ExactSizeIterator for PieceMovesIter {
    #[inline(always)]
    fn len(&self) -> usize {
        self.moves.len() - self.promotion
    }
}