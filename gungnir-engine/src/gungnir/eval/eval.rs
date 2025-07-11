use gungnir_core::*;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone, Default)]
pub struct EvalData {
    attacks: [Bitboard; Color::COUNT],
    pawn_attacks: [Bitboard; Color::COUNT],
    blocked_pawns: [Bitboard; Color::COUNT],
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
        
        let pawns = board.pieces(Piece::Pawn);
        let blockers = board.occupied();

        let w_pawns = board.color_pieces(Piece::Pawn, Color::White);
        let b_pawns = board.color_pieces(Piece::Pawn, Color::Black);
        let mut w_semiopen_files = Bitboard::EMPTY;
        let mut b_semiopen_files = Bitboard::EMPTY;
        let mut open_files = Bitboard::EMPTY;
        
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
            blocked_pawns: [w_blocked_pawns, b_blocked_pawns],
            mobility_area: [
                !(b_pawn_attacks | w_blocked_pawns),
                !(w_pawn_attacks | b_blocked_pawns)
            ],
            semiopen_files: [w_semiopen_files, b_semiopen_files],
            open_files
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Evaluator {
    weights: EvalWeights,
}

impl Evaluator {
    #[inline(always)]
    pub fn new(weights: EvalWeights) -> Evaluator {
        Evaluator { weights }
    }

    /*----------------------------------------------------------------*/

    pub fn eval(&self, board: &Board) -> Score {
        let phase = calc_phase(board);

        let data = EvalData::get(board);
        let score = self.eval_psqt(board)
            + self.eval_mobility(board, &data);
        
        Score::new(board.stm().sign() * score.scale(phase))
    }

    /*----------------------------------------------------------------*/
    
    fn eval_psqt(&self, board: &Board) -> T {
        let mut score = T::ZERO;
        
        macro_rules! eval_pieces {
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

        eval_pieces!(Piece::Pawn, self.weights.pawn_value, self.weights.pawn_psqt);
        eval_pieces!(Piece::Knight, self.weights.knight_value, self.weights.knight_psqt);
        eval_pieces!(Piece::Bishop, self.weights.bishop_value, self.weights.bishop_psqt);
        eval_pieces!(Piece::Rook, self.weights.rook_value, self.weights.rook_psqt);
        eval_pieces!(Piece::Queen, self.weights.queen_value, self.weights.queen_psqt);
        eval_pieces!(Piece::King, T::ZERO, self.weights.king_psqt);
        
        score
    }

    /*----------------------------------------------------------------*/

    fn eval_mobility(&self, board: &Board, data: &EvalData) -> T {
        let mut score = T::ZERO;
        let blockers = board.occupied();
        let white = board.colors(Color::White);
        

        score
    }
}

impl Default for Evaluator {
    #[inline(always)]
    fn default() -> Self {
        Evaluator { weights: EvalWeights::default() }
    }
}