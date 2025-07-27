use cherry_chess::*;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone, Default)]
pub struct EvalData {
    attacks: [Bitboard; Color::COUNT],
    pawn_attacks: [Bitboard; Color::COUNT],
    mobility_area: [Bitboard; Color::COUNT],
    semiopen_files: [Bitboard; Color::COUNT],
    open_files: Bitboard
}

impl EvalData {
    pub fn get(board: &Board) -> EvalData {
        let mut w_attacks = Bitboard::EMPTY;
        let mut b_attacks = Bitboard::EMPTY;
        let mut w_pawn_attacks = Bitboard::EMPTY;
        let mut b_pawn_attacks = Bitboard::EMPTY;

        let (w_pinned, b_pinned) = (board.pinned(Color::White), board.pinned(Color::Black));
        let w_pawns = board.color_pieces(Piece::Pawn, Color::White);
        let b_pawns = board.color_pieces(Piece::Pawn, Color::Black);
        let blockers = board.occupied();

        for sq in w_pawns & !w_pinned {
            let attacks = pawn_attacks(sq, Color::White);
            w_pawn_attacks |= attacks;
            w_attacks |= attacks;
        }

        for sq in b_pawns & !b_pinned {
            let attacks = pawn_attacks(sq, Color::Black);
            b_pawn_attacks |= attacks;
            b_attacks |= attacks;
        }

        macro_rules! attacks {
            ($piece:expr, $attacks:ident) => {
                for sq in board.color_pieces($piece, Color::White) & !w_pinned {
                    w_attacks |= $attacks(sq);
                }

                for sq in board.color_pieces($piece, Color::Black) & !b_pinned {
                    b_attacks |= $attacks(sq);
                }
            }
        }

        macro_rules! slider_attacks {
            ($piece:expr, $attacks:ident) => {
                for sq in board.color_pieces($piece, Color::White) & !w_pinned {
                    w_attacks |= $attacks(sq, blockers);
                }

                for sq in board.color_pieces($piece, Color::Black) & !b_pinned {
                    b_attacks |= $attacks(sq, blockers);
                }
            }
        }

        attacks!(Piece::Knight, knight_moves);
        attacks!(Piece::King, king_moves);
        slider_attacks!(Piece::Bishop, bishop_moves);
        slider_attacks!(Piece::Rook, rook_moves);
        slider_attacks!(Piece::Queen, queen_moves);

        let mut w_semiopen_files = Bitboard::EMPTY;
        let mut b_semiopen_files = Bitboard::EMPTY;
        let mut open_files = Bitboard::EMPTY;
        let pawns = board.pieces(Piece::Pawn);

        for &file in &File::ALL {
            let bb = file.bitboard();

            if pawns.is_disjoint(bb) {
                open_files |= bb;
            }

            if w_pawns.is_disjoint(bb) && !(b_pawns & bb).is_empty() {
                w_semiopen_files |= bb;
            }

            if b_pawns.is_disjoint(bb) && !(w_pawns & bb).is_empty() {
                b_semiopen_files |= bb
            }
        }

        let w_pawn_advances = w_pawns.shift::<Up>(1) & !blockers;
        let b_pawn_advances = b_pawns.shift::<Down>(1) & !blockers;
        let w_blocked_pawns = w_pawns & !w_pawn_advances.shift::<Down>(1);
        let b_blocked_pawns = b_pawns & !b_pawn_advances.shift::<Up>(1);

        EvalData {
            attacks: [w_attacks, b_attacks],
            pawn_attacks: [w_pawn_attacks, b_pawn_attacks],
            mobility_area: [
                !(b_pawn_attacks | w_blocked_pawns),
                !(w_pawn_attacks | b_blocked_pawns)
            ],
            semiopen_files: [w_semiopen_files, b_semiopen_files],
            open_files
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn attacks(&self, color: Color) -> Bitboard {
        self.attacks[color as usize]
    }

    #[inline]
    pub fn pawn_attacks(&self, color: Color) -> Bitboard {
        self.pawn_attacks[color as usize]
    }

    #[inline]
    pub fn mobility_area(&self, color: Color) -> Bitboard {
        self.mobility_area[color as usize]
    }

    #[inline]
    pub fn semiopen_files(&self, color: Color) -> Bitboard {
        self.semiopen_files[color as usize]
    }

    #[inline]
    pub fn open_files(&self) -> Bitboard {
        self.open_files
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Evaluator {
    weights: EvalWeights,
}

impl Evaluator {
    pub fn eval(&self, board: &Board) -> Score {
        let phase = calc_phase(board);
        let stm = board.stm().sign();

        let data = EvalData::get(board);
        let score = self.eval_psqt(board)
            + self.eval_mobility(board, &data)
            + self.eval_threats(board, &data)
            + self.eval_space(board, &data)
            + self.eval_other(board, &data)
            + self.eval_pawns(board);

        score.scale(phase) * stm
    }

    /*----------------------------------------------------------------*/
    
    fn eval_psqt(&self, board: &Board) -> T {
        let mut score = T::ZERO;
        
        macro_rules! psqt {
            ($piece:expr, $value:expr, $table:expr) => {
                for sq in board.pieces($piece) {
                    if board.colors(Color::White).has(sq) {
                        score += $value + $table[sq as usize];
                    } else {
                        score -= $value + $table[sq.flip_rank() as usize];
                    }
                }
            }
        }

        psqt!(Piece::Pawn, self.weights.pawn_value, self.weights.pawn_psqt);
        psqt!(Piece::Knight, self.weights.knight_value, self.weights.knight_psqt);
        psqt!(Piece::Bishop, self.weights.bishop_value, self.weights.bishop_psqt);
        psqt!(Piece::Rook, self.weights.rook_value, self.weights.rook_psqt);
        psqt!(Piece::Queen, self.weights.queen_value, self.weights.queen_psqt);
        psqt!(Piece::King, T::ZERO, self.weights.king_psqt);

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_mobility(&self, board: &Board, data: &EvalData) -> T {
        let mut score = T::ZERO;
        let (w_pinned, b_pinned) = (
            board.pinned(Color::White),
            board.pinned(Color::Black)
        );
        let (w_safe, b_safe) = (
            data.mobility_area(Color::White),
            data.mobility_area(Color::Black)
        );
        let (w_king, b_king) = (board.king(Color::White), board.king(Color::Black));
        let blockers = board.occupied();

        //pinned knights can't move so no need to calculate their mobility
        for sq in board.color_pieces(Piece::Knight, Color::White) & !w_pinned {
            let attacks = knight_moves(sq) & w_safe;
            let index = attacks.popcnt();

            score += self.weights.knight_mobility[index];
        }

        for sq in board.color_pieces(Piece::Knight, Color::Black) & !b_pinned {
            let attacks = knight_moves(sq) & b_safe;
            let index = attacks.popcnt();

            score -= self.weights.knight_mobility[index];
        }

        macro_rules! slider_mobility {
            ($piece:expr, $attacks:ident, $table:expr) => {
                for sq in board.color_pieces($piece, Color::White) {
                    let attacks = if w_pinned.has(sq) {
                        $attacks(sq, blockers) & w_safe & line(w_king, sq)
                    } else {
                        $attacks(sq, blockers) & w_safe
                    };

                    score += $table[attacks.popcnt()];
                }

                for sq in board.color_pieces($piece, Color::Black) {
                    let attacks = if b_pinned.has(sq) {
                        $attacks(sq, blockers) & b_safe & line(b_king, sq)
                    } else {
                        $attacks(sq, blockers) & b_safe
                    };

                    score -= $table[attacks.popcnt()];
                }
            }
        }

        slider_mobility!(Piece::Bishop, bishop_moves, self.weights.bishop_mobility);
        slider_mobility!(Piece::Rook, rook_moves, self.weights.rook_mobility);
        slider_mobility!(Piece::Queen, queen_moves, self.weights.queen_mobility);

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_pawns(&self, board: &Board) -> T {
        let mut score = T::ZERO;

        for &color in &Color::ALL {
            let our_pawns = board.color_pieces(Piece::Pawn, color);
            let their_pawns = board.color_pieces(Piece::Pawn, !color);
            let sign = color.sign();

            for pawn in our_pawns {
                let (file, rank) = (pawn.file(), pawn.rank());
                let (file, adjacent) = (file.bitboard(), file.adjacent());
                let (above, below) = (
                    rank.relative_to(color).above().relative_to(color),
                    rank.relative_to(color).below().relative_to(color),
                );
                let pass_mask = above & (file | adjacent);
                let backwards_mask = below & adjacent;

                let doubled = (file & our_pawns).popcnt() > 1;
                let passed = (their_pawns & pass_mask).is_empty();
                let backwards = (our_pawns & backwards_mask).is_empty() && !passed;
                let isolated = (adjacent & our_pawns).is_empty();
                let phalanx = !(adjacent & our_pawns & rank.bitboard()).is_empty();
                let support = pawn_attacks(pawn, !color) & our_pawns;

                score += self.weights.doubled_pawn * sign * doubled as i16;
                score += self.weights.passed_pawn[rank.relative_to(color) as usize] * sign * passed as i16;
                score += self.weights.backwards_pawn * sign * backwards as i16;
                score += self.weights.isolated_pawn * sign * isolated as i16;

                if !support.is_empty() || phalanx {
                    let value = self.weights.connected_pawns[rank.relative_to(color) as usize] * (1 + phalanx as i16)
                        + self.weights.supported_pawn * support.popcnt() as i16;

                    score += value * sign;
                }
            }
        }

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_threats(&self, board: &Board, data: &EvalData) -> T {
        let mut score = T::ZERO;
        let blockers = board.occupied();

        for &color in &Color::ALL {
            let their_king = king_zone(board.king(!color), !color);
            let (their_minors, their_majors) = (board.color_minors(!color), board.color_majors(!color));
            let diag_blockers = blockers & !board.color_diag_sliders(color);
            let orth_blockers = blockers & !board.color_orth_sliders(color);
            let our_pawn_attacks = data.pawn_attacks(color);
            let our_pinned = board.pinned(color);
            let mut attack_units = 0;
            let sign = color.sign();

            score += self.weights.pawn_minor_threat * sign * (our_pawn_attacks & their_minors).popcnt() as i16;
            score += self.weights.pawn_major_threat * sign * (our_pawn_attacks & their_majors).popcnt() as i16;

            for sq in board.color_pieces(Piece::Knight, color) & !our_pinned {
                let attacks = knight_moves(sq);

                score += self.weights.minor_major_threat * sign * (attacks & their_majors).popcnt() as i16;
                attack_units += KNIGHT_ATTACK * (attacks & their_king).popcnt() as u8;
            }

            for sq in board.color_pieces(Piece::Bishop, color) & !our_pinned {
                let attacks = bishop_moves(sq, diag_blockers);

                score += self.weights.minor_major_threat * sign * (attacks & their_majors).popcnt() as i16;
                attack_units += BISHOP_ATTACK * (attacks & their_king).popcnt() as u8;
            }

            for sq in board.color_pieces(Piece::Rook, color) & !our_pinned {
                let attacks = rook_moves(sq, orth_blockers);
                attack_units += ROOK_ATTACK * (attacks & their_king).popcnt() as u8;
            }

            for sq in board.color_pieces(Piece::Queen, color) & !our_pinned {
                let attacks = bishop_moves(sq, diag_blockers) | rook_moves(sq, orth_blockers);
                attack_units += QUEEN_ATTACK * (attacks & their_king).popcnt() as u8;
            }

            let danger = KING_DANGER[attack_units.min(99) as usize];
            score -= T(danger, danger) * sign;
        }

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_space(&self, board: &Board, data: &EvalData) -> T {
        let mut score = T::ZERO;

        for &color in &Color::ALL {
            let sign = color.sign();
            let our_uncontested = !data.attacks(!color) & (
                data.attacks(color) | board.colors(color)
            ) & Bitboard::BIG_CENTER;

            score += self.weights.center_control * sign * our_uncontested.popcnt() as i16;
            let our_pawns = board.color_pieces(Piece::Pawn, color);
            let our_knights = board.color_pieces(Piece::Knight, color);
            let their_pawns = board.color_pieces(Piece::Pawn, !color);
            let rank_mask = Bitboard(0x0000FFFFFF000000).relative_to(color);

            for piece in rank_mask & (our_knights | board.color_pieces(Piece::Bishop, color)) {
                let (file, rank) = (piece.file(), piece.rank());
                let outpost_mask = rank.above() & file.adjacent();

                if (outpost_mask & their_pawns).is_empty() {
                    let defended = !(pawn_attacks(piece, !color) & our_pawns).is_empty();
                    let table = if our_knights.has(piece) {
                        &self.weights.knight_outpost
                    } else {
                        &self.weights.bishop_outpost
                    };

                    score += table[defended as usize] * sign;
                }
            }
        }

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_other(&self, board: &Board, data: &EvalData) -> T {
        let mut score = T::ZERO;

        for &color in &Color::ALL {
            let our_pawns = board.color_pieces(Piece::Pawn, color);
            let our_bishops = board.color_pieces(Piece::Bishop, color);
            let sign = color.sign();

            if our_bishops.popcnt() > 1 && !(
                our_bishops.is_subset(Bitboard::LIGHT_SQUARES) || our_bishops.is_subset(Bitboard::DARK_SQUARES)
            ) {
                score += self.weights.bishop_pair * sign;
            }

            let up_offset = sign as i8;
            for knight in board.color_pieces(Piece::Knight, color) {
                if knight.try_offset(0, up_offset).is_some_and(|sq| our_pawns.has(sq)) {
                    score += self.weights.knight_behind_pawn * sign;
                }
            }

            for bishop in our_bishops {
                if bishop.try_offset(0, up_offset).is_some_and(|sq| our_pawns.has(sq)) {
                    score += self.weights.bishop_behind_pawn * sign;
                }
            }

            let semiopen_files = data.semiopen_files(color);
            for rook in board.color_pieces(Piece::Rook, color) {
                if semiopen_files.has(rook) {
                    score += self.weights.rook_semiopen_file * sign;
                }

                if data.open_files().has(rook) {
                    score += self.weights.rook_open_file * sign;
                }
            }

            for queen in board.color_pieces(Piece::Queen, color) {
                if semiopen_files.has(queen) {
                    score += self.weights.queen_semiopen_file * sign;
                }

                if data.open_files().has(queen) {
                    score += self.weights.queen_open_file * sign;
                }
            }
        }

        score
    }
}

impl Default for Evaluator {
    #[inline]
    fn default() -> Self {
        Evaluator { weights: EvalWeights::default() }
    }
}