use cozy_chess::*;
use crate::*;
use super::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct EvalData {
    king_zone: [BitBoard; Color::NUM],
    not_diag_sliders: [BitBoard; Color::NUM],
    not_orth_sliders: [BitBoard; Color::NUM],
    attacks: [BitBoard; Color::NUM],
    double_attacks: [BitBoard; Color::NUM],
    pawn_attacks: [BitBoard; Color::NUM],
    pawn_double_attacks: [BitBoard; Color::NUM],
    blocked_pawns: [BitBoard; Color::NUM],
    rammed_pawns: [BitBoard; Color::NUM],
    mobility_area: [BitBoard; Color::NUM],
    semiopen_files: [BitBoard; Color::NUM],
    open_files: BitBoard,
}

impl EvalData {
    pub fn calc(board: &Board) -> EvalData {
        let blockers = board.occupied();
        let not_pinned = !board.pinned();
        let w_pawns = board.colored_pieces(Color::White, Piece::Pawn);
        let b_pawns = board.colored_pieces(Color::Black, Piece::Pawn);

        let mut w_attacks = BitBoard::EMPTY;
        let mut b_attacks = BitBoard::EMPTY;
        let mut w_double_attacks = BitBoard::EMPTY;
        let mut b_double_attacks = BitBoard::EMPTY;
        let mut w_pawn_attacks = BitBoard::EMPTY;
        let mut b_pawn_attacks = BitBoard::EMPTY;
        let mut w_pawn_double_attacks = BitBoard::EMPTY;
        let mut b_pawn_double_attacks = BitBoard::EMPTY;

        for sq in w_pawns & not_pinned {
            let attacks = get_pawn_attacks(sq, Color::White);

            w_pawn_double_attacks |= w_pawn_attacks & attacks;
            w_double_attacks |= w_attacks & attacks;
            w_pawn_attacks |= attacks;
            w_attacks |= attacks;
        }

        for sq in b_pawns & not_pinned {
            let attacks = get_pawn_attacks(sq, Color::Black);

            b_pawn_double_attacks |= b_pawn_attacks & attacks;
            b_double_attacks |= b_attacks & attacks;
            b_pawn_attacks |= attacks;
            b_attacks |= attacks;
        }

        macro_rules! calc_pieces {
            ($piece:expr, $attack_fn:ident) => {
                for sq in board.colored_pieces(Color::White, $piece) & not_pinned {
                    let attacks = $attack_fn(sq);
                    w_double_attacks |= w_attacks & attacks;
                    w_attacks |= attacks;
                }
                
                for sq in board.colored_pieces(Color::Black, $piece) & not_pinned {
                    let attacks = $attack_fn(sq);
                    b_double_attacks |= b_attacks & attacks;
                    b_attacks |= attacks;
                }
            }
        }
        
        macro_rules! calc_sliders {
            ($piece:expr, $attack_fn:ident) => {
                for sq in board.colored_pieces(Color::White, $piece) & not_pinned {
                    let attacks = $attack_fn(sq, blockers);
                    w_double_attacks |= w_attacks & attacks;
                    w_attacks |= attacks;
                }
                
                for sq in board.colored_pieces(Color::Black, $piece) & not_pinned {
                    let attacks = $attack_fn(sq, blockers);
                    b_double_attacks |= b_attacks & attacks;
                    b_attacks |= attacks;
                }
            }
        }
        
        calc_pieces!(Piece::Knight, get_knight_moves);
        calc_pieces!(Piece::King, get_king_moves);
        calc_sliders!(Piece::Bishop, get_bishop_moves);
        calc_sliders!(Piece::Rook, get_rook_moves);
        calc_sliders!(Piece::Queen, get_queen_moves);

        let pawns = board.pieces(Piece::Pawn);
        let mut open_files = BitBoard::EMPTY;
        let mut w_semiopen_files = BitBoard::EMPTY;
        let mut b_semiopen_files = BitBoard::EMPTY;

        if pawns.is_empty() || w_pawns.is_empty() || b_pawns.is_empty() {
            if pawns.is_empty() {
                open_files = BitBoard::FULL;
            }

            if w_pawns.is_empty() {
                w_semiopen_files = BitBoard::FULL;
            }

            if b_pawns.is_empty() {
                b_semiopen_files = BitBoard::FULL;
            }
        } else {
            for &file in File::ALL.iter() {
                let bb = file.bitboard();

                if (pawns & bb).is_empty() {
                    open_files |= bb;
                }

                if (w_pawns & bb).is_empty() && !(b_pawns & bb).is_empty() {
                    w_semiopen_files |= bb;
                }

                if (b_pawns & bb).is_empty() && !(w_pawns & bb).is_empty() {
                    b_semiopen_files |= bb;
                }
            }
        }

        let w_pawn_advances = w_pawns.shift::<Up>(1) & !blockers;
        let b_pawn_advances = b_pawns.shift::<Down>(1) & !blockers;
        let w_blocked_pawns = w_pawns & !w_pawn_advances.shift::<Down>(1);
        let b_blocked_pawns = b_pawns & !b_pawn_advances.shift::<Up>(1);
        let w_rammed_pawns = w_pawns & !(w_pawn_advances & !b_pawns).shift::<Down>(1);
        let b_rammed_pawns = b_pawns & !(b_pawn_advances & !w_pawns).shift::<Up>(1);
        let w_king = board.king(Color::White);
        let b_king = board.king(Color::Black);

        EvalData {
            king_zone: [
                king_zone(w_king, Color::White),
                king_zone(b_king, Color::Black)
            ],
            not_diag_sliders: [
                blockers & !board.colored_diag_sliders(Color::White),
                blockers & !board.colored_diag_sliders(Color::Black)
            ],
            not_orth_sliders: [
                blockers & !board.colored_orth_sliders(Color::White),
                blockers & !board.colored_orth_sliders(Color::Black)
            ],
            attacks: [w_attacks, b_attacks],
            double_attacks: [w_double_attacks, b_double_attacks],
            pawn_attacks: [w_pawn_attacks, b_pawn_attacks],
            pawn_double_attacks: [w_pawn_double_attacks, b_pawn_double_attacks],
            blocked_pawns: [w_blocked_pawns, b_blocked_pawns],
            rammed_pawns: [w_rammed_pawns, b_rammed_pawns],
            mobility_area: [
                !(b_pawn_attacks | w_king.bitboard() | w_blocked_pawns),
                !(w_pawn_attacks | b_king.bitboard() | b_blocked_pawns),
            ],
            semiopen_files: [w_semiopen_files, b_semiopen_files],
            open_files,
        }
    }

