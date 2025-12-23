use core::ops::{Deref, DerefMut};
use std::ptr;
use arrayvec::ArrayVec;

use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct MoveList(ArrayVec<Move, 218>);

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

            self.write16(src | u16x16::splat(dest), Mask16(mask.0));
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
    fn write4x8(&mut self, moves: u64x8, mask: Mask8) {
        let len = self.0.len();
        unsafe {
            let ptr = self.0.as_mut_ptr().add(len);
            let new_len = len + 4 * mask.to_bitmask().count_ones() as usize;

            moves.compress_store(mask, ptr);
            self.0.set_len(new_len);
        }
    }

    #[inline]
    fn write16(&mut self, moves: u16x16, mask: Mask16) {
        let len = self.0.len();
        unsafe {
            let ptr = self.0.as_mut_ptr().add(len);
            let new_len = len + mask.to_bitmask().count_ones() as usize;

            moves.compress_store(mask, ptr);
            self.0.set_len(new_len);
        }
    }

    #[inline]
    fn write32(&mut self, moves: u16x32, mask: Mask32) {
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
    type Target = ArrayVec<Move, 218>;

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
    pub fn gen_moves(&self) -> MoveList {
        let mut moves = MoveList::empty();
        let checkers = self.checkers();

        match checkers.popcnt() {
            0 => self.gen_no_check(&mut moves),
            1 => self.gen_check(&mut moves, checkers),
            2 => self.gen_double_check(&mut moves, checkers),
            _ => panic!("Triple Check (???)"),
        }

        moves
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn gen_moves_to<const KING_MOVES: bool>(&self, moves: &mut MoveList, valid: Bitboard, checker: Option<Piece>) {
        let (attack_mask, pinned) = self.calc_pins();
        let masked_attacks = Wordboard(self.attack_table(self.stm).0 & attack_mask);

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
        moves.write_capt_promos(
            self,
            &masked_attacks,
            pawn_mask,
            pawn_dest & their_pieces & their_backrank
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

        let pinned_pawn_mask = pinned & !self.king(self.stm).file().bitboard();
        let valid_pawns = self.color_pieces(self.stm, Piece::Pawn) & !pinned_pawn_mask;
        let (normal_empty, double_empty) = pawn_empty(self.stm, empty, valid);

        let pawn_normal = valid_pawns & normal_empty;
        let pawn_double = valid_pawns & double_empty;

        let promo_mask = Mask8((pawn_normal >> PAWN_PROMO_SHIFT[self.stm]).0 as u8);
        moves.write4x8(unsafe { u64x8::load(PAWN_PROMOS[self.stm].as_ptr()) }, promo_mask);

        let normal_mask = (pawn_normal >> PAWN_DOUBLE_SHIFT[self.stm]).0 as u8;
        let double_mask = (pawn_double >> PAWN_DOUBLE_SHIFT[self.stm]).0 as u8;
        let double_mask = Mask16(double_mask as u16 | ((normal_mask as u16) << 8));
        moves.write16(unsafe { u16x16::load(PAWN_DOUBLE[self.stm].as_ptr()) }, double_mask);

        let normal_mask = Mask32((pawn_normal >> PAWN_NORMAL_SHIFT).0 as u32);
        moves.write32(unsafe { u16x32::load(PAWN_NORMAL[self.stm].as_ptr())}, normal_mask);

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

                        if !pinned.has(rook_src)
                            && blockers.is_disjoint(must_be_empty)
                            && their_attacks.is_disjoint(must_be_safe) {
                            moves.push(Move::new(king_src, rook_src, $flag));
                        }
                    }
                };
            }

            write_castling_moves!(File::G, File::F, rights.short, MoveFlag::ShortCastling);
            write_castling_moves!(File::C, File::D, rights.long, MoveFlag::LongCastling);
        }
    }

    #[inline]
    fn gen_king_moves<const CHECKERS: usize>(&self, moves: &mut MoveList, checkers: PieceMask) {
        let (our_attacks, their_attacks) = (self.attack_table(self.stm), self.attack_table(!self.stm));
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

        for dest in valid {
            let flag = if their_pieces.has(dest) {
                MoveFlag::Capture
            } else {
                MoveFlag::Normal
            };

            moves.push(Move::new(our_king, dest, flag));
        }
    }

    #[inline]
    fn gen_no_check(&self, moves: &mut MoveList) {
        self.gen_moves_to::<true>(moves, Bitboard::FULL, None)
    }

    #[inline]
    fn gen_check(&self, moves: &mut MoveList, checkers: PieceMask) {
        let king = self.king(self.stm);
        let checker = checkers.next().unwrap();
        let checker_piece = self.index_to_piece[!self.stm][checker].unwrap();
        let checker_sq = self.index_to_square[!self.stm][checker].unwrap();
        let valid = if checker_piece == Piece::Knight {
            checker_sq.bitboard()
        } else {
            between(king, checker_sq) | checker_sq
        };

        self.gen_moves_to::<false>(moves, valid, Some(checker_piece));
        self.gen_king_moves::<1>(moves, checkers);
    }

    #[inline]
    fn gen_double_check(&self, moves: &mut MoveList, checkers: PieceMask) {
        self.gen_king_moves::<2>(moves, checkers);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn calc_pins(&self) -> (u16x64, Bitboard) {
        let king = self.king(self.stm);
        let (ray_perm, ray_valid) = ray_perm(king);
        let (inv_perm, inv_valid) = inv_perm(king);
        let ray_places = self.inner.permute(ray_perm).mask(Mask64(ray_valid));

        let their_color = match self.stm {
            Color::White => ray_places.msb(),
            Color::Black => !ray_places.msb(),
        };
        let blockers = ray_places.nonzero().to_bitmask() & NON_HORSE_ATTACK_MASK;
        let sliders = ray_sliders(ray_places);
        let closest = extend_bitrays(blockers, ray_valid) & blockers;
        let pinner_bitrays = extend_bitrays(blockers & !closest, ray_valid) & NON_HORSE_ATTACK_MASK;
        let second_closest = pinner_bitrays & blockers & !closest;

        let their_pieces = their_color & blockers;
        let pinners = their_pieces & sliders & second_closest;
        let pinned = !their_pieces & u64x2::splat(closest)
            .to_u8x16()
            .mask(u64x2::splat(pinners.to_bitmask()).to_u8x16().nonzero())
            .to_u64x2()
            .extract::<0>();
        let pinned_bitmask = pinned.to_bitmask();
        let pinned_places = ray_places
            .mask(pinned)
            .extend_rays()
            .mask(Mask64(pinner_bitrays))
            .permute(inv_perm)
            .mask(Mask64(inv_valid));

        let index = unsafe { u8x16::load(self.index_to_square[self.stm].0.as_ptr()) };
        let pinned_coords = ray_perm.compress(pinned).extract16::<0>();
        let pinned_count = pinned_bitmask.count_ones() as usize;
        let pinned_mask = index.findset(pinned_coords, pinned_count);

        let pinned_indices = pinned_places & u8x64::splat(Place::INDEX_MASK);
        let valid_indices = pinned_indices.nonzero();
        let table_mask = u16x64::splat(!pinned_mask) | u16x64::splat(1)
            .shlv(pinned_indices.zero_ext())
            .mask(valid_indices);

        (table_mask, Bitboard(valid_indices.to_bitmask()))
    }
}

/*----------------------------------------------------------------*/

#[inline]
fn pawn_empty(stm: Color, empty: Bitboard, valid: Bitboard) -> (Bitboard, Bitboard) {
    let valid_empty = valid & empty;

    match stm {
        Color::White => (valid_empty.shift::<South>(1), empty.shift::<South>(1) & valid_empty.shift::<South>(2)),
        Color::Black => (valid_empty.shift::<North>(1), empty.shift::<North>(1) & valid_empty.shift::<North>(2))
    }
}

const PAWN_NORMAL_SHIFT: usize = 16;
const PAWN_NORMAL: [[Move; 32]; Color::COUNT] = {
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
const PAWN_DOUBLE: [[Move; 16]; Color::COUNT] = {
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
const PAWN_PROMOS: [[Move; 32]; Color::COUNT] = {
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