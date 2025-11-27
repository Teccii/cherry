use core::{ops::*, ptr};

use arrayvec::ArrayVec;

use crate::*;

/*----------------------------------------------------------------*/
#[derive(Debug, Clone)]
pub struct MoveList {
    inner: ArrayVec<Move, 256>,
}

impl MoveList {
    #[inline]
    pub fn empty() -> MoveList {
        MoveList { inner: ArrayVec::new() }
    }

    #[inline]
    pub(crate) fn write128_16(&mut self, mask: Vec128Mask16, vec: Vec128) {
        let len = self.inner.len();
        unsafe {
            let ptr = self.inner.as_mut_ptr().add(len);

            Vec128::compress_store16(ptr, mask, vec);
            self.inner.set_len(len + mask.count_ones() as usize);
        }
    }

    #[inline]
    pub(crate) fn write256_16(&mut self, mask: Vec256Mask16, vec: Vec256) {
        let len = self.inner.len();
        unsafe {
            let ptr = self.inner.as_mut_ptr().add(len);

            Vec256::compress_store16(ptr, mask, vec);
            self.inner.set_len(len + mask.count_ones() as usize);
        }
    }

    #[inline]
    pub(crate) fn write512_16(&mut self, mask: Vec512Mask16, vec: Vec512) {
        let len = self.inner.len();
        unsafe {
            let ptr = self.inner.as_mut_ptr().add(len);

            Vec512::compress_store16(ptr, mask, vec);
            self.inner.set_len(len + mask.count_ones() as usize);
        }
    }

    #[inline]
    pub(crate) fn write512_64(&mut self, mask: Vec512Mask64, vec: Vec512) {
        let len = self.inner.len();
        unsafe {
            let ptr = self.inner.as_mut_ptr().add(len);

            Vec512::compress_store64(ptr, mask, vec);
            self.inner.set_len(len + 4 * mask.count_ones() as usize);
        }
    }

    #[inline]
    pub(crate) fn write_promotions(&mut self, moves: u64) {
        let len = self.inner.len();

        unsafe {
            let ptr = self.inner.as_mut_ptr().add(len);

            ptr::copy_nonoverlapping((&moves as *const u64).cast(), ptr, size_of::<u64>());
            self.inner.set_len(len + 4);
        }
    }
}

impl Deref for MoveList {
    type Target = ArrayVec<Move, 256>;

    #[inline]
    fn deref(&self) -> &ArrayVec<Move, 256> {
        &self.inner
    }
}

impl DerefMut for MoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/*----------------------------------------------------------------*/