    #[inline(always)]
    pub fn king_zone(&self, color: Color) -> BitBoard {
        self.king_zone[color as usize]
    }

    #[inline(always)]
    pub fn not_diag_sliders(&self, color: Color) -> BitBoard {
        self.not_diag_sliders[color as usize]
    }

    #[inline(always)]
    pub fn not_orth_sliders(&self, color: Color) -> BitBoard {
        self.not_orth_sliders[color as usize]
    }
    
    #[inline(always)]
    pub fn attacks(&self, color: Color) -> BitBoard {
        self.attacks[color as usize]
    }

    #[inline(always)]
    pub fn double_attacks(&self, color: Color) -> BitBoard {
        self.double_attacks[color as usize]
    }

    #[inline(always)]
    pub fn pawn_attacks(&self, color: Color) -> BitBoard {
        self.pawn_attacks[color as usize]
    }

    #[inline(always)]
    pub fn pawn_double_attacks(&self, color: Color) -> BitBoard {
        self.pawn_double_attacks[color as usize]
    }

    #[inline(always)]
    pub fn blocked_pawns(&self, color: Color) -> BitBoard {
        self.blocked_pawns[color as usize]
    }

    #[inline(always)]
    pub fn rammed_pawns(&self, color: Color) -> BitBoard {
        self.rammed_pawns[color as usize]
    }

    #[inline(always)]
    pub fn mobility_area(&self, color: Color) -> BitBoard {
        self.mobility_area[color as usize]
    }

    #[inline(always)]
    pub fn semiopen_files(&self, color: Color) -> BitBoard {
        self.semiopen_files[color as usize]
    }
}

