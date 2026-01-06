use core::ops::*;
use std::num::NonZeroU8;

use crate::*;

/*----------------------------------------------------------------*/

mod move_gen;
mod parse;
mod perft;
mod print;
mod startpos;

pub use move_gen::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardStatus {
    Ongoing,
    Draw,
    Checkmate,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct CastleRights {
    pub short: Option<File>,
    pub long: Option<File>,
}

/*
Bit Layout:
- Bits 0-3: File
- Bit 4: Pawn on the left can capture
- Bit 5: Pawn on the right can capture
*/
#[derive(Debug, Copy, Clone)]
pub struct EnPassant {
    bits: NonZeroU8,
}

impl EnPassant {
    #[inline]
    pub fn new(file: File, left: bool, right: bool) -> EnPassant {
        let mut bits = 0;
        bits |= file as u8;
        bits |= (left as u8) << 3;
        bits |= (right as u8) << 4;

        EnPassant {
            bits: NonZeroU8::new(bits).unwrap(),
        }
    }

    #[inline]
    pub fn file(self) -> File {
        File::index((self.bits.get() & 0x7) as usize)
    }

    #[inline]
    pub fn left(self) -> bool {
        (self.bits.get() & 0x8) != 0
    }

    #[inline]
    pub fn right(self) -> bool {
        (self.bits.get() & 0x10) != 0
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Board {
    pub inner: Byteboard,
    attack_table: [Wordboard; Color::COUNT],
    index_to_piece: [IndexToPiece; Color::COUNT],
    index_to_square: [IndexToSquare; Color::COUNT],
    castle_rights: [CastleRights; Color::COUNT],
    en_passant: Option<EnPassant>,
    fullmove_count: u16,
    halfmove_clock: u8,
    pawn_hash: u64,
    minor_hash: u64,
    major_hash: u64,
    white_hash: u64,
    black_hash: u64,
    hash: u64,
    stm: Color,
}

impl Board {
    #[inline]
    pub fn castle_rights(&self, color: Color) -> CastleRights {
        self.castle_rights[color]
    }

    #[inline]
    pub fn attack_table(&self, color: Color) -> &Wordboard {
        &self.attack_table[color]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn en_passant(&self) -> Option<File> {
        self.en_passant.map(|e| e.file())
    }

    #[inline]
    pub fn ep_square(&self) -> Option<Square> {
        self.en_passant()
            .map(|f| Square::new(f, Rank::Sixth.relative_to(self.stm)))
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
    pub const fn white_hash(&self) -> u64 {
        self.white_hash
    }

    #[inline]
    pub const fn black_hash(&self) -> u64 {
        self.black_hash
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
    pub fn color_on(&self, sq: Square) -> Option<Color> {
        self.inner.get(sq).color()
    }

    #[inline]
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.inner.get(sq).piece()
    }

    #[inline]
    pub fn king(&self, color: Color) -> Square {
        self.index_to_square[color][PieceIndex::KING].unwrap()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn checkers(&self) -> PieceMask {
        self.attack_table[!self.stm].get(self.king(self.stm))
    }

    #[inline]
    pub fn in_check(&self) -> bool {
        !self.checkers().is_empty()
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
        let (src, dest) = (mv.src(), mv.dest());
        let (src_place, dest_place) = (self.inner.get(src), self.inner.get(dest));
        let mut new_ep = None;

        #[inline]
        fn castling(
            board: &mut Board,
            src: Square,
            dest: Square,
            src_place: Place,
            dest_place: Place,
            king_dest: File,
            rook_dest: File,
        ) {
            let stm = board.stm;
            let our_backrank = Rank::First.relative_to(stm);
            let king_dest = Square::new(king_dest, our_backrank);
            let rook_dest = Square::new(rook_dest, our_backrank);

            board.remove_piece(src, src_place);
            board.remove_piece(dest, dest_place);
            board.add_piece(king_dest, src_place);
            board.add_piece(rook_dest, dest_place);

            board.index_to_square[stm][src_place.index().unwrap()] = Some(king_dest);
            board.index_to_square[stm][dest_place.index().unwrap()] = Some(rook_dest);

            board.xor_piece(src, Piece::King, stm);
            board.xor_piece(dest, Piece::Rook, stm);
            board.xor_piece(king_dest, Piece::King, stm);
            board.xor_piece(rook_dest, Piece::Rook, stm);

            board.set_halfmove_clock((board.halfmove_clock + 1).min(100));
            board.set_castle_rights(stm, true, None);
            board.set_castle_rights(stm, false, None);
        }

        #[inline]
        fn capture_promotion(
            board: &mut Board,
            src: Square,
            dest: Square,
            src_place: Place,
            dest_place: Place,
            promotion: Piece,
        ) {
            let stm = board.stm;
            let src_index = src_place.index().unwrap();
            let dest_index = dest_place.index().unwrap();
            let new_place = Place::from_piece(promotion, stm, src_index);

            board.remove_piece(src, src_place);
            board.change_piece(dest, dest_place, new_place);

            board.index_to_piece[stm][src_index] = Some(promotion);
            board.index_to_piece[!stm][dest_index] = None;
            board.index_to_square[stm][src_index] = Some(dest);
            board.index_to_square[!stm][dest_index] = None;

            board.xor_piece(src, Piece::Pawn, stm);
            board.xor_piece(dest, dest_place.piece().unwrap(), !stm);
            board.xor_piece(dest, promotion, stm);

            board.set_halfmove_clock(0);
            check_castle_rights(board, !stm, dest);
        }

        #[inline]
        fn promotion(
            board: &mut Board,
            src: Square,
            dest: Square,
            src_place: Place,
            promotion: Piece,
        ) {
            let stm = board.stm;
            let src_index = src_place.index().unwrap();
            let new_place = Place::from_piece(promotion, stm, src_index);

            board.remove_piece(src, src_place);
            board.add_piece(dest, new_place);

            board.index_to_piece[stm][src_index] = Some(promotion);
            board.index_to_square[stm][src_index] = Some(dest);

            board.xor_piece(src, Piece::Pawn, stm);
            board.xor_piece(dest, promotion, stm);

            board.set_halfmove_clock(0);
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
                let src_piece = src_place.piece().unwrap();

                self.remove_piece(src, src_place);
                self.add_piece(dest, src_place);

                self.index_to_square[stm][src_place.index().unwrap()] = Some(dest);

                self.xor_piece(src, src_piece, stm);
                self.xor_piece(dest, src_piece, stm);

                if src_piece != Piece::Pawn {
                    self.set_halfmove_clock((self.halfmove_clock + 1).min(100));
                } else {
                    self.set_halfmove_clock(0);
                }

                match src_piece {
                    Piece::Rook => check_castle_rights(self, stm, src),
                    Piece::King => {
                        self.set_castle_rights(stm, true, None);
                        self.set_castle_rights(stm, false, None);
                    }
                    _ => {}
                }
            }
            MoveFlag::DoublePush => {
                let stm = self.stm;
                let src_piece = src_place.piece().unwrap();

                self.remove_piece(src, src_place);
                self.add_piece(dest, src_place);

                self.index_to_square[stm][src_place.index().unwrap()] = Some(dest);

                self.xor_piece(src, src_piece, stm);
                self.xor_piece(dest, src_piece, stm);

                let their_pawns = self.index_to_piece[!stm].mask_eq(Piece::Pawn);
                let their_attacks = self.attack_table(!stm).get(src.offset(0, stm.sign() as i8));

                if !(their_pawns & their_attacks).is_empty() {
                    new_ep = Some(src.offset(0, stm.sign() as i8));
                }

                self.set_halfmove_clock(0);
            }
            MoveFlag::Capture => {
                let stm = self.stm;
                let src_piece = src_place.piece().unwrap();
                let src_index = src_place.index().unwrap();
                let dest_index = dest_place.index().unwrap();

                self.remove_piece(src, src_place);
                self.change_piece(dest, dest_place, src_place);

                self.index_to_square[stm][src_index] = Some(dest);
                self.index_to_square[!stm][dest_index] = None;
                self.index_to_piece[!stm][dest_index] = None;

                self.xor_piece(src, src_piece, stm);
                self.xor_piece(dest, src_piece, stm);
                self.xor_piece(dest, dest_place.piece().unwrap(), !stm);

                self.set_halfmove_clock(0);

                match src_piece {
                    Piece::Rook => check_castle_rights(self, stm, src),
                    Piece::King => {
                        self.set_castle_rights(stm, true, None);
                        self.set_castle_rights(stm, false, None);
                    }
                    _ => {}
                }

                check_castle_rights(self, !stm, dest);
            }
            MoveFlag::EnPassant => {
                let stm = self.stm;
                let ep_sq = Square::new(dest.file(), src.rank());
                let ep_place = self.get(ep_sq);
                let ep_index = ep_place.index().unwrap();

                self.remove_piece(src, src_place);
                self.remove_piece(ep_sq, ep_place);
                self.add_piece(dest, src_place);

                self.xor_piece(src, Piece::Pawn, stm);
                self.xor_piece(ep_sq, Piece::Pawn, !stm);
                self.xor_piece(dest, Piece::Pawn, stm);

                self.index_to_square[stm][src_place.index().unwrap()] = Some(dest);
                self.index_to_piece[!stm][ep_index] = None;
                self.index_to_square[!stm][ep_index] = None;

                self.set_halfmove_clock(0);
            }
            MoveFlag::ShortCastling =>
                castling(self, src, dest, src_place, dest_place, File::G, File::F),
            MoveFlag::LongCastling =>
                castling(self, src, dest, src_place, dest_place, File::C, File::D),
            MoveFlag::PromotionQueen => promotion(self, src, dest, src_place, Piece::Queen),
            MoveFlag::PromotionRook => promotion(self, src, dest, src_place, Piece::Rook),
            MoveFlag::PromotionBishop => promotion(self, src, dest, src_place, Piece::Bishop),
            MoveFlag::PromotionKnight => promotion(self, src, dest, src_place, Piece::Knight),
            MoveFlag::CapturePromotionQueen =>
                capture_promotion(self, src, dest, src_place, dest_place, Piece::Queen),
            MoveFlag::CapturePromotionRook =>
                capture_promotion(self, src, dest, src_place, dest_place, Piece::Rook),
            MoveFlag::CapturePromotionBishop =>
                capture_promotion(self, src, dest, src_place, dest_place, Piece::Bishop),
            MoveFlag::CapturePromotionKnight =>
                capture_promotion(self, src, dest, src_place, dest_place, Piece::Knight),
        }

        if self.stm == Color::Black {
            self.fullmove_count += 1;
        }

        self.toggle_stm();
        self.calc_ep(new_ep);
    }

    pub fn null_move(&mut self) -> bool {
        if self.in_check() {
            return false;
        }

        self.set_halfmove_clock((self.halfmove_clock + 1).min(100));
        if self.stm == Color::Black {
            self.fullmove_count += 1;
        }

        self.set_en_passant(None);
        self.toggle_stm();

        true
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn calc_hashes(&mut self) {
        self.hash = 0;
        self.pawn_hash = 0;
        self.minor_hash = 0;
        self.major_hash = 0;
        self.white_hash = 0;
        self.black_hash = 0;

        for sq in self.occupied() {
            let piece = self.piece_on(sq).unwrap();
            let color = self.color_on(sq).unwrap();
            self.xor_piece(sq, piece, color);
        }

        if let Some(file) = self.en_passant() {
            self.hash ^= ZOBRIST.en_passant(file);
        }

        for &color in &Color::ALL {
            let rights = self.castle_rights(color);

            if let Some(file) = rights.short {
                self.hash ^= ZOBRIST.castle_rights(file, color);
            }

            if let Some(file) = rights.long {
                self.hash ^= ZOBRIST.castle_rights(file, color);
            }
        }

        self.hash ^= ZOBRIST.halfmove_clock(Zobrist::hm_bucket(self.halfmove_clock));

        if self.stm == Color::Black {
            self.hash ^= ZOBRIST.stm;
        }
    }

    #[inline]
    fn calc_ep(&mut self, new_ep: Option<Square>) {
        let Some(ep_sq) = new_ep else {
            self.set_en_passant(None);
            return;
        };

        let ep_file = ep_sq.file();
        let ep_victim = ep_sq.offset(0, -self.stm.sign() as i8);
        let our_pawns = self.index_to_piece[self.stm].mask_eq(Piece::Pawn);
        let our_attacks = self.attack_table(self.stm).get(ep_sq);
        let mut left = false;
        let mut right = false;

        let king = self.king(self.stm);
        let (ray_perm, ray_valid) = ray_perm(king);
        for index in our_pawns & our_attacks {
            let ep_src = self.index_to_square[self.stm][index].unwrap();
            let pawn_place = self.inner.get(ep_src);
            let mut ep_board = self.inner.clone();
            ep_board.set(ep_src, Place::EMPTY);
            ep_board.set(ep_victim, Place::EMPTY);
            ep_board.set(ep_sq, pawn_place);

            let ray_places = ep_board.permute(ray_perm).mask(Mask8x64::from(ray_valid));
            let their_color = match self.stm {
                Color::White => ray_places.msb(),
                Color::Black => !ray_places.msb(),
            };
            let blockers = ray_places.nonzero().to_bitmask();
            let attackers = ray_attackers(ray_places);
            let closest = extend_bitrays(blockers, ray_valid) & blockers;
            let their_attackers = their_color & attackers & closest;
            if their_attackers.to_bitmask() == 0 {
                if ep_src.file() < ep_file {
                    left = true;
                } else {
                    right = true;
                }
            }
        }

        self.set_en_passant((left || right).then(|| EnPassant::new(ep_file, left, right)));
    }

    #[inline]
    fn calc_attacks(&mut self) {
        let mut attacks = [[PieceMask::EMPTY; Square::COUNT]; Color::COUNT];

        for &sq in &Square::ALL {
            let (ray_perm, ray_valid) = ray_perm(sq);
            let ray_places = self.inner.permute(ray_perm).mask(Mask8x64::from(ray_valid));

            let color = ray_places.msb();
            let blockers = ray_places.nonzero().to_bitmask();
            let attackers = ray_attackers(ray_places);
            let closest = extend_bitrays(blockers, ray_valid) & blockers;

            let white_index = unsafe { u8x16::load(self.index_to_square[Color::White].0.as_ptr()) };
            let black_index = unsafe { u8x16::load(self.index_to_square[Color::Black].0.as_ptr()) };
            let white_attackers = !color & attackers & closest;
            let black_attackers = color & attackers & closest;

            let white_coords = ray_perm.compress(white_attackers).extract16::<0>();
            let black_coords = ray_perm.compress(black_attackers).extract16::<0>();
            let white_count = white_attackers.to_bitmask().count_ones() as usize;
            let black_count = black_attackers.to_bitmask().count_ones() as usize;
            let white_mask = white_index.findset(white_coords, white_count);
            let black_mask = black_index.findset(black_coords, black_count);

            attacks[Color::White][sq] = PieceMask(white_mask);
            attacks[Color::Black][sq] = PieceMask(black_mask);
        }

        self.attack_table = unsafe { core::mem::transmute(attacks) };
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn add_piece(&mut self, sq: Square, new_place: Place) {
        self.inner.set(sq, new_place);

        self.update_sliders(sq);
        self.add_attacks(sq, new_place);
    }

    #[inline]
    fn remove_piece(&mut self, sq: Square, old_place: Place) {
        self.remove_attacks(old_place);
        self.update_sliders(sq);

        self.inner.set(sq, Place::EMPTY);
    }

    #[inline]
    fn change_piece(&mut self, sq: Square, old_place: Place, new_place: Place) {
        self.inner.set(sq, new_place);

        self.remove_attacks(old_place);
        self.add_attacks(sq, new_place);
    }

    #[inline]
    fn update_sliders(&mut self, sq: Square) {
        let (ray_perm, ray_valid) = ray_perm(sq);
        let (inv_perm, inv_valid) = inv_perm(sq);
        let ray_places = self.inner.permute(ray_perm).mask(Mask8x64::from(ray_valid));

        let blockers = ray_places.nonzero().to_bitmask();
        let sliders = ray_sliders(ray_places);
        let bitrays = extend_bitrays(blockers, ray_valid) & NON_HORSE_ATTACK_MASK;

        let slider_rays = ray_places.mask(sliders & bitrays).extend_rays();
        let updates = slider_rays
            .flip_rays()
            .mask(Mask8x64::from(bitrays))
            .permute(inv_perm)
            .mask(Mask8x64::from(inv_valid));
        let update_colors = updates.msb().widen();
        let update_indices = (updates & u8x64::splat(Place::INDEX_MASK)).zero_ext();
        let valid_updates = update_indices.nonzero();

        let updates = u16x64::splat(1).shlv(update_indices).mask(valid_updates);
        self.attack_table[Color::White].0 ^= updates.mask(!update_colors);
        self.attack_table[Color::Black].0 ^= updates.mask(update_colors);
    }

    #[inline]
    fn add_attacks(&mut self, sq: Square, new_place: Place) {
        let (index, piece, color) = (
            new_place.index().unwrap(),
            new_place.piece().unwrap(),
            new_place.color().unwrap(),
        );

        let (ray_perm, ray_valid) = ray_perm(sq);
        let (inv_perm, inv_valid) = inv_perm(sq);
        let ray_places = self.inner.permute(ray_perm).mask(Mask8x64::from(ray_valid));

        let blockers = ray_places.nonzero().to_bitmask();
        let bitrays = extend_bitrays(blockers, ray_valid);
        let valid_bitrays = bitrays & attack_mask(color, piece);

        let add_mask = u8x64::splat(1)
            .mask(Mask8x64::from(valid_bitrays))
            .permute(inv_perm)
            .mask(Mask8x64::from(inv_valid))
            .zero_ext()
            .shlv(u16x64::splat(index.0 as u16));

        self.attack_table[color].0 |= add_mask;
    }

    #[inline]
    fn remove_attacks(&mut self, old_place: Place) {
        let (index, color) = (old_place.index().unwrap(), old_place.color().unwrap());

        self.attack_table[color].0 &= u16x64::splat(!index.into_mask().0);
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

        if piece != Piece::Pawn {
            match color {
                Color::White => self.white_hash ^= value,
                Color::Black => self.black_hash ^= value,
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
    fn set_en_passant(&mut self, ep: Option<EnPassant>) {
        if let Some(prev) = core::mem::replace(&mut self.en_passant, ep) {
            self.hash ^= ZOBRIST.en_passant(prev.file());
        }

        if let Some(ep) = ep {
            self.hash ^= ZOBRIST.en_passant(ep.file());
        }
    }

    #[inline]
    fn set_halfmove_clock(&mut self, hm: u8) {
        let old_bucket = Zobrist::hm_bucket(self.halfmove_clock);
        let new_bucket = Zobrist::hm_bucket(hm);
        self.halfmove_clock = hm;

        if old_bucket != new_bucket {
            self.hash ^= ZOBRIST.halfmove_clock(old_bucket);
            self.hash ^= ZOBRIST.halfmove_clock(new_bucket);
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
        &self.inner
    }
}

impl DerefMut for Board {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
