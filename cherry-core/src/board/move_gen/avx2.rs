use core::ops::{Deref, DerefMut};
use arrayvec::ArrayVec;

use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct MoveList(ArrayVec<Move, 256>);

impl MoveList {
    #[inline]
    pub fn empty() -> Self {
        MoveList(ArrayVec::new())
    }

    #[inline]
    fn write<const N: usize>(
        &mut self,
        index_sq: &IndexToSquare,
        attacks: &Wordboard,
        mask: PieceMask,
        flags: [MoveFlag; N],
        dest: Bitboard,
    ) {
        for dest in dest {
            let mask = attacks.get(dest) & mask;
            for index in mask {
                let src = index_sq[index].unwrap();
                for flag in flags {
                    self.push(Move::new(src, dest, flag));
                }
            }
        }
    }
}

impl Deref for MoveList {
    type Target = ArrayVec<Move, 256>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/*----------------------------------------------------------------*/

impl Board {
    #[inline]
    pub fn gen_tactics(&self) -> MoveList {
        let mut moves = MoveList::empty();
        let checkers = self.checkers();

        match checkers.popcnt() {
            0 => self.gen_no_check::<true>(&mut moves),
            1 => self.gen_check::<true>(&mut moves, checkers),
            2 => self.gen_double_check::<true>(&mut moves, checkers),
            _ => unreachable!(),
        }

        moves
    }

    #[inline]
    pub fn gen_quiets(&self) -> MoveList {
        let mut moves = MoveList::empty();
        let checkers = self.checkers();

        match checkers.popcnt() {
            0 => self.gen_no_check::<false>(&mut moves),
            1 => self.gen_check::<false>(&mut moves, checkers),
            2 => self.gen_double_check::<false>(&mut moves, checkers),
            _ => unreachable!(),
        }

        moves
    }

