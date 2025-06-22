use crate::{Bitboard, Piece, Rank, Square};

/*----------------------------------------------------------------*/

/*
Bit Layout:
bits 0-5: Source square
bits 6-11: Target square
bits 12-13: Promotion Piece - 2
bits 14: 15: Special Flag: Promotion (1), En Passant (2), Castling (3)
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move { bits: u16 }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MoveFlag {
    Normal,
    Promotion,
    EnPassant,
    Castling,
}

impl MoveFlag {
    #[inline(always)]
    pub const fn index(i: usize) -> MoveFlag {
        match i {
            0 => MoveFlag::Normal,
            1 => MoveFlag::Promotion,
            2 => MoveFlag::EnPassant,
            3 => MoveFlag::Castling,
            _ => panic!("MoveFlag::index(): Index out of bounds")
        }
    }

    #[inline(always)]
    pub const fn try_index(i: usize) -> Option<MoveFlag> {
        match i {
            0 => Some(MoveFlag::Normal),
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

        Move { bits }
    }

    #[inline(always)]
    pub const fn new_promotion(from: Square, to: Square, piece: Piece) -> Move {
        let mut bits = 0;

        bits |= from as u16;
        bits |= (to as u16) << 6;
        bits |= (piece as u16 - Piece::Knight as u16) << 12;
        bits |= (MoveFlag::Promotion as u16) << 14;

        Move { bits }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn from(self) -> Square {
        Square::index((self.bits & 0b111111) as usize)
    }

    #[inline(always)]
    pub const fn to(self) -> Square {
        Square::index(((self.bits >> 6) & 0b111111) as usize)
    }

    #[inline(always)]
    pub const fn promotion(self) -> Option<Piece> {
        match self.flag() {
            MoveFlag::Promotion => Some(Piece::index(Piece::Knight as usize + ((self.bits >> 12) & 0b11) as usize)),
            _ => None
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn flag(self) -> MoveFlag {
        MoveFlag::index(((self.bits << 14) & 0b11) as usize)
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
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct PieceMoves {
    pub piece: Piece,
    pub from: Square,
    pub to: Bitboard,
    en_passant: Option<Square>,
}

impl PieceMoves {
    #[inline(always)]
    pub const fn has(self, mv: Move) -> bool {
        self.from as u8 == mv.from() as u8
            && self.to.has(mv.to())
            && self.is_promotion() == mv.is_promotion()
    }

    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        let rank_matches = !self.to.is_empty()
            && matches!(self.to.next_square().rank(), Rank::First | Rank::Eighth);

        matches!(self.piece, Piece::Pawn) && rank_matches
    }

    #[inline(always)]
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

        if self.moves.is_promotion() {
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

            let flag = match () {
                _ if self.moves.piece == Piece::King && from.dist(to) > 1 => MoveFlag::Castling,
                _ if self.moves.piece == Piece::Pawn && Some(to) == self.moves.en_passant => MoveFlag::EnPassant,
                _ => MoveFlag::Normal
            };

            Some(Move::new(from, to, flag))
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