impl Board {
    #[inline]
    pub fn gen_moves(&self) -> MoveList {
        let stm = self.stm;
        let our_king = self.king(stm);

        let checkers = self.attack_table(!stm).get(our_king);
        let mut moves = MoveList::empty();

        match checkers.popcnt() {
            0 => self.gen_moves_to::<true>(&mut moves, our_king, Bitboard::FULL, None),
            1 => {
                let checker_index = checkers.lsb();
                let checker_piece = self.index_to_piece[!stm][checker_index].unwrap();
                let checker_sq = self.index_to_square[!stm][checker_index].unwrap();

                self.gen_moves_to::<false>(
                    &mut moves,
                    our_king,
                    if checker_piece == Piece::Knight {
                        checker_sq.bitboard()
                    } else {
                        between(our_king, checker_sq) | checker_sq
                    },
                    Some(checker_piece),
                );
                self.gen_king_moves_in_check::<1>(&mut moves, our_king, checkers);
            }
            2 => self.gen_king_moves_in_check::<2>(&mut moves, our_king, checkers),
            _ => unreachable!(),
        }

        moves
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn gen_moves_to<const KING_MOVES: bool>(&self, moves: &mut MoveList, our_king: Square, valid_dest: Bitboard, checker: Option<Piece>) {
        let stm = self.stm;
        let our_attack_table = self.attack_table(stm);

        let (pin_mask, pinned) = self.calc_pins();
        let masked_attack_table = *(*our_attack_table & pin_mask).as_mailbox();

        let valid_pieces = self.index_to_piece[stm].valid();
        let pawn_mask = self.index_to_piece[stm].mask_eq(Piece::Pawn);
        let non_pawn_mask = valid_pieces & !pawn_mask & !PieceMask::KING;

        let empty = self.empty();
        let their_pieces = self.colors(!stm);
        let their_attacks = self.attack_table(!stm).all();
        let pawn_dest = our_attack_table.for_mask(pawn_mask) & valid_dest;
        let non_pawn_dest = our_attack_table.for_mask(non_pawn_mask) & valid_dest;
        let king_dest = our_attack_table.for_mask(PieceMask::KING) & valid_dest;

        let src = unsafe { Vec128::load(self.index_to_square[stm].into_inner().as_ptr()) }.zext8to16();
        let pawn_info = pawns::pawn_info(stm);

        self.gen_capture_promotions_for(
            moves,
            &masked_attack_table,
            pawn_mask,
            pawn_dest & their_pieces & pawn_info.promo_dest,
        );
        if let Some(sq) = self.ep_square()
            && (KING_MOVES || checker == Some(Piece::Pawn))
        {
            let mask = masked_attack_table[sq] & pawn_mask;

            if !mask.is_empty() {
                let victim = sq.offset(0, -stm.sign() as i8);
                let pinned = {
                    let our_piece = self.index_to_square[stm][mask.lsb()].unwrap();

                    let dir = match victim.file() < our_king.file() {
                        true => -1,
                        false => 1,
                    };

                    let mut test_sq = our_king;
                    let mut result = true;

                    while let Some(sq) = test_sq.try_offset(dir, 0) {
                        let place = self.board.get(sq);

                        if !(place.is_empty() || sq == victim || sq == our_piece) {
                            let (color, piece) = (place.color().unwrap(), place.piece().unwrap());

                            result = color == stm || (piece != Piece::Rook && piece != Piece::Queen);
                            break;
                        }

                        test_sq = sq;
                    }

                    result
                };

                if mask.popcnt() > 1 || victim.rank() != our_king.rank() || pinned {
                    let dest = Vec256::splat16(((sq as u16) << 6) | MoveFlag::EnPassant as u16);
                    moves.write256_16(mask.into_inner(), src | dest);
                }
            }
        }

        self.gen_captures_for(
            moves,
            &masked_attack_table,
            pawn_mask,
            src,
            pawn_dest & their_pieces & pawn_info.non_promo_dest,
        );
        self.gen_captures_for(moves, &masked_attack_table, non_pawn_mask, src, non_pawn_dest & their_pieces);

        if KING_MOVES {
            self.gen_captures_for(
                moves,
                &masked_attack_table,
                PieceMask::KING,
                src,
                king_dest & their_pieces & !their_attacks,
            );
            self.gen_quiets_for(
                moves,
                &masked_attack_table,
                PieceMask::KING,
                src,
                king_dest & empty & !their_attacks,
            );

            let blockers = self.occupied();
            let our_backrank = Rank::First.relative_to(stm);
            let rights = self.castle_rights(stm);

            macro_rules! write_castling {
                ($blockers:ident, $our_backrank:ident, $king_src:ident, $rook_src:expr, $flag:expr, $king_dest:expr, $rook_dest:expr) => {
                    if let Some(rook_src) = $rook_src.map(|f| Square::new(f, $our_backrank)) {
                        let king_dest = Square::new($king_dest, $our_backrank);
                        let rook_dest = Square::new($rook_dest, $our_backrank);
                        let king_to_rook = between($king_src, rook_src);
                        let king_to_dest = between($king_src, king_dest);
                        let must_be_safe = king_to_dest | king_dest;
                        let must_be_empty = must_be_safe | king_to_rook | rook_dest;
                        let blockers = $blockers ^ $king_src ^ rook_src;

                        if !pinned.has(rook_src) && blockers.is_disjoint(must_be_empty) && their_attacks.is_disjoint(must_be_safe) {
                            moves.push(Move::new($king_src, rook_src, $flag));
                        }
                    }
                };
            }

            write_castling!(
                blockers,
                our_backrank,
                our_king,
                rights.short,
                MoveFlag::ShortCastling,
                File::G,
                File::F
            );
            write_castling!(
                blockers,
                our_backrank,
                our_king,
                rights.long,
                MoveFlag::LongCastling,
                File::C,
                File::D
            );
        }

        self.gen_quiets_for(moves, &masked_attack_table, non_pawn_mask, src, non_pawn_dest & empty);

        let pinned_pawns = pinned & !our_king.file().bitboard();
        let bb = self.color_pieces(Piece::Pawn, stm) & !pinned_pawns;
        let (normal_empty, double_empty) = pawns::pawn_empty(stm, empty, valid_dest);
        let pawn_moves = pawns::pawn_moves(stm);

        let pawn_normal = bb & normal_empty;
        let pawn_double = bb & double_empty;

        let mask = (pawn_normal >> pawn_info.promo_shift).0 as u8;
        moves.write512_64(mask, pawn_moves.promotions);

        let mask = (pawn_normal >> pawn_info.normal_shift).0 as u32;
        moves.write512_16(mask, pawn_moves.normal_moves);

        let normal_mask = (pawn_normal >> pawn_info.double_shift).0 as u8;
        let double_mask = (pawn_double >> pawn_info.double_shift).0 as u8;
        let mask = ((double_mask as u16) << 8) | normal_mask as u16;
        moves.write256_16(mask, pawn_moves.double_moves);
    }

    #[inline]
    fn gen_king_moves_in_check<const CHECKERS: usize>(&self, moves: &mut MoveList, our_king: Square, mut checkers: PieceMask) {
        let stm = self.stm;
        let attack_table = self.attack_table(!stm);

        let (king_rays, rays_valid) = geometry::superpiece_rays(our_king);
        let (king_leaps, leaps_valid) = geometry::adjacents(our_king);
        let king_leaps16 = king_leaps.zext8to16lo();

        let places = Vec512::permute8(Vec512::from(king_leaps), self.board.inner).into_vec128();
        let blockers = places.nonzero8();
        let color = places.msb8()
            ^ match stm {
                Color::White => u16::MAX,
                Color::Black => 0,
            };
        let our_pieces = blockers & color;

        let [half0, half1] = attack_table.inner.map(|v| v.zero16() as u32);
        let at_empty = interleave64(half0, half1);
        let no_attackers = Vec128::mask_bitshuffle(leaps_valid, Vec128::from(at_empty), king_leaps);

        let mut dest = (leaps_valid & !our_pieces & no_attackers) as u8;
        let mut additional_checks = 0;

        for _ in 0..CHECKERS {
            let checker_index = checkers.lsb();
            let checker_piece = self.index_to_piece[!stm][checker_index].unwrap();
            let checker_sq = self.index_to_square[!stm][checker_index].unwrap();

            if checker_piece.is_slider() {
                const VALID_MASK: [u8; 3] = [0b10101010, 0b01010101, 0b11111111];

                let dir = (rays_valid & Vec512::eq8(king_rays, Vec512::splat8(checker_sq as u8)))
                    .rotate_left(32)
                    .trailing_zeros()
                    / 8;
                additional_checks |= (1u8 << dir) & VALID_MASK[checker_piece.bits() as usize - Piece::Bishop.bits() as usize];
            }

            checkers &= PieceMask::new(checkers.into_inner() - 1);
        }

        dest &= !additional_checks;

        let write_vec = Vec128::shl16::<6>(king_leaps16)
            | Vec128::splat16(our_king as u16)
            | Vec128::mask16(blockers as u8, Vec128::splat16(MoveFlag::Capture as u16));
        moves.write128_16(dest, write_vec);
    }

    #[inline]
    fn gen_quiets_for(&self, moves: &mut MoveList, attack_table: &[PieceMask; Square::COUNT], mask: PieceMask, src: Vec256, dest: Bitboard) {
        for sq in dest {
            let mask = mask & attack_table[sq];

            if !mask.is_empty() {
                let dest = Vec256::splat16((sq as u16) << 6);

                moves.write256_16(mask.into_inner(), src | dest);
            }
        }
    }

    #[inline]
    fn gen_captures_for(&self, moves: &mut MoveList, attack_table: &[PieceMask; Square::COUNT], mask: PieceMask, src: Vec256, dest: Bitboard) {
        for sq in dest {
            let mask = mask & attack_table[sq];

            if !mask.is_empty() {
                let dest = Vec256::splat16(((sq as u16) << 6) | MoveFlag::Capture as u16);

                moves.write256_16(mask.into_inner(), src | dest);
            }
        }
    }

    #[inline]
    fn gen_capture_promotions_for(&self, moves: &mut MoveList, attack_table: &[PieceMask; Square::COUNT], mask: PieceMask, dest: Bitboard) {
        let stm = self.stm;

        for sq in dest {
            let mask = mask & attack_table[sq];

            for index in mask {
                let src = self.index_to_square[stm][index].expect(&format!("{} | {} | {:?}", self.to_fen(true), index.into_inner(), self));
                let base_move = ((sq as u16) << 6) | src as u16;
                let promos = MoveFlag::CapturePromotionQueen as u64
                    | ((MoveFlag::CapturePromotionRook as u64) << 16)
                    | ((MoveFlag::CapturePromotionBishop as u64) << 32)
                    | ((MoveFlag::CapturePromotionKnight as u64) << 48);

                moves.write_promotions(promos + 0x0001000100010001 * (base_move as u64));
            }
        }
    }

    #[inline]
    pub(super) fn calc_pins(&self) -> (Wordboard, Bitboard) {
        let stm = self.stm;
        let our_king = self.king(stm);

        let (ray_coords, ray_valid) = geometry::superpiece_rays(our_king);
        let ray_places = Vec512::permute8(ray_coords, self.board.inner);
        let inv_perm = geometry::superpiece_inv_rays(our_king);

        let color = ray_places.msb8()
            ^ match stm {
                Color::White => Bitboard::EMPTY,
                Color::Black => Bitboard::FULL,
            };
        let blockers = ray_places.nonzero8() & geometry::NON_HORSE_ATTACK_MASK;
        let sliders = geometry::sliders_from_rays(ray_places);
        let closest = geometry::superpiece_attacks(blockers, ray_valid) & blockers;
        let pin_raymask = geometry::superpiece_attacks(blockers & !closest, ray_valid) & geometry::NON_HORSE_ATTACK_MASK;
        let second_closest = pin_raymask & blockers & !closest;

        let their_pieces = blockers & color;
        let pinners = their_pieces & sliders & second_closest;
        let pinned = !their_pieces & Vec128::mask8(Vec128::from(pinners.0).nonzero8(), Vec128::from(closest)).into_u64();

        let pinned_ids = Vec512::mask8(pin_raymask, Vec512::lane_splat8to64(Vec512::mask8(pinned.0, ray_places)));
        let pinned_ids = Vec512::permute8_mz(!inv_perm.msb8(), inv_perm, pinned_ids);

        let pinned_count = pinned.popcnt();
        let pinned_coords = Vec512::compress8(pinned.0, ray_coords).into_vec128();
        let sq_idx = unsafe { Vec128::load(self.index_to_square[stm].into_inner().as_ptr()) };
        let piece_mask = Vec128::findset8(pinned_coords, pinned_count as i32, sq_idx);

        let ones = Vec512::splat16(1);
        let valid_ids = pinned_ids.nonzero8();
        let masked_ids = pinned_ids & Vec512::splat8(Place::INDEX_MASK);
        let bits0 = Vec512::shlv16_mz(valid_ids as Vec512Mask16, ones, masked_ids.into_vec256().zext8to16());
        let bits1 = Vec512::shlv16_mz(
            (valid_ids >> 32) as Vec512Mask16,
            ones,
            masked_ids.extract_vec256::<1>().zext8to16(),
        );
        let table_mask0 = Vec512::splat16(!piece_mask) | bits0;
        let table_mask1 = Vec512::splat16(!piece_mask) | bits1;

        (Wordboard::new(table_mask0, table_mask1), Bitboard(pinned_ids.nonzero8()))
    }
}

/*----------------------------------------------------------------*/

mod pawns {
    use cherry_types::*;