    #[inline]
    pub fn gen_moves(&self) -> MoveList {
        let mut moves = MoveList::empty();
        let checkers = self.checkers();

        match checkers.popcnt() {
            0 => {
                self.gen_no_check::<true>(&mut moves);
                self.gen_no_check::<false>(&mut moves);
            }
            1 => {
                self.gen_check::<true>(&mut moves, checkers);
                self.gen_check::<false>(&mut moves, checkers);
            }
            2 => {
                self.gen_double_check::<true>(&mut moves, checkers);
                self.gen_double_check::<false>(&mut moves, checkers);
            }
            _ => unreachable!(),
        }

        moves
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn gen_moves_to<const KING_MOVES: bool, const TACTICS: bool>(
        &self,
        moves: &mut MoveList,
        valid: Bitboard,
    ) {
        let masked_attacks = Wordboard(self.attack_table(self.stm).0 & self.pinned_mask.0);

        let valid_pieces = self.index_to_piece[self.stm].valid();
        let pawn_mask = self.index_to_piece[self.stm].mask_eq(Piece::Pawn);
        let non_pawn_mask = valid_pieces & !pawn_mask & !PieceMask::KING;

        let empty = self.empty();
        let their_pieces = self.colors(!self.stm);
        let their_attacks = self.attack_table(!self.stm).all();
        let their_backrank = Rank::Eighth.relative_to(self.stm).bitboard();

        let pawn_dest = valid & masked_attacks.for_mask(pawn_mask);
        let non_pawn_dest = valid & masked_attacks.for_mask(non_pawn_mask);
        let king_dest = valid & masked_attacks.for_mask(PieceMask::KING) & !their_attacks;

        let index_sq = &self.index_to_square[self.stm];
        if TACTICS {
            moves.write(
                index_sq,
                &masked_attacks,
                pawn_mask,
                [
                    MoveFlag::CapturePromotionQueen,
                    MoveFlag::CapturePromotionRook,
                    MoveFlag::CapturePromotionBishop,
                    MoveFlag::CapturePromotionKnight
                ],
                pawn_dest & their_pieces & their_backrank
            );

            moves.write(
                index_sq,
                &masked_attacks,
                pawn_mask,
                [MoveFlag::Capture],
                pawn_dest & their_pieces & !their_backrank
            );

            if let Some(ep_sq) = self.ep_square() {
                let mask = masked_attacks.get(ep_sq) & pawn_mask;
                let ep_info = self.en_passant.unwrap();

                for index in mask {
                    let src = index_sq[index].unwrap();
                    let left = src.file() < ep_sq.file();

                    if (left && ep_info.left()) || (!left && ep_info.right()) {
                        moves.push(Move::new(src, ep_sq, MoveFlag::EnPassant));
                    }
                }
            }

            moves.write(
                index_sq,
                &masked_attacks,
                non_pawn_mask,
                [MoveFlag::Capture],
                non_pawn_dest & their_pieces
            );

            if KING_MOVES {
                moves.write(
                    index_sq,
                    &masked_attacks,
                    PieceMask::KING,
                    [MoveFlag::Capture],
                    king_dest & their_pieces
                );
            }
        }

        let pinned_pawn_mask = self.pinned & !self.king(self.stm).file().bitboard();
        let valid_pawns = self.color_pieces(self.stm, Piece::Pawn) & !pinned_pawn_mask;
        let (normal_empty, double_empty) = pawn_empty(self.stm, empty, valid);

        let pawn_normal = valid_pawns & normal_empty;
        let pawn_double = valid_pawns & double_empty;

        if TACTICS {
            let mut promo_mask = (pawn_normal >> PAWN_PROMO_SHIFT[self.stm]).0 as u8;
            while promo_mask != 0 {
                let index = promo_mask.trailing_zeros() as usize * 4;
                moves.push(PAWN_PROMOS[self.stm][index + 0]);
                moves.push(PAWN_PROMOS[self.stm][index + 1]);
                moves.push(PAWN_PROMOS[self.stm][index + 2]);
                moves.push(PAWN_PROMOS[self.stm][index + 3]);

                promo_mask &= promo_mask.wrapping_sub(1);
            }
        } else {
            let normal_mask = (pawn_normal >> PAWN_DOUBLE_SHIFT[self.stm]).0 as u8;
            let double_mask = (pawn_double >> PAWN_DOUBLE_SHIFT[self.stm]).0 as u8;
            let mut double_mask = double_mask as u16 | ((normal_mask as u16) << 8);

            while double_mask != 0 {
                moves.push(PAWN_DOUBLE[self.stm][double_mask.trailing_zeros() as usize]);
                double_mask &= double_mask.wrapping_sub(1);
            }

            let mut normal_mask = (pawn_normal >> PAWN_NORMAL_SHIFT).0 as u32;
            while normal_mask != 0 {
                moves.push(PAWN_NORMAL[self.stm][normal_mask.trailing_zeros() as usize]);
                normal_mask &= normal_mask.wrapping_sub(1);
            }

            moves.write(
                index_sq,
                &masked_attacks,
                non_pawn_mask,
                [MoveFlag::Normal],
                non_pawn_dest & empty,
            );

            if KING_MOVES {
                moves.write(
                    index_sq,
                    &masked_attacks,
                    PieceMask::KING,
                    [MoveFlag::Normal],
                    king_dest & empty,
                );

                let blockers = self.occupied();
                let our_backrank = Rank::First.relative_to(self.stm);
                let rights = self.castle_rights(self.stm);
                let king_src = self.king(self.stm);

                macro_rules! write_castling_moves {
                    ($king_dest:expr, $rook_dest:expr, $rights:expr, $flag:expr) => {
                        if let Some(rook_src) = $rights.map(|f| Square::new(f, our_backrank)) {
                            let king_dest = Square::new($king_dest, our_backrank);
                            let rook_dest = Square::new($rook_dest, our_backrank);
                            let king_to_rook = between(king_src, rook_src);
                            let king_to_dest = between(king_src, king_dest);
                            let must_be_safe = king_to_dest | king_dest;
                            let must_be_empty = must_be_safe | king_to_rook | rook_dest;
                            let blockers = blockers ^ king_src ^ rook_src;

                            if !self.pinned.has(rook_src)
                                && blockers.is_disjoint(must_be_empty)
                                && their_attacks.is_disjoint(must_be_safe)
                            {
                                moves.push(Move::new(king_src, rook_src, $flag));
                            }
                        }
                    };
                }

                write_castling_moves!(File::G, File::F, rights.short, MoveFlag::ShortCastling);
                write_castling_moves!(File::C, File::D, rights.long, MoveFlag::LongCastling);
            }
        }
    }

    #[inline]
    fn gen_king_moves<const CHECKERS: usize, const TACTICS: bool>(
        &self,
        moves: &mut MoveList,
        checkers: PieceMask,
    ) {
        let (our_attacks, their_attacks) =
            (self.attack_table(self.stm), self.attack_table(!self.stm));
        let (our_pieces, their_pieces) = (self.colors(self.stm), self.colors(!self.stm));
        let our_king = self.king(self.stm);

        let mut valid = our_attacks.for_mask(PieceMask::KING) & !their_attacks.all() & !our_pieces;
        for checker in checkers.into_iter().take(CHECKERS) {
            let checker_piece = self.index_to_piece[!self.stm][checker].unwrap();
            let checker_sq = self.index_to_square[!self.stm][checker].unwrap();

            if checker_piece.is_slider() {
                valid &= !line(checker_sq, our_king);
            }
        }

        let flag = if TACTICS {
            valid &= their_pieces;
            MoveFlag::Capture
        } else {
            valid &= !their_pieces;
            MoveFlag::Normal
        };

        for dest in valid {
            moves.push(Move::new(our_king, dest, flag));
        }
    }

    #[inline]
    fn gen_no_check<const TACTICS: bool>(&self, moves: &mut MoveList) {
        self.gen_moves_to::<true, TACTICS>(moves, Bitboard::FULL)
    }

    #[inline]
    fn gen_check<const TACTICS: bool>(&self, moves: &mut MoveList, checkers: PieceMask) {
        let king = self.king(self.stm);
        let checker = checkers.next().unwrap();
        let checker_piece = self.index_to_piece[!self.stm][checker].unwrap();
        let checker_sq = self.index_to_square[!self.stm][checker].unwrap();
        let valid = if checker_piece == Piece::Knight {
            checker_sq.bitboard()
        } else {
            between(king, checker_sq) | checker_sq
        };

        self.gen_moves_to::<false, TACTICS>(moves, valid);
        self.gen_king_moves::<1, TACTICS>(moves, checkers);
    }

    #[inline]
    fn gen_double_check<const TACTICS: bool>(&self, moves: &mut MoveList, checkers: PieceMask) {
        self.gen_king_moves::<2, TACTICS>(moves, checkers);
    }
}

/*----------------------------------------------------------------*/

#[inline]
fn pawn_empty(stm: Color, empty: Bitboard, valid: Bitboard) -> (Bitboard, Bitboard) {
    let valid_empty = valid & empty;

    match stm {
        Color::White => (
            valid_empty.shift::<South>(1),
            empty.shift::<South>(1) & valid_empty.shift::<South>(2),
        ),
        Color::Black => (
            valid_empty.shift::<North>(1),
            empty.shift::<North>(1) & valid_empty.shift::<North>(2),
        ),
    }
}

const PAWN_NORMAL_SHIFT: usize = 16;
static PAWN_NORMAL: [[Move; 32]; Color::COUNT] = {
    let mut table = [[Move::from_bits(1); 32]; Color::COUNT];
    let mut i = 0;
    while i < 32 {
        let src = Square::index(i + PAWN_NORMAL_SHIFT);

        table[0][i] = Move::new(src, src.offset(0, 1), MoveFlag::Normal);
        table[1][i] = Move::new(src, src.offset(0, -1), MoveFlag::Normal);
        i += 1;
    }

    table
};

const PAWN_DOUBLE_SHIFT: [usize; Color::COUNT] = [8, 48];
static PAWN_DOUBLE: [[Move; 16]; Color::COUNT] = {
    let mut table = [[Move::from_bits(1); 16]; Color::COUNT];
    let mut i = 0;
    while i < 8 {
        let white_src = Square::index(i + PAWN_DOUBLE_SHIFT[0]);
        let black_src = Square::index(i + PAWN_DOUBLE_SHIFT[1]);

        table[0][i] = Move::new(white_src, white_src.offset(0, 2), MoveFlag::DoublePush);
        table[0][i + 8] = Move::new(white_src, white_src.offset(0, 1), MoveFlag::Normal);
        table[1][i] = Move::new(black_src, black_src.offset(0, -2), MoveFlag::DoublePush);
        table[1][i + 8] = Move::new(black_src, black_src.offset(0, -1), MoveFlag::Normal);
        i += 1;
    }

    table
};

const PAWN_PROMO_SHIFT: [usize; Color::COUNT] = [48, 8];
static PAWN_PROMOS: [[Move; 32]; Color::COUNT] = {
    let mut table = [[Move::from_bits(1); 32]; Color::COUNT];
    let mut i = 0;
    while i < 8 {
        let white_src = Square::index(i + PAWN_PROMO_SHIFT[0]);
        let black_src = Square::index(i + PAWN_PROMO_SHIFT[1]);
        let white_dest = white_src.offset(0, 1);
        let black_dest = black_src.offset(0, -1);

        table[0][i * 4 + 0] = Move::new(white_src, white_dest, MoveFlag::PromotionQueen);
        table[0][i * 4 + 1] = Move::new(white_src, white_dest, MoveFlag::PromotionRook);
        table[0][i * 4 + 2] = Move::new(white_src, white_dest, MoveFlag::PromotionBishop);
        table[0][i * 4 + 3] = Move::new(white_src, white_dest, MoveFlag::PromotionKnight);
        table[1][i * 4 + 0] = Move::new(black_src, black_dest, MoveFlag::PromotionQueen);
        table[1][i * 4 + 1] = Move::new(black_src, black_dest, MoveFlag::PromotionRook);
        table[1][i * 4 + 2] = Move::new(black_src, black_dest, MoveFlag::PromotionBishop);
        table[1][i * 4 + 3] = Move::new(black_src, black_dest, MoveFlag::PromotionKnight);

        i += 1;
    }

    table
};