impl Default for EvalData {
    #[inline(always)]
    fn default() -> Self {
        EvalData {
            king_zone: [BitBoard::EMPTY; Color::NUM],
            not_diag_sliders: [BitBoard::EMPTY; Color::NUM],
            not_orth_sliders: [BitBoard::EMPTY; Color::NUM],
            attacks: [BitBoard::EMPTY; Color::NUM],
            double_attacks: [BitBoard::EMPTY; Color::NUM],
            pawn_attacks: [BitBoard::EMPTY; Color::NUM],
            pawn_double_attacks: [BitBoard::EMPTY; Color::NUM],
            blocked_pawns: [BitBoard::EMPTY; Color::NUM],
            rammed_pawns: [BitBoard::EMPTY; Color::NUM],
            mobility_area: [BitBoard::EMPTY; Color::NUM],
            semiopen_files: [BitBoard::EMPTY; Color::NUM],
            open_files: BitBoard::EMPTY,
        }
    }
}

/*----------------------------------------------------------------*/

macro_rules! trace {
    ($e:block) => {
        #[cfg(feature = "trace")] $e
    }
}

#[derive(Debug, Clone)]
pub struct Evaluator {
    #[cfg(feature="trace")] trace: EvalTrace,
    weights: EvalWeights,
    data: EvalData
}

impl Evaluator {
    pub fn with_weights(weights: EvalWeights) -> Self {
        let mut evaluator = Evaluator::default();
        evaluator.weights = weights;
        evaluator
    }
    
    pub fn eval(&mut self, board: &Board) -> Score {
        trace!({
            self.trace = EvalTrace::default();
        });

        let phase = calc_phase(board);
        let stm = match board.side_to_move() {
            Color::White => 1,
            Color::Black => -1,
        };

        self.data = EvalData::calc(board);
        let score = self.eval_psqt(board)
            + self.eval_bishops(board)
            + self.eval_open_files(board)
            + self.eval_mobility(board)
            + self.eval_threats(board)
            + self.eval_space(board)
            + self.eval_pawns(board);

        trace!({
            self.trace.phase = phase;
            self.trace.stm = stm;
        });

        stm * score.scale(phase)
    }
    
    #[cfg(feature="trace")]
    pub fn trace(&self) -> EvalTrace {
        self.trace.clone()
    }

    /*----------------------------------------------------------------*/