    use crate::{Move, MoveFlag, Vec256, Vec512};

    #[inline]
    pub fn pawn_empty(stm: Color, empty: Bitboard, valid_dest: Bitboard) -> (Bitboard, Bitboard) {
        let valid_empty = empty & valid_dest;

        match stm {
            Color::White => (valid_empty.shift::<Down>(1), empty.shift::<Down>(1) & valid_empty.shift::<Down>(2)),
            Color::Black => (valid_empty.shift::<Up>(1), empty.shift::<Up>(1) & valid_empty.shift::<Up>(2)),
        }
    }

    /*----------------------------------------------------------------*/

    pub struct PawnInfo {
        pub promo_dest: Bitboard,
        pub non_promo_dest: Bitboard,
        pub promo_shift: u8,
        pub double_shift: u8,
        pub normal_shift: u8,
    }

    #[inline]
    pub fn pawn_info(stm: Color) -> PawnInfo {
        match stm {
            Color::White => PawnInfo {
                promo_dest: Bitboard(0xFF00000000000000),
                non_promo_dest: Bitboard(0x00FFFFFFFFFF0000),
                promo_shift: 48,
                double_shift: 8,
                normal_shift: 16,
            },
            Color::Black => PawnInfo {
                promo_dest: Bitboard(0x00000000000000FF),
                non_promo_dest: Bitboard(0x0000FFFFFFFFFF00),
                promo_shift: 8,
                double_shift: 48,
                normal_shift: 16,
            },
        }
    }

