use core::ops::{Deref, DerefMut};
use std::ptr;

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

    /*----------------------------------------------------------------*/

    #[inline]
    fn write(
        &mut self,
        attacks: &Wordboard,
        mask: PieceMask,
        flag: MoveFlag,
        dest: Bitboard,
        src: u16x16,
    ) {
        for sq in dest {
            let mask = attacks.get(sq) & mask;
            let dest = ((sq as u16) << 6) | (flag as u16);

            self.write16(src | u16x16::splat(dest), Mask16x16::from(mask.0));
        }
    }

    #[inline]
    fn write_capt_promos(
        &mut self,
        board: &Board,
        attacks: &Wordboard,
        mask: PieceMask,
        dest: Bitboard,
    ) {
        for sq in dest {
            let mask = attacks.get(sq) & mask;

            for index in mask {
                let src = board.index_to_square[board.stm][index].unwrap();
                let base_move = src as u16 | ((sq as u16) << 6);
                let flags = MoveFlag::CapturePromotionQueen as u64
                    | ((MoveFlag::CapturePromotionRook as u64) << 16)
                    | ((MoveFlag::CapturePromotionBishop as u64) << 32)
                    | ((MoveFlag::CapturePromotionKnight as u64) << 48);

                self.write4(flags + base_move as u64 * 0x0001000100010001u64);
            }
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn write4(&mut self, moves: u64) {
        let len = self.0.len();
        unsafe {
            let ptr = self.0.as_mut_ptr().add(len);
            ptr::copy_nonoverlapping((&moves as *const u64).cast(), ptr, 4);
            self.0.set_len(len + 4);
        }
    }

    #[inline]
    fn write4x8(&mut self, moves: u64x8, mask: Mask64x8) {
        let len = self.0.len();
        unsafe {
            let ptr = self.0.as_mut_ptr().add(len);
            let new_len = len + 4 * mask.to_bitmask().count_ones() as usize;

            moves.compress_store(mask, ptr);
            self.0.set_len(new_len);
        }
    }

    #[inline]
    fn write16(&mut self, moves: u16x16, mask: Mask16x16) {
        let len = self.0.len();
        unsafe {
            let ptr = self.0.as_mut_ptr().add(len);
            let new_len = len + mask.to_bitmask().count_ones() as usize;

            moves.compress_store(mask, ptr);
            self.0.set_len(new_len);
        }
    }

    /*
    SAFETY:
    Since we're doing `_mm512_storeu_si512(ptr.cast(), _mm512_maskz_compress_epi16(mask, moves)` instead of
    `_mm512_mask_compressstoreu_epi16(ptr.cast(), mask, moves)` which is slow on zen4 for whatever reason,
    this does a full vector width write instead of a write with just the size of the compressed elements.
    This doesn't write outside the list because the list has a size of u16x256, and since the maximum number
    of legal moves is 218, even if we were to write u16x32 at move number 217, we would not write anything
    outside the list, because 217+32=249 < 256.
    */
    #[inline]
    fn write32(&mut self, moves: u16x32, mask: Mask16x32) {
        let len = self.0.len();
        unsafe {
            let ptr = self.0.as_mut_ptr().add(len);
            let new_len = len + mask.to_bitmask().count_ones() as usize;

            moves.compress_store(mask, ptr);
            self.0.set_len(new_len);
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

        let src = unsafe { u8x16::load(self.index_to_square[self.stm].0.as_ptr()) }.zero_ext();

        if TACTICS {
            moves.write_capt_promos(
                self,
                &masked_attacks,
                pawn_mask,
                pawn_dest & their_pieces & their_backrank,
            );
            moves.write(
                &masked_attacks,
                pawn_mask,
                MoveFlag::Capture,
                pawn_dest & their_pieces & !their_backrank,
                src,
            );

            if let Some(ep_sq) = self.ep_square() {
                let mask = masked_attacks.get(ep_sq) & pawn_mask;
                let ep_info = self.en_passant.unwrap();

                for index in mask {
                    let src = self.index_to_square[self.stm][index].unwrap();
                    let left = src.file() < ep_sq.file();

                    if (left && ep_info.left()) || (!left && ep_info.right()) {
                        moves.push(Move::new(src, ep_sq, MoveFlag::EnPassant));
                    }
                }
            }

            moves.write(
                &masked_attacks,
                non_pawn_mask,
                MoveFlag::Capture,
                non_pawn_dest & their_pieces,
                src,
            );

            if KING_MOVES {
                moves.write(
                    &masked_attacks,
                    PieceMask::KING,
                    MoveFlag::Capture,
                    king_dest & their_pieces,
                    src,
                );
            }
        }

        let pinned_pawn_mask = self.pinned & !self.king(self.stm).file().bitboard();
        let valid_pawns = self.color_pieces(self.stm, Piece::Pawn) & !pinned_pawn_mask;
        let (normal_empty, double_empty) = pawn_empty(self.stm, empty, valid);

        let pawn_normal = valid_pawns & normal_empty;
        let pawn_double = valid_pawns & double_empty;

        if TACTICS {
            let promo_mask = Mask64x8::from((pawn_normal >> PAWN_PROMO_SHIFT[self.stm]).0 as u8);
            moves.write4x8(
                unsafe { u64x8::load(PAWN_PROMOS[self.stm].as_ptr()) },
                promo_mask,
            );
        } else {
            let normal_mask = (pawn_normal >> PAWN_DOUBLE_SHIFT[self.stm]).0 as u8;
            let double_mask = (pawn_double >> PAWN_DOUBLE_SHIFT[self.stm]).0 as u8;
            let double_mask = Mask16x16::from(double_mask as u16 | ((normal_mask as u16) << 8));
            moves.write16(
                unsafe { u16x16::load(PAWN_DOUBLE[self.stm].as_ptr()) },
                double_mask,
            );

            let normal_mask = Mask16x32::from((pawn_normal >> PAWN_NORMAL_SHIFT).0 as u32);
            moves.write32(
                unsafe { u16x32::load(PAWN_NORMAL[self.stm].as_ptr()) },
                normal_mask,
            );

            moves.write(
                &masked_attacks,
                non_pawn_mask,
                MoveFlag::Normal,
                non_pawn_dest & empty,
                src,
            );

            if KING_MOVES {
                moves.write(
                    &masked_attacks,
                    PieceMask::KING,
                    MoveFlag::Normal,
                    king_dest & empty,
                    src,
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