    fn eval_psqt(&mut self, board: &Board) -> T {
        let mut score = T::ZERO;
        let white = board.colors(Color::White);

        macro_rules! eval_pieces {
            ($piece:expr, $trace_value:expr, $trace_psqt:expr, $value:expr, $table:expr) => {
                for sq in board.pieces($piece) {
                    if white.has(sq) {
                        score += $value + $table[sq as usize];
                        trace!({
                            $trace_value += 1;
                            $trace_psqt.white |= sq.bitboard();
                        });
                    } else {
                        score -= $value + $table[sq.flip_rank() as usize];
                        trace!({
                            $trace_value -= 1;
                            $trace_psqt.black |= sq.bitboard();
                        });
                    }
                }
            };
            
            ($piece:expr, $trace_psqt:expr, $table:expr) => {
                for sq in board.pieces($piece) {
                    if white.has(sq) {
                        score += $table[sq as usize];
                        trace!({
                            $trace_psqt.white |= sq.bitboard();
                        });
                    } else {
                        score -= $table[sq.flip_rank() as usize];
                        trace!({
                            $trace_psqt.black |= sq.bitboard();
                        });
                    }
                }
            }
        }

        eval_pieces!(Piece::Pawn, self.trace.pawn_value, self.trace.pawn_psqt, self.weights.pawn_value, self.weights.pawn_psqt);
        eval_pieces!(Piece::Knight, self.trace.knight_value, self.trace.knight_psqt, self.weights.knight_value, self.weights.knight_psqt);
        eval_pieces!(Piece::Bishop, self.trace.bishop_value, self.trace.bishop_psqt, self.weights.bishop_value, self.weights.bishop_psqt);
        eval_pieces!(Piece::Rook, self.trace.rook_value, self.trace.rook_psqt, self.weights.rook_value, self.weights.rook_psqt);
        eval_pieces!(Piece::Queen, self.trace.queen_value, self.trace.queen_psqt, self.weights.queen_value, self.weights.queen_psqt);
        eval_pieces!(Piece::King, self.trace.king_psqt, self.weights.king_psqt);

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_bishops(&mut self, board: &Board) -> T {
        let mut score = T::ZERO;
        let w_bishops = board.colored_pieces(Color::White, Piece::Bishop);
        let b_bishops = board.colored_pieces(Color::Black, Piece::Bishop);

        if w_bishops.len() >= 2 && !(w_bishops.is_subset(BitBoard::LIGHT_SQUARES) || w_bishops.is_subset(BitBoard::DARK_SQUARES)) {
            score += self.weights.bishop_pair;
            
            trace!({
                self.trace.bishop_pair += 1;
            });
        }

        if b_bishops.len() >= 2 && !(b_bishops.is_subset(BitBoard::LIGHT_SQUARES) || b_bishops.is_subset(BitBoard::DARK_SQUARES)) {
            score -= self.weights.bishop_pair;

            trace!({
                self.trace.bishop_pair -= 1;
            });
        }

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_open_files(&mut self, board: &Board) -> T {
        let mut score = T::ZERO;
        let white = board.colors(Color::White);

        macro_rules! eval_pieces {
            ($piece:expr, $trace_open_bb: expr, $trace_semiopen_bb: expr, $open_param:expr, $semiopen_param:expr) => {
                for sq in board.pieces($piece) {
                    if white.has(sq) {
                        if self.data.open_files.has(sq) {
                            score += $open_param[sq.file() as usize];
                            
                            trace!({
                                $trace_open_bb.white |= sq.bitboard();
                            });
                        } else if self.data.semiopen_files(Color::White).has(sq) {
                            score += $semiopen_param[sq.file() as usize];
                            
                            trace!({
                                $trace_semiopen_bb.white |= sq.bitboard();
                            });
                        }
                    } else {
                        if self.data.open_files.has(sq) {
                            score -= $open_param[sq.file() as usize];
                            
                            trace!({
                                $trace_open_bb.black |= sq.bitboard();
                            });
                        } else if self.data.semiopen_files(Color::Black).has(sq) {
                            score -= $semiopen_param[sq.file() as usize];
                            
                            trace!({
                                $trace_semiopen_bb.black |= sq.bitboard();
                            });
                        }
                    }
                }
            }
        }

        eval_pieces!(Piece::Rook, self.trace.rook_open_file, self.trace.rook_semiopen_file, self.weights.rook_open_file, self.weights.rook_semiopen_file);
        eval_pieces!(Piece::Queen, self.trace.queen_open_file, self.trace.queen_semiopen_file, self.weights.queen_open_file, self.weights.queen_semiopen_file);

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_mobility(&mut self, board: &Board) -> T {
        let mut score = T::ZERO;
        let white = board.colors(Color::White);
        let blockers = board.occupied();
        let not_pinned = !board.pinned();

        macro_rules! eval_pieces {
            ($piece:expr, $trace_indices:expr, $attack_fn:ident, $table:expr) => {
                for sq in board.pieces($piece) & not_pinned {
                    let attacks = $attack_fn(sq);

                    if white.has(sq) {
                        let index = (attacks & self.data.mobility_area(Color::White)).len() as usize;
                        score += $table[index];
                        
                        trace!({
                            $trace_indices.white.push(index);
                        });
                    } else {
                        let index = (attacks & self.data.mobility_area(Color::Black)).len() as usize;
                        score -= $table[index];
                        
                        trace!({
                            $trace_indices.black.push(index);
                        });
                    }
                }
            }
        }

        macro_rules! eval_sliders {
            ($piece:expr, $trace_indices:expr, $attack_fn:ident, $blockers:expr, $table:expr) => {
                for sq in board.pieces($piece) & not_pinned {
                    if white.has(sq) {
                        let attacks = $attack_fn(sq, $blockers[0]);
                        let index = (attacks & self.data.mobility_area(Color::White)).len() as usize;
                        
                        score += $table[index];
                        
                        trace!({
                            $trace_indices.white.push(index);
                        });
                    } else {
                        let attacks = $attack_fn(sq, $blockers[1]);
                        let index = (attacks & self.data.mobility_area(Color::Black)).len() as usize;

                        score -= $table[index];
                        
                        trace!({
                            $trace_indices.black.push(index);
                        });
                    }
                }
            };

            ($piece:expr, $trace_indices:expr, $diag_attack_fn:ident, $orth_attack_fn:ident, $diag_blockers:expr, $orth_blockers:expr, $table:expr) => {
                for sq in board.pieces($piece) & not_pinned {
                    if white.has(sq) {
                        let attacks = $diag_attack_fn(sq, $diag_blockers[0]) | $orth_attack_fn(sq, $orth_blockers[0]);
                        let index = (attacks & self.data.mobility_area(Color::White)).len() as usize;
                        score += $table[index];
                        
                        trace!({
                            $trace_indices.white.push(index);
                        });
                        
                    } else {
                        let attacks = $diag_attack_fn(sq, $diag_blockers[1]) | $orth_attack_fn(sq, $orth_blockers[1]);
                        let index = (attacks & self.data.mobility_area(Color::Black)).len() as usize;
                        score -= $table[index];
                        
                        trace!({
                            $trace_indices.black.push(index);
                        });
                    }
                }
            }
        }

        eval_pieces!(Piece::Knight, self.trace.knight_mobility, get_knight_moves, self.weights.knight_mobility);
        eval_sliders!(Piece::Bishop, self.trace.bishop_mobility, get_bishop_moves, self.data.not_diag_sliders, self.weights.bishop_mobility);
        eval_sliders!(Piece::Rook, self.trace.rook_mobility, get_rook_moves, self.data.not_orth_sliders, self.weights.rook_mobility);
        eval_sliders!(Piece::Queen, self.trace.queen_mobility, get_bishop_moves, get_rook_moves, self.data.not_diag_sliders, self.data.not_orth_sliders, self.weights.queen_mobility);
        
        score
    }

    /*----------------------------------------------------------------*/

    fn eval_threats(&mut self, board: &Board) -> T {
        let mut score = T::ZERO;

        let w_king = self.data.king_zone(Color::White);
        let b_king = self.data.king_zone(Color::Black);
        let (w_minors, w_majors) = (board.colored_minors(Color::White), board.colored_majors(Color::White));
        let (b_minors, b_majors) = (board.colored_minors(Color::Black), board.colored_majors(Color::Black));
        let not_pinned = !board.pinned();

        let w_pawn_attacks = self.data.pawn_attacks(Color::White);
        let b_pawn_attacks = self.data.pawn_attacks(Color::Black);
        
        macro_rules! pawn_threats {
            ($w_pieces:expr, $b_pieces:expr, $trace:expr, $weights:expr) => {
                let amount = (w_pawn_attacks & $b_pieces).len() as i16;
                score += amount * $weights;
                
                trace!({
                    $trace += amount;
                });
                
                let amount = (b_pawn_attacks & $w_pieces).len() as i16;
                score += amount * $weights;
                
                trace!({
                    $trace -= amount;
                });
            }
        }
        
        pawn_threats!(w_minors, b_minors, self.trace.pawn_minor_threat, self.weights.pawn_minor_threat);
        pawn_threats!(w_majors, b_majors, self.trace.pawn_major_threat, self.weights.pawn_major_threat);

        macro_rules! eval_minors {
            ($piece:expr, $trace_major:expr, $trace_king:expr, $attack_fn:ident, $attack_units:expr) => {
                for sq in board.colored_pieces(Color::White, $piece) & not_pinned {
                    let moves = $attack_fn(sq);
                    let major_threats = (moves & b_majors).len() as i16;
                    let king_threats = (moves & b_king).len() as i16;

                    score += major_threats * self.weights.minor_major_threat;
                    score += king_threats * $attack_units;
                    
                    trace!({
                        $trace_major += major_threats;
                        $trace_king += king_threats;
                    });
                }

                for sq in board.colored_pieces(Color::Black, $piece) & not_pinned {
                    let moves = $attack_fn(sq);
                    let major_threats = (moves & b_majors).len() as i16;
                    let king_threats = (moves & w_king).len() as i16;

                    score -= major_threats * self.weights.minor_major_threat;
                    score -= king_threats * $attack_units;
                    
                    trace!({
                        $trace_major -= major_threats;
                        $trace_king -= king_threats;
                    });
                }
            };

            ($piece:expr, $trace_major:expr, $trace_king:expr, $attack_fn:ident, $blockers:expr, $attack_units:expr) => {
                for sq in board.colored_pieces(Color::White, $piece) & not_pinned {
                    let moves = $attack_fn(sq, $blockers[0]);
                    let major_threats = (moves & b_majors).len() as i16;
                    let king_threats = (moves & b_king).len() as i16;

                    score += major_threats * self.weights.minor_major_threat;
                    score += king_threats * $attack_units;
                    
                    trace!({
                        $trace_major += major_threats;
                        $trace_king += king_threats;
                    });
                }

                for sq in board.colored_pieces(Color::Black, $piece) & not_pinned {
                    let moves = $attack_fn(sq, $blockers[1]);
                    let major_threats = (moves & b_majors).len() as i16;
                    let king_threats = (moves & w_king).len() as i16;

                    score -= major_threats * self.weights.minor_major_threat;
                    score -= king_threats * $attack_units;
                    
                    trace!({
                        $trace_major -= major_threats;
                        $trace_king -= king_threats;
                    });
                }
            }
        }

        macro_rules! eval_majors {
            ($piece:expr, $trace:expr, $attack_fn:ident, $blockers:expr, $attack_units:expr) => {
                for sq in board.colored_pieces(Color::White, $piece) & not_pinned {
                    let moves = $attack_fn(sq, $blockers[0]);
                    let threats = (moves & b_king).len() as i16;

                    score += threats * $attack_units;
                    
                    trace!({
                        $trace += threats;
                    });
                }

                for sq in board.colored_pieces(Color::Black, $piece) & not_pinned {
                    let moves = $attack_fn(sq, $blockers[1]);
                    let threats = (moves & w_king).len() as i16;

                    score -= threats * $attack_units;
                    
                    trace!({
                        $trace -= threats;
                    });
                }
            };

            ($piece:expr, $trace:expr, $diag_attack_fn:ident, $orth_attack_fn:ident, $diag_blockers:expr, $orth_blockers:expr, $attack_units:expr) => {
                for sq in board.colored_pieces(Color::White, $piece) & not_pinned {
                    let moves = $diag_attack_fn(sq, $diag_blockers[0]) | $orth_attack_fn(sq, $orth_blockers[0]);
                    let threats = (moves & b_king).len() as i16;
                    
                    score += threats * $attack_units;
                    
                    trace!({
                        $trace += threats;
                    });
                }

                for sq in board.colored_pieces(Color::Black, $piece) & not_pinned {
                    let moves = $diag_attack_fn(sq, $diag_blockers[1]) | $orth_attack_fn(sq, $orth_blockers[1]);
                    let threats = (moves & w_king).len() as i16;
                    
                    score -= threats * $attack_units;
                    
                    trace!({
                        $trace -= threats;
                    });
                }
            }
        }

        eval_minors!(Piece::Knight, self.trace.minor_major_threat, self.trace.knight_attack, get_knight_moves, self.weights.knight_attack);
        eval_minors!(Piece::Bishop, self.trace.minor_major_threat, self.trace.bishop_attack, get_bishop_moves, self.data.not_diag_sliders, self.weights.bishop_attack);
        eval_majors!(Piece::Rook, self.trace.rook_attack, get_rook_moves, self.data.not_orth_sliders, self.weights.rook_attack);
        eval_majors!(Piece::Queen, self.trace.queen_attack, get_bishop_moves, get_rook_moves, self.data.not_diag_sliders, self.data.not_orth_sliders, self.weights.queen_attack);

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_space(&mut self, board: &Board) -> T {
        let mut score = T::ZERO;
        
        const CENTER: BitBoard = BitBoard(0x3C3C3C3C0000);

        let w_uncontested = !self.data.attacks(Color::Black) & (self.data.attacks(Color::White) | board.colors(Color::White)) & CENTER;
        let b_uncontested = !self.data.attacks(Color::White) & (self.data.attacks(Color::Black) | board.colors(Color::Black)) & CENTER;
        
        score += w_uncontested.len() as i16 * self.weights.center_control;
        score -= b_uncontested.len() as i16 * self.weights.center_control;
        
        trace!({
            self.trace.center_control += w_uncontested.len() as i16;
            self.trace.center_control -= b_uncontested.len() as i16;
        });

        score
    }

    /*----------------------------------------------------------------*/

    fn eval_pawns(&mut self, board: &Board) -> T {
        let mut score = T::ZERO;

        let w_pawns = board.colored_pieces(Color::White, Piece::Pawn);
        let b_pawns = board.colored_pieces(Color::Black, Piece::Pawn);

        for pawn in w_pawns {
            let (file, rank) = (pawn.file(), pawn.rank());
            let (file_bb, adjacent) = (file.bitboard(), file.adjacent());
            let pass_mask = rank.above() & (file_bb | adjacent);
            let backward_mask = rank.below() & adjacent;

            let doubled = (file_bb & w_pawns).len() > 1;
            let passed = pass_mask.is_disjoint(b_pawns) && !doubled;
            let backwards = !backward_mask.is_disjoint(w_pawns) && !passed;
            let isolated = adjacent.is_disjoint(w_pawns);
            let phalanx = !(adjacent & rank.bitboard()).is_disjoint(w_pawns);
            let support = (w_pawns & get_pawn_attacks(pawn, Color::Black)).len() as i16;

            if doubled {
                score += self.weights.doubled_pawn;
                
                trace!({
                    self.trace.doubled_pawn += 1;
                });
            }
            if passed {
                score += self.weights.passed_pawn[rank as usize];

                trace!({
                    self.trace.passed_pawn.white |= pawn.bitboard();
                });
            }
            if backwards {
                score += self.weights.backwards_pawn;

                trace!({
                    self.trace.backwards_pawn += 1;
                });
            }
            if isolated {
                score += self.weights.isolated_pawn;

                trace!({
                    self.trace.isolated_pawn += 1;
                });
            }

            if phalanx || support > 0 {
                score += self.weights.phalanx[rank as usize];
                score += support * self.weights.support;
                
                trace!({
                    self.trace.phalanx.white |= pawn.bitboard();
                    self.trace.support += support;
                });
            }
        }

        for pawn in b_pawns {
            let (file, rank) = (pawn.file(), pawn.rank());
            let (file_bb, adjacent) = (file.bitboard(), file.adjacent());
            let pass_mask = rank.below() & (file_bb | adjacent);
            let backward_mask = rank.above() & adjacent;

            let doubled = (file_bb & b_pawns).len() > 1;
            let passed = pass_mask.is_disjoint(w_pawns) && !doubled;
            let backwards = !backward_mask.is_disjoint(b_pawns) && !passed;
            let isolated = adjacent.is_disjoint(b_pawns);
            let phalanx = !(adjacent & rank.bitboard()).is_disjoint(b_pawns);
            let support = (b_pawns & get_pawn_attacks(pawn, Color::White)).len() as i16;

            if doubled {
                score -= self.weights.doubled_pawn;

                trace!({
                    self.trace.doubled_pawn -= 1;
                });
            }
            if passed {
                score -= self.weights.passed_pawn[rank.flip() as usize];
                
                trace!({
                    self.trace.passed_pawn.black |= pawn.bitboard();
                });
            }
            if backwards {
                score -= self.weights.backwards_pawn;
                
                trace!({
                    self.trace.backwards_pawn -= 1;
                });
            }
            if isolated {
                score -= self.weights.isolated_pawn;
                
                trace!({
                    self.trace.backwards_pawn -= 1;
                })
            }

            if phalanx || support > 0 {
                score -= self.weights.phalanx[rank.flip() as usize];
                score -= support * self.weights.support;

                trace!({
                    self.trace.phalanx.black |= pawn.bitboard();
                    self.trace.support -= support;
                });
            }
        }

        score
    }
}

impl Default for Evaluator {
    #[inline(always)]
    fn default() -> Self {
        Evaluator {
            #[cfg(feature="trace")] trace: EvalTrace::default(),
            weights: EvalWeights::default(),
            data: EvalData::default(),
        }
    }
}

/*----------------------------------------------------------------*/

pub fn calc_phase(board: &Board) -> u16 {
    let mut phase: u16 = TOTAL_PHASE;

    phase -= board.pieces(Piece::Pawn).len() as u16 * PAWN_PHASE;
    phase -= board.pieces(Piece::Knight).len() as u16 * KNIGHT_PHASE;
    phase -= board.pieces(Piece::Bishop).len() as u16 * BISHOP_PHASE;
    phase -= board.pieces(Piece::Rook).len() as u16 * ROOK_PHASE;
    phase -= board.pieces(Piece::Queen).len() as u16 * QUEEN_PHASE;

    phase
}

pub fn piece_value(piece: Piece) -> i16 {
    match piece {
        Piece::Pawn => PAWN_VALUE.0,
        Piece::Knight => KNIGHT_VALUE.0,
        Piece::Bishop => BISHOP_VALUE.0,
        Piece::Rook => ROOK_VALUE.0,
        Piece::Queen => QUEEN_VALUE.0,
        Piece::King => i16::MAX,
    }
}