    /*----------------------------------------------------------------*/

    pub struct PawnMoves {
        pub normal_moves: Vec512,
        pub double_moves: Vec256,
        pub promotions: Vec512,
    }

    #[inline]
    pub fn pawn_moves(stm: Color) -> PawnMoves {
        let mut normal_moves = [0u16; 32];
        let normal_shift = stm.sign() * 8;

        for i in 16..48 {
            normal_moves[i - 16] = Move::new(
                Square::index(i),
                Square::index((i as i16 + normal_shift) as usize),
                MoveFlag::Normal,
            )
            .bits();
        }

        let mut double_moves = [0u16; 16];
        let double_offset = match stm {
            Color::White => 8,
            Color::Black => 48,
        };
        let double_shift = stm.sign() * 16;

        for i in 0..8 {
            let src = i + double_offset;
            double_moves[i] = Move::new(
                Square::index(src),
                Square::index((src as i16 + normal_shift) as usize),
                MoveFlag::Normal,
            )
            .bits();
            double_moves[i + 8] = Move::new(
                Square::index(src),
                Square::index((src as i16 + double_shift) as usize),
                MoveFlag::DoublePush,
            )
            .bits();
        }

        let mut promotions = [0u16; 32];
        let promotion_offset = match stm {
            Color::White => 48,
            Color::Black => 8,
        };

        for i in 0..8 {
            let src = i + promotion_offset;
            promotions[i * 4] = Move::new(
                Square::index(src),
                Square::index((src as i16 + normal_shift) as usize),
                MoveFlag::PromotionQueen,
            )
            .bits();
            promotions[i * 4 + 1] = Move::new(
                Square::index(src),
                Square::index((src as i16 + normal_shift) as usize),
                MoveFlag::PromotionRook,
            )
            .bits();
            promotions[i * 4 + 2] = Move::new(
                Square::index(src),
                Square::index((src as i16 + normal_shift) as usize),
                MoveFlag::PromotionBishop,
            )
            .bits();
            promotions[i * 4 + 3] = Move::new(
                Square::index(src),
                Square::index((src as i16 + normal_shift) as usize),
                MoveFlag::PromotionKnight,
            )
            .bits();
        }

        PawnMoves {
            normal_moves: normal_moves.into(),
            double_moves: double_moves.into(),
            promotions: promotions.into(),
        }
    }
}
