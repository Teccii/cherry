mod move_gen;
mod parse;
mod perft;
mod print;
mod see;
mod startpos;

pub use move_gen::*;

use core::ops::Deref;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardStatus {
    Ongoing,
    Draw,
    Checkmate,
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct CastleRights {
    pub short: Option<File>,
    pub long: Option<File>,
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Board {
    board: Byteboard,
    xray_tables: [Wordboard; Color::COUNT],
    attack_tables: [Wordboard; Color::COUNT],
    index_to_square: [IndexToSquare; Color::COUNT],
    index_to_piece: [IndexToPiece; Color::COUNT],
    castle_rights: [CastleRights; Color::COUNT],
    en_passant: Option<File>,
    fullmove_count: u16,
    halfmove_clock: u8,
    pawn_hash: u64,
    minor_hash: u64,
    major_hash: u64,
    hash: u64,
    stm: Color,
}

impl Board {
    #[inline]
    pub fn castle_rights(&self, color: Color) -> CastleRights {
        self.castle_rights[color]
    }

    #[inline]
    pub fn xray_table(&self, color: Color) -> &Wordboard {
        &self.xray_tables[color]
    }

    #[inline]
    pub fn attack_table(&self, color: Color) -> &Wordboard {
        &self.attack_tables[color]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn en_passant(&self) -> Option<File> {
        self.en_passant
    }

    #[inline]
    pub fn ep_square(&self) -> Option<Square> {
        self.en_passant.map(|f| Square::new(f, Rank::Sixth.relative_to(self.stm)))
    }

    #[inline]
    pub const fn fullmove_count(&self) -> u16 {
        self.fullmove_count
    }

    #[inline]
    pub const fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    #[inline]
    pub const fn pawn_hash(&self) -> u64 {
        self.pawn_hash
    }

    #[inline]
    pub const fn minor_hash(&self) -> u64 {
        self.minor_hash
    }

    #[inline]
    pub const fn major_hash(&self) -> u64 {
        self.major_hash
    }

    #[inline]
    pub const fn hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    pub const fn stm(&self) -> Color {
        self.stm
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.board.get(sq).piece()
    }

    #[inline]
    pub fn color_on(&self, sq: Square) -> Option<Color> {
        self.board.get(sq).color()
    }

    #[inline]
    pub fn king(&self, color: Color) -> Square {
        self.index_to_square[color][PieceIndex::new(0)].unwrap()
    }

    #[inline]
    pub fn pinned(&self, color: Color) -> Bitboard {
        let our_king = self.king(color);
        let our_pieces = self.colors(color);
        let pinners = self.xray_table(!color).get(our_king);
        let mut pinned = Bitboard::EMPTY;

        for index in pinners {
            pinned |= our_pieces & between(our_king, self.index_to_square[!color][index].unwrap());
        }

        pinned
    }

    #[inline]
    pub fn checkers(&self) -> Bitboard {
        let stm = self.stm;
        let checker_mask = self.attack_table(!stm).get(self.king(stm));
        let mut checkers = Bitboard::EMPTY;

        for index in checker_mask {
            checkers |= self.index_to_square[!stm][index].unwrap();
        }

        checkers
    }

    #[inline]
    pub fn in_check(&self) -> bool {
        let stm = self.stm;

        !self.attack_table(!stm).get(self.king(stm)).is_empty()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn status(&self) -> BoardStatus {
        if !self.gen_moves().is_empty() {
            if self.halfmove_clock < 100 {
                BoardStatus::Ongoing
            } else {
                BoardStatus::Draw
            }
        } else if self.in_check() {
            BoardStatus::Checkmate
        } else {
            BoardStatus::Draw
        }
    }

    /*----------------------------------------------------------------*/

    pub fn make_move(&mut self, mv: Move) {
        let (src, dest) = (mv.from(), mv.to());
        let (src_place, dest_place) = (self.board.get(src), self.board.get(dest));
        let (src_id, dest_id) = (src_place.index().unwrap(), dest_place.index());
        let (src_piece, dest_piece) = (src_place.piece().unwrap(), dest_place.piece());
        let mut new_ep = None;

        #[inline]
        fn castling(
            board: &mut Board,
            src: Square,
            dest: Square,
            src_id: PieceIndex,
            dest_id: Option<PieceIndex>,
            king_dest: File,
            rook_dest: File,
        ) {
            let stm = board.stm;
            let our_backrank = Rank::First.relative_to(stm);
            let king_dest = Square::new(king_dest, our_backrank);
            let rook_dest = Square::new(rook_dest, our_backrank);
            let king_src = src;
            let rook_src = dest;
            let king_id = src_id;
            let rook_id = dest_id.unwrap();

            board.update_slider(king_src);
            board.board.set(king_src, Place::EMPTY);
            board.remove_attacks(stm, king_id);

            board.update_slider(rook_src);
            board.board.set(rook_src, Place::EMPTY);
            board.remove_attacks(stm, rook_id);

            board.board.set(king_dest, Place::from_piece(Piece::King, stm, king_id));
            board.update_slider(king_dest);
            board.add_attacks(king_dest, Piece::King, stm, king_id);

            board.board.set(rook_dest, Place::from_piece(Piece::Rook, stm, rook_id));
            board.update_slider(rook_dest);
            board.add_attacks(rook_dest, Piece::Rook, stm, rook_id);

            board.index_to_square[stm][king_id] = Some(king_dest);
            board.index_to_square[stm][rook_id] = Some(rook_dest);
            board.xor_piece(king_src, Piece::King, stm);
            board.xor_piece(king_dest, Piece::King, stm);
            board.xor_piece(rook_src, Piece::Rook, stm);
            board.xor_piece(rook_dest, Piece::Rook, stm);

            board.halfmove_clock = (board.halfmove_clock + 1).min(100);
            board.set_castle_rights(stm, true, None);
            board.set_castle_rights(stm, false, None);
        }

        #[inline]
        fn capture_promotion(
            board: &mut Board,
            src: Square,
            dest: Square,
            src_id: PieceIndex,
            dest_id: Option<PieceIndex>,
            src_piece: Piece,
            dest_piece: Option<Piece>,
            promotion: Piece,
        ) {
            let dest_id = dest_id.unwrap();
            let stm = board.stm;

            board.index_to_piece[stm][src_id] = Some(promotion);
            board.index_to_square[!stm][dest_id] = None;
            board.index_to_piece[!stm][dest_id] = None;

            board.remove_attacks(!stm, dest_id);
            board.move_piece::<false>(stm, src, dest, promotion, src_id);
            board.halfmove_clock = 0;

            board.xor_piece(src, src_piece, stm);
            board.xor_piece(dest, promotion, stm);
            board.xor_piece(dest, dest_piece.unwrap(), !stm);

            check_castle_rights(board, !stm, dest);
        }

        #[inline]
        fn promotion(
            board: &mut Board,
            src: Square,
            dest: Square,
            src_id: PieceIndex,
            src_piece: Piece,
            promotion: Piece,
        ) {
            let stm = board.stm;

            board.index_to_piece[stm][src_id] = Some(promotion);
            board.move_piece::<true>(stm, src, dest, promotion, src_id);
            board.halfmove_clock = 0;

            board.xor_piece(src, src_piece, stm);
            board.xor_piece(dest, promotion, stm);
        }

        #[inline]
        fn check_castle_rights(board: &mut Board, color: Color, sq: Square) {
            if sq.rank() == Rank::First.relative_to(color) {
                let rights = board.castle_rights(color);
                let file = sq.file();

                if rights.short == Some(file) {
                    board.set_castle_rights(color, true, None);
                }

                if rights.long == Some(file) {
                    board.set_castle_rights(color, false, None);
                }
            }
        }

        match mv.flag() {
            MoveFlag::Normal => {
                let stm = self.stm;

                self.move_piece::<true>(stm, src, dest, src_piece, src_id);

                if src_piece != Piece::Pawn {
                    self.halfmove_clock = (self.halfmove_clock + 1).min(100);
                } else {
                    self.halfmove_clock = 0;
                }

                match src_piece {
                    Piece::Rook => check_castle_rights(self, stm, src),
                    Piece::King => {
                        self.set_castle_rights(stm, true, None);
                        self.set_castle_rights(stm, false, None);
                    },
                    _ => { }
                }

                self.xor_piece(src, src_piece, stm);
                self.xor_piece(dest, src_piece, stm);
            },
            MoveFlag::DoublePush => {
                let stm = self.stm;

                self.move_piece::<true>(self.stm, src, dest, src_piece, src_id);
                self.halfmove_clock = 0;

                self.xor_piece(src, src_piece, stm);
                self.xor_piece(dest, src_piece, stm);

                let their_pawns = self.index_to_piece[!stm].mask_eq(Piece::Pawn);
                let their_attacks = self.attack_table(!stm).get(src.offset(0, stm.sign() as i8));

                if !(their_attacks & their_pawns).is_empty() {
                    new_ep = Some(src.file());
                }
            },
            MoveFlag::Capture => {
                let stm = self.stm;
                let dest_id = dest_id.unwrap();

                self.index_to_square[!stm][dest_id] = None;
                self.index_to_piece[!stm][dest_id] = None;

                self.remove_attacks(!stm, dest_id);
                self.move_piece::<false>(stm, src, dest, src_piece, src_id);
                self.halfmove_clock = 0;

                self.xor_piece(src, src_piece, stm);
                self.xor_piece(dest, src_piece, stm);
                self.xor_piece(dest, dest_piece.unwrap(), !stm);
                
                match src_piece {
                    Piece::Rook => check_castle_rights(self, stm, src),
                    Piece::King => {
                        self.set_castle_rights(stm, true, None);
                        self.set_castle_rights(stm, false, None);
                    },
                    _ => { }
                }

                check_castle_rights(self, !stm, dest);
            },
            MoveFlag::EnPassant => {
                let stm = self.stm;
                let victim_sq = Square::new(dest.file(), src.rank());
                let victim_id = self.board.get(victim_sq).index().unwrap();

                self.move_piece::<true>(stm, src, dest, src_piece, src_id);
                self.update_slider(victim_sq);
                self.halfmove_clock = 0;

                self.xor_piece(src, src_piece, stm);
                self.xor_piece(dest, src_piece, stm);
                self.xor_piece(victim_sq, src_piece, stm);

                self.index_to_piece[!stm][victim_id] = None;
                self.index_to_square[!stm][victim_id] = None;
                self.board.set(victim_sq, Place::EMPTY);
                self.remove_attacks(!stm, victim_id);
            },
            MoveFlag::ShortCastling => castling(
                self,
                src,
                dest,
                src_id,
                dest_id,
                File::G,
                File::F
            ),
            MoveFlag::LongCastling => castling(
                self,
                src,
                dest,
                src_id,
                dest_id,
                File::C,
                File::D
            ),
            MoveFlag::PromotionQueen => promotion(
                self,
                src,
                dest,
                src_id,
                src_piece,
                Piece::Queen
            ),
            MoveFlag::PromotionRook => promotion(
                self,
                src,
                dest,
                src_id,
                src_piece,
                Piece::Rook
            ),
            MoveFlag::PromotionBishop => promotion(
                self,
                src,
                dest,
                src_id,
                src_piece,
                Piece::Bishop
            ),
            MoveFlag::PromotionKnight => promotion(
                self,
                src,
                dest,
                src_id,
                src_piece,
                Piece::Knight
            ),
            MoveFlag::CapturePromotionQueen => capture_promotion(
                self,
                src,
                dest,
                src_id,
                dest_id,
                src_piece,
                dest_piece,
                Piece::Queen
            ),
            MoveFlag::CapturePromotionRook => capture_promotion(
                self,
                src,
                dest,
                src_id,
                dest_id,
                src_piece,
                dest_piece,
                Piece::Rook
            ),
            MoveFlag::CapturePromotionBishop => capture_promotion(
                self,
                src,
                dest,
                src_id,
                dest_id,
                src_piece,
                dest_piece,
                Piece::Bishop
            ),
            MoveFlag::CapturePromotionKnight => capture_promotion(
                self,
                src,
                dest,
                src_id,
                dest_id,
                src_piece,
                dest_piece,
                Piece::Knight
            ),
        }

        self.set_en_passant(new_ep);
        if self.stm == Color::Black {
            self.fullmove_count += 1;
        }

        self.toggle_stm();
    }

    pub fn null_move(&mut self) -> bool {
        if self.in_check() {
            return false;
        }

        self.halfmove_clock = (self.halfmove_clock + 1).min(100);
        if self.stm == Color::Black {
            self.fullmove_count += 1;
        }

        self.set_en_passant(None);
        self.toggle_stm();

        true
    }

    pub fn is_pseudolegal(&self, mv: Move) -> bool {
        let (src, dest, flag) = (mv.from(), mv.to(), mv.flag());
        let (src_place, dest_place) = (self.board.get(src), self.board.get(dest));
        let (src_piece, src_index) = if !src_place.is_empty() {
            (src_place.piece().unwrap(), src_place.index().unwrap())
        } else {
            return false;
        };
        let stm = self.stm;

        match mv.flag() {
            MoveFlag::Normal => dest_place.is_empty() && if src_piece == Piece::Pawn {
                dest.offset(0, -stm.sign() as i8) == src
            } else {
                self.attack_table(stm).get(dest).has(src_index)
            },
            MoveFlag::Capture => !dest_place.is_empty() && dest_place.color().unwrap() != stm && self.attack_table(stm).get(dest).has(src_index),
            MoveFlag::EnPassant => src_piece == Piece::Pawn && self.ep_square().is_some_and(|ep| dest == ep) && self.attack_table(stm).get(dest).has(src_index),
            MoveFlag::DoublePush => src_piece == Piece::Pawn && src.rank() == Rank::Second.relative_to(stm) && dest.rank() == Rank::Fourth.relative_to(stm),
            _ if mv.is_castling() => src_piece == Piece::King && {
                let our_backrank = Rank::First.relative_to(stm);
                let castle_rights = if flag == MoveFlag::ShortCastling {
                    self.castle_rights(stm).short
                } else {
                    self.castle_rights(stm).long
                };

                src.rank() == our_backrank && Some(dest) == castle_rights.map(|f| Square::new(f, our_backrank))
            },
            _ if mv.is_promotion() => src_piece == Piece::Pawn && {
                let is_capture = mv.is_capture();

                src.rank() == Rank::Seventh.relative_to(stm)
                    && dest.rank() == Rank::Eighth.relative_to(stm)
                    && is_capture == !dest_place.is_empty()
                    && is_capture == self.attack_table(stm).get(dest).has(src_index)
            },
            _ => false
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn calc_hashes(&self) -> (u64, u64, u64, u64) {
        let mut hash = 0;
        let mut pawn_hash = 0;
        let mut minor_hash = 0;
        let mut major_hash = 0;

        let mailbox = self.as_mailbox();
        for &sq in &Square::ALL {
            let place = mailbox[sq];

            if !place.is_empty() {
                let (piece, color) = (place.piece().unwrap(), place.color().unwrap());
                let value = ZOBRIST.piece(sq, piece, color);

                hash ^= value;
                match piece {
                    Piece::Pawn => pawn_hash ^= value,
                    Piece::Knight | Piece::Bishop => minor_hash ^= value,
                    Piece::Rook | Piece::Queen => major_hash ^= value,
                    Piece::King => {
                        minor_hash ^= value;
                        major_hash ^= value;
                    }
                }
            }
        }

        if let Some(file) = self.en_passant {
            hash ^= ZOBRIST.en_passant(file);
        }

        for &color in &Color::ALL {
            let rights = self.castle_rights(color);

            if let Some(file) = rights.short {
                hash ^= ZOBRIST.castle_rights(file, color);
            }

            if let Some(file) = rights.long {
                hash ^= ZOBRIST.castle_rights(file, color);
            }
        }

        if self.stm == Color::Black {
            hash ^= ZOBRIST.stm;
        }

        (hash, pawn_hash, minor_hash, major_hash)
    }

    #[inline]
    pub fn calc_attacks(&self) -> ([Wordboard; Color::COUNT], [Wordboard; Color::COUNT]) {
        let mut attacks = [[PieceMask::EMPTY; Square::COUNT]; Color::COUNT];
        let mut xrays = [[PieceMask::EMPTY; Square::COUNT]; Color::COUNT];

        for &sq in &Square::ALL {
            let ([white_attacks, black_attacks], [white_xrays, black_xrays]) = self.calc_attacks_to(sq);
            attacks[Color::White][sq] = white_attacks;
            attacks[Color::Black][sq] = black_attacks;
            xrays[Color::White][sq] = white_xrays;
            xrays[Color::Black][sq] = black_xrays;
        }

        unsafe { (core::mem::transmute(attacks), core::mem::transmute(xrays)) }
    }

    #[inline]
    pub fn calc_attacks_to(&self, sq: Square) -> ([PieceMask; Color::COUNT], [PieceMask; Color::COUNT]) {
        let (ray_coords, ray_valid) = geometry::superpiece_rays(sq);
        let ray_places = Vec512::permute8(ray_coords, self.board.inner);

        let color = ray_places.msb8();
        let blockers = ray_places.nonzero8();
        let attackers = geometry::attackers_from_rays(ray_places);
        let closest = geometry::superpiece_attacks(blockers, ray_valid) & blockers;
        let second_closest = geometry::superpiece_attacks(blockers & !closest, ray_valid) & blockers & !closest;

        let white_sq_idx = unsafe { Vec128::load(self.index_to_square[Color::White].into_inner().as_ptr()) };
        let black_sq_idx = unsafe { Vec128::load(self.index_to_square[Color::Black].into_inner().as_ptr()) };

        let calc_masks = |raymask: u64| {
            let white = !color & raymask & attackers;
            let black = color & raymask & attackers;
            let white_count = white.count_ones() as i32;
            let black_count = black.count_ones() as i32;
            let white_coords = Vec512::compress8(white, ray_coords).into_vec128();
            let black_coords = Vec512::compress8(black, ray_coords).into_vec128();
            let white_mask = Vec128::findset8(white_coords, white_count, white_sq_idx);
            let black_mask = Vec128::findset8(black_coords, black_count, black_sq_idx);

            [PieceMask::new(white_mask), PieceMask::new(black_mask)]
        };

        (calc_masks(closest), calc_masks(second_closest))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn move_piece<const UPDATE_DEST_SLIDERS: bool>(&mut self, color: Color, src: Square, dest: Square, piece: Piece, index: PieceIndex) {
        self.index_to_square[color][index] = Some(dest);

        let (src_ray_coords, src_ray_valid) = geometry::superpiece_rays(src);
        let (dest_ray_coords, dest_ray_valid) = geometry::superpiece_rays(dest);
        let new_place = Vec512::splat8(Place::from_piece(piece, color, index).into_inner());

        let mut new_board = self.board.inner;
        let src_ray_places = Vec512::permute8(src_ray_coords, new_board);
        new_board = Vec512::mask8(!src.bitboard().0, new_board);
        let dest_ray_places = Vec512::permute8(dest_ray_coords, new_board);
        new_board = Vec512::blend8(dest.bitboard().0, new_board, new_place);
        self.board.inner = new_board;

        let src_swapped_perm = geometry::superpiece_inv_rays_swapped(src);
        let dest_swapped_perm = geometry::superpiece_inv_rays_swapped(dest);

        let src_blockers = src_ray_places.nonzero8();
        let dest_blockers = dest_ray_places.nonzero8();
        let src_sliders = geometry::sliders_from_rays(src_ray_places);
        let dest_sliders = geometry::sliders_from_rays(dest_ray_places);
        let src_closest = geometry::superpiece_attacks(src_blockers, src_ray_valid);
        let dest_closest = geometry::superpiece_attacks(dest_blockers, dest_ray_valid);
        let src_second_closest = geometry::superpiece_attacks(src_blockers & !src_closest, src_ray_valid);
        let dest_second_closest = geometry::superpiece_attacks(dest_blockers & !dest_closest, dest_ray_valid);

        let update_tables = |tables: &mut [Wordboard; Color::COUNT], src_raymask0: u64, dest_raymask0: u64, src_raymask1: u64, dest_raymask1: u64| {
            let src_visible = src_sliders & src_raymask0;
            let dest_visible = dest_sliders & dest_raymask0;
            let src_visible_ids = Vec512::lane_splat8to64(Vec512::mask8(src_visible, Vec512::permute8(src_ray_coords, new_board)));
            let dest_visible_ids = Vec512::lane_splat8to64(Vec512::mask8(dest_visible, dest_ray_places));
            let src_updates = Vec512::mask8((src_raymask1 & geometry::NON_HORSE_ATTACK_MASK).rotate_left(32), src_visible_ids);
            let dest_updates = Vec512::mask8((dest_raymask1 & geometry::NON_HORSE_ATTACK_MASK).rotate_left(32), dest_visible_ids);

            let src_updates = Vec512::permute8_mz(!src_swapped_perm.msb8(), src_swapped_perm, src_updates);
            let dest_updates = Vec512::permute8_mz(!dest_swapped_perm.msb8(), dest_swapped_perm, dest_updates);
            let src_valid_updates = src_updates.nonzero8();
            let dest_valid_updates = dest_updates.nonzero8();
            let src_color = src_updates.msb8();
            let dest_color = dest_updates.msb8();

            let update_mask = Vec512::splat8(0xF);
            let src_masked_updates = src_updates & update_mask;
            let dest_masked_updates = dest_updates & update_mask;

            let ones = Vec512::splat16(1);
            let src_bits0 = Vec512::shlv16_mz(src_valid_updates as Vec512Mask16, ones, src_masked_updates.into_vec256().zext8to16());
            let src_bits1 = Vec512::shlv16_mz((src_valid_updates >> 32) as Vec512Mask16, ones, src_masked_updates.extract_vec256::<1>().zext8to16());
            let dest_bits0 = Vec512::shlv16_mz(dest_valid_updates as Vec512Mask16, ones, dest_masked_updates.into_vec256().zext8to16());
            let dest_bits1 = Vec512::shlv16_mz((dest_valid_updates >> 32) as Vec512Mask16, ones, dest_masked_updates.extract_vec256::<1>().zext8to16());

            let mut update00 = Vec512::mask16(!src_color as Vec512Mask16, src_bits0);
            let mut update01 = Vec512::mask16(!(src_color >> 32) as Vec512Mask16, src_bits1);
            let mut update10 = Vec512::mask16(src_color as Vec512Mask16, src_bits0);
            let mut update11 = Vec512::mask16((src_color >> 32) as Vec512Mask16, src_bits1);

            if UPDATE_DEST_SLIDERS {
                update00 ^= Vec512::mask16(!dest_color as Vec512Mask16, dest_bits0);
                update01 ^= Vec512::mask16(!(dest_color >> 32) as Vec512Mask16, dest_bits1);
                update10 ^= Vec512::mask16(dest_color as Vec512Mask16, dest_bits0);
                update11 ^= Vec512::mask16((dest_color >> 32) as Vec512Mask16, dest_bits1);
            }

            tables[0].inner[0] ^= update00;
            tables[0].inner[1] ^= update01;
            tables[1].inner[0] ^= update10;
            tables[1].inner[1] ^= update11;
        };

        update_tables(&mut self.attack_tables, src_closest, dest_closest, src_closest, dest_closest);
        update_tables(&mut self.xray_tables, src_closest, dest_closest, src_second_closest, dest_second_closest);
        update_tables(&mut self.xray_tables, src_second_closest & !src_closest, dest_second_closest & !dest_closest, src_closest, dest_closest);

        let piece_mask = Vec512::splat16(index.into_mask().into_inner());
        let not_piece_mask = Vec512::splat16(!index.into_mask().into_inner());
        let attack_mask = dest_closest & geometry::attack_mask(piece, color);
        let xray_mask = dest_second_closest & !dest_closest & geometry::attack_mask(piece, color);
        let add_attack_mask = Vec512::mask_bitshuffle(!dest_swapped_perm.msb8(), Vec512::splat64(attack_mask.rotate_left(32)), dest_swapped_perm);
        let add_xray_mask = Vec512::mask_bitshuffle(!dest_swapped_perm.msb8(), Vec512::splat64(xray_mask.rotate_left(32)), dest_swapped_perm);

        self.attack_tables[color].inner[0] &= not_piece_mask;
        self.attack_tables[color].inner[1] &= not_piece_mask;
        self.attack_tables[color].inner[0] |= Vec512::mask16(add_attack_mask as Vec512Mask16, piece_mask);
        self.attack_tables[color].inner[1] |= Vec512::mask16((add_attack_mask >> 32) as Vec512Mask16, piece_mask);

        self.xray_tables[color].inner[0] &= not_piece_mask;
        self.xray_tables[color].inner[1] &= not_piece_mask;
        self.xray_tables[color].inner[0] |= Vec512::mask16(add_xray_mask as Vec512Mask16, piece_mask);
        self.xray_tables[color].inner[1] |= Vec512::mask16((add_xray_mask >> 32) as Vec512Mask16, piece_mask);
    }

    #[inline]
    fn update_slider(&mut self, sq: Square) {
        let (ray_coords, ray_valid) = geometry::superpiece_rays(sq);
        let ray_places = Vec512::permute8(ray_coords, self.board.inner);
        let swapped_perm = geometry::superpiece_inv_rays_swapped(sq);

        let blockers = ray_places.nonzero8();
        let sliders = geometry::sliders_from_rays(ray_places);
        let closest = geometry::superpiece_attacks(blockers, ray_valid) & geometry::NON_HORSE_ATTACK_MASK;
        let second_closest = geometry::superpiece_attacks(blockers & !closest, ray_valid) & geometry::NON_HORSE_ATTACK_MASK;

        let update_tables = |tables: &mut [Wordboard; Color::COUNT], raymask0: u64, raymask1: u64| {
            let visible = raymask0 & sliders;
            let visible_ids = Vec512::lane_splat8to64(Vec512::mask8(visible, ray_places));

            let updates = Vec512::mask8(raymask1.rotate_left(32), visible_ids);
            let updates = Vec512::permute8_mz(!swapped_perm.msb8(), swapped_perm, updates);
            let masked_updates = updates & Vec512::splat8(0xF);
            let valid_updates = updates.nonzero8();
            let color = updates.msb8();

            let ones = Vec512::splat16(1);
            let bits0 = Vec512::shlv16_mz(valid_updates as Vec512Mask16, ones, masked_updates.into_vec256().zext8to16());
            let bits1 = Vec512::shlv16_mz((valid_updates >> 32) as Vec512Mask16, ones, masked_updates.extract_vec256::<1>().zext8to16());

            tables[0].inner[0] ^= Vec512::mask16(!color as Vec512Mask16, bits0);
            tables[0].inner[1] ^= Vec512::mask16(!(color >> 32) as Vec512Mask16, bits1);
            tables[1].inner[0] ^= Vec512::mask16(color as Vec512Mask16, bits0);
            tables[1].inner[1] ^= Vec512::mask16((color >> 32) as Vec512Mask16, bits1);
        };

        update_tables(&mut self.attack_tables, closest, closest);
        update_tables(&mut self.xray_tables, closest, second_closest);
        update_tables(&mut self.xray_tables, second_closest & !closest,  closest);
    }

    #[inline]
    fn add_attacks(&mut self, sq: Square, piece: Piece, color: Color, index: PieceIndex) {
        let piece_mask = Vec512::splat16(index.into_mask().into_inner());
        let (ray_coords, ray_valid) = geometry::superpiece_rays(sq);
        let ray_places = Vec512::permute8(ray_coords, self.board.inner);
        let perm = geometry::superpiece_inv_rays(sq);

        let blockers = ray_places.nonzero8();
        let closest = geometry::superpiece_attacks(blockers, ray_valid);
        let second_closest = geometry::superpiece_attacks(blockers & !closest, ray_valid) & !closest;

        let update_tables = |tables: &mut [Wordboard; Color::COUNT], raymask: u64| {
            let attacker_mask = raymask & geometry::attack_mask(piece, color);
            let add_mask = Vec512::mask_bitshuffle(!perm.msb8(), Vec512::splat64(attacker_mask), perm);

            tables[color].inner[0] |= Vec512::mask16(add_mask as Vec512Mask16, piece_mask);
            tables[color].inner[1] |= Vec512::mask16((add_mask >> 32) as Vec512Mask16, piece_mask);
        };

        update_tables(&mut self.attack_tables, closest);
        update_tables(&mut self.xray_tables, second_closest);
    }

    #[inline]
    fn remove_attacks(&mut self, color: Color, index: PieceIndex) {
        let piece_mask = Vec512::splat16(!index.into_mask().into_inner());
        self.attack_tables[color].inner[0] &= piece_mask;
        self.attack_tables[color].inner[1] &= piece_mask;
        self.xray_tables[color].inner[0] &= piece_mask;
        self.xray_tables[color].inner[1] &= piece_mask;
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn xor_piece(&mut self, sq: Square, piece: Piece, color: Color) {
        let value = ZOBRIST.piece(sq, piece, color);

        self.hash ^= value;
        match piece {
            Piece::Pawn => self.pawn_hash ^= value,
            Piece::Knight | Piece::Bishop => self.minor_hash ^= value,
            Piece::Rook | Piece::Queen => self.major_hash ^= value,
            Piece::King => {
                self.minor_hash ^= value;
                self.major_hash ^= value;
            }
        }
    }

    #[inline]
    fn set_castle_rights(&mut self, color: Color, short: bool, file: Option<File>) {
        let rights = if short {
            &mut self.castle_rights[color].short
        } else {
            &mut self.castle_rights[color].long
        };

        if let Some(prev) = core::mem::replace(rights, file) {
            self.hash ^= ZOBRIST.castle_rights(prev, color);
        }

        if let Some(file) = file {
            self.hash ^= ZOBRIST.castle_rights(file, color);
        }
    }

    #[inline]
    fn set_en_passant(&mut self, file: Option<File>) {
        if let Some(prev) = core::mem::replace(&mut self.en_passant, file) {
            self.hash ^= ZOBRIST.en_passant(prev);
        }

        if let Some(file) = file {
            self.hash ^= ZOBRIST.en_passant(file);
        }
    }

    #[inline]
    fn toggle_stm(&mut self) {
        self.stm = !self.stm;
        self.hash ^= ZOBRIST.stm();
    }
}


impl Deref for Board {
    type Target = Byteboard;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.board
    }
}