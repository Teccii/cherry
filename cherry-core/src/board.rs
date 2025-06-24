use std::{fmt, str::FromStr};
use crate::*;

#[derive(Debug, Copy, Clone)]
pub struct CastleRights {
    pub short: Option<File>,
    pub long: Option<File>,
}

#[derive(Debug, Copy, Clone)]
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
    /*----------------------------------------------------------------*/

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
    pub fn attacks(&self, sq: Square, blockers: Bitboard) -> Bitboard {
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
                    self.set_castle_rights(self.stm, None, true);
                    self.set_castle_rights(self.stm, None, false);
                    self.repetition = 0;
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

        board.set_en_passant(None);
        board.toggle_stm();

        board.checkers = Bitboard::EMPTY;
        board.pinners = Bitboard::EMPTY;
        board.pinned = Bitboard::EMPTY;

        let king = board.king(!self.stm);
        let attackers = knight_moves(king) & board.color_pieces(Piece::Knight, board.stm)
            | bishop_rays(king) & board.color_diag_sliders(board.stm)
            | rook_rays(king) & board.color_orth_sliders(board.stm)
            | pawn_attacks(king, !board.stm) & board.color_pieces(Piece::Pawn, board.stm);

        let occ = board.occupied();
        for sq in attackers {
            let between = between(sq, king) & occ;

            match between.popcnt() {
                0 => board.checkers |= sq.bitboard(),
                1 => {
                    board.pinners |= sq.bitboard();
                    board.pinned |= between;
                },
                _ => ()
            }
        }
        
        if !board.checkers.is_empty() {
            return None;
        }

        Some(board)
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
}