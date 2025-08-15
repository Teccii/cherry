mod builder;
mod move_gen;
mod parse;
mod print;
mod sanity;
mod attacks;

pub use builder::*;
pub use parse::*;

/*----------------------------------------------------------------*/

use crate::*;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BoardStatus {
    Draw,
    Checkmate,
    Ongoing
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CastleRights {
    pub short: Option<File>,
    pub long: Option<File>,
}

impl CastleRights {
    pub const EMPTY: CastleRights = CastleRights {
        short: None,
        long: None
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Board {
    colors: [Bitboard; Color::COUNT],
    pieces: [Bitboard; Piece::COUNT],
    castle_rights: [CastleRights; Color::COUNT],
    pinned: Bitboard,
    checkers: Bitboard,
    en_passant: Option<File>,
    fullmove_count: u16,
    halfmove_clock: u8,
    minor_hash: u64,
    major_hash: u64,
    pawn_hash: u64,
    hash: u64,
    stm: Color,
}

impl Board {
    #[inline]
    pub const fn occupied(&self) -> Bitboard {
        self.colors[0].union(self.colors[1])
    }

    #[inline]
    pub const fn colors(&self, color: Color) -> Bitboard {
        self.colors[color as usize]
    }

    #[inline]
    pub const fn pieces(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize]
    }

    #[inline]
    pub const fn color_pieces(&self, piece: Piece, color: Color) -> Bitboard {
        self.colors(color).intersection(self.pieces(piece))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn minors(&self) -> Bitboard {
        self.pieces(Piece::Knight).union(self.pieces(Piece::Bishop))
    }

    #[inline]
    pub const fn color_minors(&self, color: Color) -> Bitboard {
        self.colors(color).intersection(self.minors())
    }

    #[inline]
    pub const fn majors(&self) -> Bitboard {
        self.pieces(Piece::Rook).union(self.pieces(Piece::Queen))
    }

    #[inline]
    pub const fn color_majors(&self, color: Color) -> Bitboard {
        self.colors(color).intersection(self.majors())
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn diag_sliders(&self) -> Bitboard {
        self.pieces(Piece::Bishop).union(self.pieces(Piece::Queen))
    }

    #[inline]
    pub const fn color_diag_sliders(&self, color: Color) -> Bitboard {
        self.colors(color).intersection(self.diag_sliders())
    }

    #[inline]
    pub const fn orth_sliders(&self) -> Bitboard {
        self.pieces(Piece::Rook).union(self.pieces(Piece::Queen))
    }

    #[inline]
    pub const fn color_orth_sliders(&self, color: Color) -> Bitboard {
        self.colors(color).intersection(self.orth_sliders())
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn castle_rights(&self, color: Color) -> CastleRights {
        self.castle_rights[color as usize]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn king(&self, color: Color) -> Square {
        self.color_pieces(Piece::King, color).next_square()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn pinned(&self) -> Bitboard { self.pinned }

    #[inline]
    pub const fn checkers(&self) -> Bitboard { self.checkers }

    #[inline]
    pub const fn en_passant(&self) -> Option<File> { self.en_passant }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn in_check(&self) -> bool { !self.checkers.is_empty() }

    #[inline]
    pub fn ep_square(&self) -> Option<Square> {
        self.en_passant.map(|f|
            Square::new(f, Rank::Sixth.relative_to(self.stm))
        )
    }

    /*----------------------------------------------------------------*/
    
    #[inline]
    pub const fn halfmove_clock(&self) -> u8 { self.halfmove_clock }

    #[inline]
    pub const fn fullmove_count(&self) -> u16 { self.fullmove_count }

    #[inline]
    pub const fn minor_hash(&self) -> u64 { self.minor_hash }

    #[inline]
    pub const fn major_hash(&self) -> u64 { self.major_hash }

    #[inline]
    pub const fn pawn_hash(&self) -> u64 { self.pawn_hash }

    #[inline]
    pub const fn hash(&self) -> u64 { self.hash }

    #[inline]
    pub const fn stm(&self) -> Color { self.stm }

    /*----------------------------------------------------------------*/

    #[inline]
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

    #[inline]
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
        self.checkers = Bitboard::EMPTY;
        self.pinned = Bitboard::EMPTY;

        let (from, to, promotion) = (mv.from(), mv.to(), mv.promotion());
        let moved = self.piece_on(from).unwrap();
        let victim = self.piece_on(to);
        let our_king = self.king(self.stm);
        let their_king = self.king(!self.stm);
        let backrank = Rank::First.relative_to(self.stm);
        let their_backrank = Rank::Eighth.relative_to(self.stm);

        // Castling encoded as king captures rook
        let is_castle = self.colors(self.stm).has(to);

        if moved == Piece::Pawn || (victim.is_some() && !is_castle) {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock = (self.halfmove_clock + 1).min(100);
        }
        if self.stm == Color::Black {
            self.fullmove_count = self.fullmove_count.saturating_add(1);
        }

        let mut new_en_passant = None;
        if is_castle {
            let (king, rook) = if from.file() < to.file() {
                (File::G, File::F)
            } else {
                (File::C, File::D)
            };

            self.xor_square(Piece::King, self.stm, from);
            self.xor_square(Piece::Rook, self.stm, to);

            self.xor_square(Piece::King, self.stm, Square::new(king, backrank));
            self.xor_square(Piece::Rook, self.stm, Square::new(rook, backrank));

            self.set_castle_rights(self.stm, None, true);
            self.set_castle_rights(self.stm, None, false);
        } else {
            self.xor_square(moved, self.stm, from);
            self.xor_square(moved, self.stm, to);

            if let Some(victim) = victim {
                self.xor_square(victim, !self.stm, to);

                if to.rank() == their_backrank {
                    let rights = self.castle_rights(!self.stm);
                    let file = to.file();

                    if Some(file) == rights.short {
                        self.set_castle_rights(!self.stm, None, true);
                    } else if Some(file) == rights.long {
                        self.set_castle_rights(!self.stm, None, false);
                    }
                }
            }

            match moved {
                Piece::Knight => self.checkers |= knight_moves(their_king) & to,
                Piece::Pawn => {
                    if let Some(promotion) = promotion {
                        self.xor_square(Piece::Pawn, self.stm, to);
                        self.xor_square(promotion, self.stm, to);

                        if promotion == Piece::Knight {
                            self.checkers |= knight_moves(their_king) & to;
                        }
                    } else {
                        let double_push_from = Rank::Second.relative_to(self.stm).bitboard();
                        let double_push_to = Rank::Fourth.relative_to(self.stm).bitboard();

                        let their_pawns = double_push_to & self.color_pieces(Piece::Pawn, !self.stm) & to.file().adjacent();
                        if double_push_from.has(from) && double_push_to.has(to) && !their_pawns.is_empty() {
                            new_en_passant = Some(to.file());
                        } else if Some(to) == self.ep_square() {
                            let victim_square = Square::new(
                                to.file(),
                                Rank::Fifth.relative_to(self.stm)
                            );
                            self.xor_square(Piece::Pawn, !self.stm, victim_square);
                        }

                        self.checkers |= pawn_attacks(their_king, !self.stm) & to;
                    }
                }
                Piece::King => {
                    self.set_castle_rights(self.stm, None, true);
                    self.set_castle_rights(self.stm, None, false);
                }
                Piece::Rook => if from.rank() == backrank {
                    let rights = self.castle_rights(self.stm);
                    let file = from.file();

                    if Some(file) == rights.short {
                        self.set_castle_rights(self.stm, None, true);
                    } else if Some(file) == rights.long {
                        self.set_castle_rights(self.stm, None, false);
                    }
                }
                _ => {}
            }
        }
        self.set_en_passant(new_en_passant);

        let (diag, orth) = (self.diag_sliders(), self.orth_sliders());
        let our_attackers = self.colors(self.stm) & (
            (bishop_rays(their_king) & diag) | (rook_rays(their_king) & orth)
        );

        let occ = self.occupied();
        for sq in our_attackers {
            let between = between(sq, their_king) & occ;
            match between.popcnt() {
                0 => self.checkers |= sq,
                1 => self.pinned |= between,
                _ => {}
            }
        }

        let their_attackers = self.colors(!self.stm) & (
            (bishop_rays(our_king) & diag) | (rook_rays(our_king) & orth)
        );

        for sq in their_attackers {
            let between = between(sq, our_king) & occ;

            if between.popcnt() == 1 {
                self.pinned |= between & self.colors(self.stm);
            }
        }

        self.toggle_stm();
    }

    pub fn null_move(&self) -> Option<Board> {
        if self.in_check() {
            return None;
        }

        let mut board = self.clone();
        board.halfmove_clock = (board.halfmove_clock + 1).min(100);

        if board.stm == Color::Black {
            board.fullmove_count = board.fullmove_count.saturating_add(1);
        }

        board.set_en_passant(None);
        board.toggle_stm();

        board.pinned = Bitboard::EMPTY;
        let our_king = board.king(board.stm);
        let (diag, orth) = (self.diag_sliders(), self.orth_sliders());
        let their_attackers = board.colors(!board.stm) & (
            (bishop_rays(our_king) & diag) | (rook_rays(our_king) & orth)
        );

        let occ = board.occupied();
        for sq in their_attackers {
            let between = between(sq, our_king) & occ;

            if between.popcnt() == 1 {
                board.pinned |= between;
            }
        }

        let their_king = board.king(!board.stm);
        let our_attackers = board.colors(board.stm) & (
            (bishop_rays(their_king) & diag) | (rook_rays(their_king) & orth)
        );

        for sq in our_attackers {
            let between = between(sq, their_king) & occ;

            if between.popcnt() == 1 {
                board.pinned |= between;
            }
        }

        Some(board)
    }

    pub fn status(&self) -> BoardStatus {
        if self.gen_moves(|_| true) {
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

    #[inline]
    fn xor_square(&mut self, piece: Piece, color: Color, sq: Square) {
        let bb = sq.bitboard();
        self.colors[color as usize] ^= bb;
        self.pieces[piece as usize] ^= bb;

        let zobrist = ZOBRIST.piece(sq, piece, color);
        self.hash ^= zobrist;

        match piece {
            Piece::Pawn => self.pawn_hash ^= zobrist,
            Piece::Knight | Piece::Bishop => self.minor_hash ^= zobrist,
            Piece::Rook | Piece::Queen => self.major_hash ^= zobrist,
            Piece::King => {
                self.minor_hash ^= zobrist;
                self.major_hash ^= zobrist;
            }
        }
    }

    #[inline]
    fn set_castle_rights(&mut self, color: Color, file: Option<File>, short: bool) {
        let rights = if short {
            &mut self.castle_rights[color as usize].short
        } else {
            &mut self.castle_rights[color as usize].long
        };

        if let Some(prev) = ::core::mem::replace(rights, file) {
            self.hash ^= ZOBRIST.castle_rights(prev, color);
        }

        if let Some(file) = file {
            self.hash ^= ZOBRIST.castle_rights(file, color);
        }
    }

    #[inline]
    fn set_en_passant(&mut self, file: Option<File>) {
        if let Some(prev) = ::core::mem::replace(&mut self.en_passant, file) {
            self.hash ^= ZOBRIST.en_passant(prev);
        }

        if let Some(file) = file {
            self.hash ^= ZOBRIST.en_passant(file);
        }
    }

    #[inline]
    fn toggle_stm(&mut self) {
        self.stm = !self.stm;
        self.hash ^= ZOBRIST.stm;
    }
}

impl Default for Board {
    #[inline]
    fn default() -> Self {
        BoardBuilder::startpos().build().unwrap()
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
        write!(f, " {} ", stm)?;

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
                    write!(f, "{}", right)?;
                }

                Ok(())
            };

            write_rights(rights.short, 'k')?;
            write_rights(rights.long, 'q')?;
        }

        if !wrote_castle_rights {
            write!(f, "-")?;
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

/*----------------------------------------------------------------*/

#[cfg(test)]
mod tests {
    use crate::Board;

    fn perft(board: &Board, depth: u8) -> u64 {
        let mut nodes = 0;

        if depth == 0 {
            return 1;
        }

        if depth == 1 {
            board.gen_moves(|moves| {
                nodes += moves.len() as u64;
                false
            });
        } else {
            board.gen_moves(|moves| {
                for mv in moves {
                    let mut board = board.clone();
                    board.make_move(mv);

                    nodes += perft(&board, depth - 1);
                }

                false
            });
        }

        nodes
    }

    macro_rules! perft_test {
        ($name:ident: $board:expr; $($nodes:expr),*) => {
            #[test]
            fn $name() {
                const NODES: &'static [u64] = &[$($nodes),*];

                let board = $board.parse::<Board>().unwrap();
                for (depth, &nodes) in NODES.iter().enumerate() {
                    assert_eq!(perft(&board, depth as u8), nodes);
                }
            }
        }
    }

    perft_test!(
        perft_startpos: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        1,
        20,
        400,
        8902,
        197281,
        4865609,
        119060324
    );

    perft_test!(
        perft_kiwipete:  "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        1,
        48,
        2039,
        97862,
        4085603,
        193690690
    );

    perft_test!(
        perft_pos3: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        1,
        14,
        191,
        2812,
        43238,
        674624,
        11030083,
        178633661
    );

    perft_test!(
        perft_pos4: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        1,
        6,
        264,
        9467,
        422333,
        15833292
    );

    perft_test!(
        perft_pos5: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        1,
        44,
        1486,
        62379,
        2103487,
        89941194
    );

    perft_test!(
        perft_pos6: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        1,
        46,
        2079,
        89890,
        3894594,
        164075551
    );

    perft_test!(
        perft960_position333: "1rqbkrbn/1ppppp1p/1n6/p1N3p1/8/2P4P/PP1PPPP1/1RQBKRBN w FBfb - 0 9";
        1,
        29,
        502,
        14569,
        287739,
        8652810,
        191762235
    );

    perft_test!(
        perft960_position404: "rbbqn1kr/pp2p1pp/6n1/2pp1p2/2P4P/P7/BP1PPPP1/R1BQNNKR w HAha - 0 9";
        1,
        27,
        916,
        25798,
        890435,
        26302461,
        924181432
    );

    perft_test!(
        perft960_position789: "rqbbknr1/1ppp2pp/p5n1/4pp2/P7/1PP5/1Q1PPPPP/R1BBKNRN w GAga - 0 9";
        1,
        24,
        600,
        15347,
        408207,
        11029596,
        308553169
    );

    perft_test!(
        perft960_position726: "rkb2bnr/pp2pppp/2p1n3/3p4/q2P4/5NP1/PPP1PP1P/RKBNQBR1 w Aha - 0 9";
        1,
        29,
        861,
        24504,
        763454,
        22763215,
        731511256
    );
}