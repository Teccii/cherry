mod builder;
mod move_gen;
mod parse;

pub use builder::*;
pub use move_gen::*;
pub use parse::*;

/*----------------------------------------------------------------*/

use crate::*;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct CastleRights {
    pub short: Option<File>,
    pub long: Option<File>,
}

impl CastleRights {
    pub const EMPTY: CastleRights = CastleRights { short: None, long: None };
}

#[derive(Copy, Clone)]
pub struct Board {
    colors: [Bitboard; Color::COUNT],
    pieces: [Bitboard; Piece::COUNT],
    castle_rights: [CastleRights; Color::COUNT],
    en_passant: Option<File>,
    checkers: Bitboard,
    pinners: Bitboard,
    pinned: Bitboard,
    fullmove_count: u16,
    halfmove_clock: u8,
    repetition: u8,
    pawn_hash: u64,
    hash: u64,
    stm: Color,
}

impl Board {
    #[inline(always)]
    pub const fn occupied(&self) -> Bitboard {
        Bitboard(self.colors[0].0 | self.colors[1].0)
    }

    #[inline(always)]
    pub const fn colors(&self, color: Color) -> Bitboard {
        self.colors[color as usize]
    }

    #[inline(always)]
    pub const fn pieces(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize]
    }

    #[inline(always)]
    pub const fn color_pieces(&self, piece: Piece, color: Color) -> Bitboard {
        Bitboard(self.colors(color).0 & self.pieces(piece).0)
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn minors(&self) -> Bitboard {
        Bitboard(self.pieces(Piece::Knight).0 | self.pieces(Piece::Bishop).0)
    }

    #[inline(always)]
    pub const fn color_minors(&self, color: Color) -> Bitboard {
        Bitboard(self.colors(color).0 & self.minors().0)
    }

    #[inline(always)]
    pub const fn majors(&self) -> Bitboard {
        Bitboard(self.pieces(Piece::Rook).0 | self.pieces(Piece::Queen).0)
    }

    #[inline(always)]
    pub const fn color_majors(&self, color: Color) -> Bitboard {
        Bitboard(self.colors(color).0 & self.majors().0)
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn diag_sliders(&self) -> Bitboard {
        Bitboard(self.pieces(Piece::Bishop).0 | self.pieces(Piece::Queen).0)
    }

    #[inline(always)]
    pub const fn color_diag_sliders(&self, color: Color) -> Bitboard {
        Bitboard(self.colors(color).0 & self.diag_sliders().0)
    }

    #[inline(always)]
    pub const fn orth_sliders(&self) -> Bitboard {
        Bitboard(self.pieces(Piece::Rook).0 | self.pieces(Piece::Queen).0)
    }

    #[inline(always)]
    pub const fn color_orth_sliders(&self, color: Color) -> Bitboard {
        Bitboard(self.colors(color).0 & self.orth_sliders().0)
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn castle_rights(&self, color: Color) -> CastleRights {
        self.castle_rights[color as usize]
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn king(&self, color: Color) -> Square {
        self.color_pieces(Piece::King, color).next_square()
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn pinned(&self) -> Bitboard { self.pinned }

    #[inline(always)]
    pub const fn pinners(&self) -> Bitboard { self.pinners }

    #[inline(always)]
    pub const fn checkers(&self) -> Bitboard { self.checkers }

    #[inline(always)]
    pub const fn en_passant(&self) -> Option<File> { self.en_passant }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn in_check(&self) -> bool { !self.checkers.is_empty() }

    #[inline(always)]
    pub fn ep_square(&self) -> Option<Square> {
        self.en_passant.map(|f|
            Square::new(f, Rank::Sixth.relative_to(self.stm))
        )
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn halfmove_clock(&self) -> u8 { self.halfmove_clock }

    #[inline(always)]
    pub const fn fullmove_count(&self) -> u16 { self.fullmove_count }

    #[inline(always)]
    pub const fn pawn_hash(&self) -> u64 { self.pawn_hash }

    #[inline(always)]
    pub const fn hash(&self) -> u64 { self.hash }

    #[inline(always)]
    pub const fn stm(&self) -> Color { self.stm }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn attackers(&self, sq: Square, blockers: Bitboard) -> Bitboard {
        knight_moves(sq)                 & self.pieces(Piece::Knight)
            | king_moves(sq)                 & self.pieces(Piece::King)
            | bishop_moves(sq, blockers)     & self.diag_sliders()
            | rook_moves(sq, blockers)       & self.orth_sliders()
            | pawn_attacks(sq, Color::White) & self.color_pieces(Piece::Pawn, Color::Black)
            | pawn_attacks(sq, Color::Black) & self.color_pieces(Piece::Pawn, Color::White)
    }

    #[inline(always)]
    pub fn pawn_attacks(&self, color: Color) -> Bitboard {
        let pawns = self.color_pieces(Piece::Pawn, color);

        match color {
            Color::White => pawns | pawns.shift::<UpLeft>(1) | pawns.shift::<UpRight>(1),
            Color::Black => pawns | pawns.shift::<DownLeft>(1) | pawns.shift::<DownRight>(1),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        let bb = sq.bitboard();

        if self.occupied().is_disjoint(bb) {
            return None;
        }

        if bb.is_subset(self.pieces(Piece::Pawn)) {
            Some(Piece::Pawn)
        } else if bb.is_subset(self.pieces(Piece::Knight)) {
            Some(Piece::Knight)
        } else if bb.is_subset(self.pieces(Piece::Bishop)) {
            Some(Piece::Bishop)
        } else if bb.is_subset(self.pieces(Piece::Rook)) {
            Some(Piece::Rook)
        } else if bb.is_subset(self.pieces(Piece::Queen)) {
            Some(Piece::Queen)
        } else {
            Some(Piece::King)
        }
    }

    #[inline(always)]
    pub fn color_on(&self, sq: Square) -> Option<Color> {
        let bb = sq.bitboard();

        if self.occupied().is_disjoint(bb) {
            return None;
        }

        if bb.is_subset(self.colors(Color::White)) {
            Some(Color::White)
        } else {
            Some(Color::Black)
        }
    }

    /*----------------------------------------------------------------*/

    pub fn make_move(&mut self, mv: Move) {
        let (from, to, promotion, flag) = (mv.from(), mv.to(), mv.promotion(), mv.flag());
        let back_rank = Rank::First.relative_to(self.stm);
        let piece = self.piece_on(from).unwrap();
        let victim = self.piece_on(to);

        if piece == Piece::Pawn || (victim.is_some() && flag != MoveFlag::Castling) {
            self.halfmove_clock = 0;
            self.repetition = 0;
        } else {
            self.halfmove_clock = (self.halfmove_clock + 1).min(100);
            self.repetition = (self.repetition + 1).min(100);
        }

        if self.stm == Color::Black {
            self.fullmove_count += 1;
        }

        let new_ep = None;
        if flag == MoveFlag::Castling {
            let (king, rook) = if from.file() < to.file() {
                (File::G, File::F)
            } else {
                (File::C, File::D)
            };

            self.xor_square(Piece::King, self.stm, from);
            self.xor_square(Piece::Rook, self.stm, to);

            self.xor_square(Piece::King, self.stm, Square::new(king, back_rank));
            self.xor_square(Piece::Rook, self.stm, Square::new(rook, back_rank));

            self.set_castle_rights(self.stm, None, true);
            self.set_castle_rights(self.stm, None, false);

            self.repetition = 0;
        } else if flag == MoveFlag::EnPassant {
            if self.ep_square() == Some(to) {
                let victim_sq = Square::new(to.file(), Rank::Fifth.relative_to(self.stm));

                self.xor_square(Piece::Pawn, !self.stm, victim_sq);
                self.repetition = 0;
            }
        } else {
            self.xor_square(piece, self.stm, from);
            self.xor_square(piece, self.stm, to);

            if let Some(victim) = victim {
                self.xor_square(victim, !self.stm, to);

                if to.rank() == Rank::First.relative_to(!self.stm) {
                    let rights = self.castle_rights(!self.stm);
                    let file = to.file();

                    if rights.short == Some(file) {
                        self.set_castle_rights(!self.stm, None, true);
                        //self.repetition = 0 already handled earlier
                    } else if rights.long == Some(file) {
                        self.set_castle_rights(!self.stm, None, false);
                    }
                }
            }

            match piece {
                Piece::Pawn => {
                    if let Some(promotion) = promotion {
                        self.xor_square(Piece::Pawn, self.stm, to);
                        self.xor_square(promotion, self.stm, to);
                    } else {
                        const DOUBLE_PUSH_FROM: Bitboard = Bitboard(Rank::Second.bitboard().0 | Rank::Seventh.bitboard().0);
                        const DOUBLE_PUSH_TO: Bitboard = Bitboard(Rank::Fourth.bitboard().0 | Rank::Fifth.bitboard().0);

                        if DOUBLE_PUSH_FROM.has(from) && DOUBLE_PUSH_TO.has(to) {
                            self.set_en_passant(Some(to.file()));
                        }
                    }
                },
                Piece::Rook => if from.rank() == back_rank {
                    let rights = self.castle_rights(!self.stm);
                    let file = from.file();

                    if rights.short == Some(file) {
                        self.set_castle_rights(!self.stm, None, true);
                        self.repetition = 0;
                    } else if rights.long == Some(file) {
                        self.set_castle_rights(!self.stm, None, false);
                        self.repetition = 0;
                    }
                },
                Piece::King => {
                    let old_rights = self.castle_rights(self.stm);

                    self.set_castle_rights(self.stm, None, true);
                    self.set_castle_rights(self.stm, None, false);

                    if old_rights.short.is_some() || old_rights.long.is_some() {
                        self.repetition = 0;
                    }
                },
                _ => ()
            }
        }

        self.pinners = Bitboard::EMPTY;
        self.pinned = Bitboard::EMPTY;
        self.checkers = Bitboard::EMPTY;

        let king = self.king(!self.stm);
        let attackers = knight_moves(king) & self.color_pieces(Piece::Knight, self.stm)
            | bishop_rays(king) & self.color_diag_sliders(self.stm)
            | rook_rays(king) & self.color_orth_sliders(self.stm)
            | pawn_attacks(king, !self.stm) & self.color_pieces(Piece::Pawn, self.stm);

        let occ = self.occupied();
        for sq in attackers {
            let between = between(sq, king) & occ;

            match between.popcnt() {
                0 => self.checkers |= sq.bitboard(),
                1 => {
                    self.pinners |= sq.bitboard();
                    self.pinned |= between;
                },
                _ => ()
            }
        }

        self.set_en_passant(new_ep);
        self.toggle_stm();
    }

    pub fn null_move(&self) -> Option<Board> {
        if self.in_check() {
            return None;
        }

        let mut board = self.clone();
        board.halfmove_clock = (board.halfmove_clock + 1).min(100);
        board.repetition = 0;

        if board.stm == Color::Black {
            board.fullmove_count = board.fullmove_count.saturating_add(1);
        }

        board.set_en_passant(None);
        board.toggle_stm();

        board.pinners = Bitboard::EMPTY;
        board.pinned = Bitboard::EMPTY;

        let king = board.king(board.stm);
        let attackers = bishop_rays(king) & self.color_diag_sliders(!self.stm)
            | rook_rays(king) & self.color_orth_sliders(!self.stm);

        let occ = board.occupied();
        for sq in attackers {
            let between = between(sq, king) & occ;

            if between.popcnt() == 1 {
                board.pinners |= sq.bitboard();
                board.pinned |= between;
            }
        }

        Some(board)
    }

    /*----------------------------------------------------------------*/

    pub fn calc_checks(&self, color: Color) -> (Bitboard, Bitboard, Bitboard) {
        let mut checkers = Bitboard::EMPTY;
        let mut pinners = Bitboard::EMPTY;
        let mut pinned = Bitboard::EMPTY;

        let king = self.king(color);
        let attackers = knight_moves(king) & self.color_pieces(Piece::Knight, !color)
            | bishop_rays(king) & self.color_diag_sliders(!color)
            | rook_rays(king) & self.color_orth_sliders(!color)
            | pawn_attacks(king, color) & self.color_pieces(Piece::Pawn, !color);

        let occ = self.occupied();
        for sq in attackers {
            let between = between(sq, king) & occ;

            match between.popcnt() {
                0 => checkers |= sq.bitboard(),
                1 => {
                    pinners |= sq.bitboard();
                    pinned |= between;
                },
                _ => ()
            }
        }

        (checkers, pinners, pinned)
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    fn xor_square(&mut self, piece: Piece, color: Color, sq: Square) {
        let bb = sq.bitboard();
        self.colors[color as usize] ^= bb;
        self.pieces[piece as usize] ^= bb;

        let zobrist = ZOBRIST.piece(sq, piece, color);
        self.hash ^= zobrist;

        if piece == Piece::Pawn {
            self.pawn_hash ^= zobrist;
        }
    }

    #[inline(always)]
    fn set_castle_rights(&mut self, color: Color, file: Option<File>, short: bool) {
        let rights = if short {
            &mut self.castle_rights[color as usize].short
        } else {
            &mut self.castle_rights[color as usize].long
        };

        if let Some(prev) = std::mem::replace(rights, file) {
            self.hash ^= ZOBRIST.castle_rights(prev, color);
        }

        if let Some(file) = file {
            self.hash ^= ZOBRIST.castle_rights(file, color);
        }
    }

    #[inline(always)]
    fn set_en_passant(&mut self, file: Option<File>) {
        if let Some(prev) = std::mem::replace(&mut self.en_passant, file) {
            self.hash ^= ZOBRIST.en_passant(prev);
        }

        if let Some(file) = file {
            self.hash ^= ZOBRIST.en_passant(file);
        }
    }

    #[inline(always)]
    fn toggle_stm(&mut self) {
        self.stm = !self.stm;
        self.hash ^= ZOBRIST.stm;
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn is_sane(&self) -> bool {
        self.halfmove_clock_is_sane()
            && self.fullmove_count_is_sane()
            && self.checkers_is_sane()
            && self.castle_rights_is_sane()
            && self.en_passant_is_sane()
            && self.board_is_sane()
    }

    fn board_is_sane(&self) -> bool {
        macro_rules! soft_assert {
            ($e:expr) => {
                if !$e {
                    return false;
                }
            }
        }

        let mut occupied = Bitboard::EMPTY;

        for &piece in &Piece::ALL {
            let pieces = self.pieces(piece);

            soft_assert!(pieces.is_disjoint(occupied));
            occupied |= pieces;
        }

        soft_assert!(self.colors(Color::White).is_disjoint(self.colors(Color::Black)));
        soft_assert!(occupied == self.occupied());

        for &color in &Color::ALL {
            let colors = self.colors(color);
            let pawn_mask = Rank::First.bitboard() | Rank::Eighth.bitboard();

            soft_assert!(self.pieces.len() <= 16);
            soft_assert!((colors & self.pieces(Piece::King)).popcnt() == 1);
            soft_assert!((colors & self.pieces(Piece::Pawn)).popcnt() <= 8);
            soft_assert!((colors & self.pieces(Piece::Pawn)).is_disjoint(pawn_mask));
        }

        true
    }

    fn en_passant_is_sane(&self) -> bool {
        macro_rules! soft_assert {
            ($e:expr) => {
                if !$e {
                    return false;
                }
            }
        }

        if let Some(ep_file) = self.en_passant {
            let from = Square::new(ep_file, Rank::Seventh.relative_to(self.stm));
            let to = Square::new(ep_file, Rank::Fifth.relative_to(self.stm));
            let ep = Square::new(ep_file, Rank::Sixth.relative_to(self.stm));

            soft_assert!(self.color_pieces(Piece::Pawn, !self.stm).has(to));
            soft_assert!(!self.occupied().has(from));
            soft_assert!(!self.occupied().has(ep));

            let king = self.king(self.stm);
            for checker in self.checkers {
                let ray_through = between(checker, king).has(from);
                soft_assert!(checker == to || ray_through)
            }
        }

        true
    }

    fn castle_rights_is_sane(&self) -> bool {
        macro_rules! soft_assert {
            ($e:expr) => {
                if !$e {
                    return false;
                }
            }
        }

        for &color in &Color::ALL {
            let back_rank = Rank::First.relative_to(color);
            let rights = self.castle_rights(color);
            let rooks = self.color_pieces(Piece::Rook, color);

            if rights.short.is_some() || rights.long.is_some() {
                let king = self.king(color);

                soft_assert!(king.rank() == back_rank);

                if let Some(rook) = rights.short {
                    soft_assert!(rooks.has(Square::new(rook, back_rank)));
                    soft_assert!(king.file() < rook);
                }

                if let Some(rook) = rights.long {
                    soft_assert!(rooks.has(Square::new(rook, back_rank)));
                    soft_assert!(rook < king.file());
                }
            }
        }

        true
    }

    fn checkers_is_sane(&self) -> bool {
        if !self.calc_checks(self.stm).0.is_empty() {
            return false;
        }

        let (checkers, pinners, pinned) = self.calc_checks(!self.stm);

        checkers == self.checkers
            && pinners == self.pinners
            && pinned == self.pinned
            && self.checkers.popcnt() < 2
    }

    #[inline(always)]
    fn halfmove_clock_is_sane(&self) -> bool {
        self.halfmove_clock <= 100
    }

    #[inline(always)]
    fn fullmove_count_is_sane(&self) -> bool {
        self.fullmove_count > 0
    }
}

impl Default for Board {
    #[inline(always)]
    fn default() -> Self {
        BoardBuilder::startpos().build().unwrap()
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &rank in Rank::ALL.iter().rev() {
            for &file in File::ALL.iter() {
                let sq = Square::new(file, rank);

                if !self.occupied().has(sq) {
                    write!(f, " .")?;
                } else {
                    let piece: char = self.piece_on(sq).unwrap().into();

                    if self.colors(Color::White).has(sq) {
                        write!(f, " {}", piece.to_ascii_uppercase())?;
                    } else {
                        write!(f, " {}", piece)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let shredder = f.alternate();

        for &rank in Rank::ALL.iter().rev() {
            let mut empty = 0;

            for &file in &File::ALL {
                let sq = Square::new(file, rank);

                if let Some(piece) = self.piece_on(sq) {
                    if empty > 0 {
                        write!(f, "{}", empty)?;
                        empty = 0;
                    }

                    let mut piece: char = piece.into();
                    if self.color_on(sq).unwrap() == Color::White {
                        piece = piece.to_ascii_uppercase();
                    }

                    write!(f, "{}", piece)?;
                } else {
                    empty += 1;
                }
            }

            if empty > 0 {
                write!(f, "{}", empty)?;
            }

            if rank > Rank::First {
                write!(f, "/")?;
            }
        }

        let stm: char = self.stm.into();
        write!(f, " {}", stm)?;

        let mut wrote_castle_rights = false;
        for &color in &Color::ALL {
            let rights = self.castle_rights(color);
            let mut write_rights = |file: Option<File>, right_char: char| {
                if let Some(file) = file {
                    let mut right = if shredder {
                        file.into()
                    } else {
                        right_char
                    };

                    if color == Color::White {
                        right = right.to_ascii_uppercase();
                    }

                    wrote_castle_rights = true;
                    write!(f, " {}", right)?;
                }

                Ok(())
            };

            write_rights(rights.short, 'k')?;
            write_rights(rights.long, 'q')?;
        }

        if !wrote_castle_rights {
            write!(f, " -")?;
        }

        if let Some(file) = self.en_passant {
            let rank = Rank::Sixth.relative_to(self.stm);
            write!(f, " {}", Square::new(file, rank))?;
        } else {
            write!(f, " -")?;
        }

        write!(f, " {} {}", self.halfmove_clock, self.fullmove_count)
    }
}