use arrayvec::ArrayVec;
use cherry_core::*;
use crate::*;

/*----------------------------------------------------------------*/

pub const MAX_MOVES: usize = 218;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct ScoredMove(pub Move, pub i16);

fn select_next(moves: &ArrayVec<ScoredMove, MAX_MOVES>) -> Option<usize> {
    if moves.is_empty() {
        return None;
    }

    moves.iter()
        .enumerate()
        .max_by_key(|(_, mv)| mv.1)
        .map(|(i, _)| i)
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Phase {
    HashMove,
    GenPieceMoves,
    GenCaptures,
    YieldGoodCaptures,
    GenQuiets,
    YieldQuiets,
    YieldBadCaptures,
    Finished
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct MovePicker {
    phase: Phase,
    hash_move: Option<Move>,
    piece_moves: ArrayVec<PieceMoves, 20>,
    good_captures: ArrayVec<ScoredMove, MAX_MOVES>,
    bad_captures: ArrayVec<ScoredMove, MAX_MOVES>,
    quiets: ArrayVec<ScoredMove, MAX_MOVES>,
}

impl MovePicker {
    #[inline]
    pub fn new(hash_move: Option<Move>) -> MovePicker {
        MovePicker {
            phase: Phase::HashMove,
            hash_move,
            piece_moves: ArrayVec::new(),
            good_captures: ArrayVec::new(),
            bad_captures: ArrayVec::new(),
            quiets: ArrayVec::new(),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    #[inline]
    pub fn skip_quiets(&mut self) {
        self.phase = match self.phase {
            Phase::GenQuiets | Phase::YieldQuiets => Phase::YieldBadCaptures,
            _ => self.phase
        }
    }

    /*----------------------------------------------------------------*/
    
    pub fn next(
        &mut self,
        pos: &mut Position,
        history: &History,
        counter_move: Option<MoveData>,
        follow_up: Option<MoveData>
    ) -> Option<Move> {
        if self.phase == Phase::HashMove {
            self.phase = Phase::GenPieceMoves;
            
            if self.hash_move.is_some() {
                return self.hash_move;
            }
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenPieceMoves {
            self.phase = Phase::GenCaptures;
            
            pos.board().gen_moves(|moves| {
                self.piece_moves.push(moves);
                false
            });
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenCaptures {
            self.phase = Phase::YieldGoodCaptures;
            
            let board = pos.board();
            let mask = board.colors(!board.stm());

            for mut moves in self.piece_moves.iter().copied() {
                if moves.piece == Piece::Pawn {
                    moves.to &= mask | board.ep_square().map_or(Bitboard::EMPTY, |sq| sq.bitboard());
                } else {
                    moves.to &= mask;
                }

                for mv in moves {
                    if self.hash_move == Some(mv) {
                        continue;
                    }
                    
                    let see = board.see(mv);
                    let score = history.get_capture(board, mv);
                    
                    if see >= 0  {
                        self.good_captures.push(ScoredMove(mv, score));
                    } else {
                        self.bad_captures.push(ScoredMove(mv, score));
                    }
                }
            }
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldGoodCaptures {
            if let Some(index) = select_next(&self.good_captures) {
                return self.good_captures.swap_pop(index).map(|mv| mv.0);
            }
            
            self.phase = Phase::GenQuiets;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenQuiets {
            let board = pos.board();
            let mask = !board.colors(!board.stm());

            for mut moves in self.piece_moves.iter().copied() {
                if moves.piece == Piece::Pawn {
                    moves.to &= mask & !board.ep_square().map_or(Bitboard::EMPTY, |sq| sq.bitboard());
                } else {
                    moves.to &= mask;
                }

                for mv in moves {
                    if self.hash_move == Some(mv) {
                        continue;
                    }
                    
                    let score = history.get_quiet(board, mv)
                        + history.get_counter_move(board, mv, counter_move).unwrap_or_default()
                        + history.get_follow_up(board, mv, follow_up).unwrap_or_default();

                    self.quiets.push(ScoredMove(mv, score));
                }
            }

            self.phase = Phase::YieldQuiets;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldQuiets {
            if let Some(index) = select_next(&self.quiets) {
                return self.quiets.swap_pop(index).map(|mv| mv.0);
            }
            
            self.phase = Phase::YieldBadCaptures;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldBadCaptures {
            if let Some(index) = select_next(&self.bad_captures) {
                return self.bad_captures.swap_pop(index).map(|mv| mv.0);
            }
            
            self.phase = Phase::Finished;
        }

        /*----------------------------------------------------------------*/

        None
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QPhase {
    GenPieceMoves,
    GenEvasions,
    YieldEvasions,
    GenCaptures,
    YieldCaptures,
    Finished,
}

#[derive(Debug, Clone)]
pub struct QMovePicker {
    phase: QPhase,
    piece_moves: ArrayVec<PieceMoves, 20>,
    evasions: ArrayVec<ScoredMove, MAX_MOVES>,
    captures: ArrayVec<ScoredMove, MAX_MOVES>,
}

impl QMovePicker {
    #[inline]
    pub fn new() -> QMovePicker {
        QMovePicker {
            phase: QPhase::GenPieceMoves,
            piece_moves: ArrayVec::new(),
            evasions: ArrayVec::new(),
            captures: ArrayVec::new(),
        }
    }

    pub fn next(
        &mut self,
        pos: &mut Position,
        history: &History,
        counter_move: Option<MoveData>,
        follow_up: Option<MoveData>,
    ) -> Option<Move> {
        if self.phase == QPhase::GenPieceMoves {
            pos.board().gen_moves(|moves| {
                self.piece_moves.push(moves);
                false
            });
            
            if pos.in_check() {
                self.phase = QPhase::GenEvasions;
            } else {
                self.phase = QPhase::GenCaptures;
            }
        }
        
        if self.phase == QPhase::GenEvasions {
            let board = pos.board();

            for moves in self.piece_moves.iter().copied() {
                for mv in moves {
                    let score = history.get_move(board, mv, counter_move, follow_up);

                    self.evasions.push(ScoredMove(mv, score));
                }
            }

            self.phase = QPhase::YieldEvasions;
        }
        
        if self.phase == QPhase::YieldEvasions {
            if let Some(index) = select_next(&self.evasions) {
                return self.evasions.swap_pop(index).map(|mv| mv.0);
            }
            
            self.phase = QPhase::Finished;
        }
        
        if self.phase == QPhase::GenCaptures {
            let board = pos.board();
            let mask = board.colors(!board.stm());

            for mut moves in self.piece_moves.iter().copied() {
                if moves.piece == Piece::Pawn {
                    moves.to &= mask | board.ep_square().map_or(Bitboard::EMPTY, |sq| sq.bitboard());
                } else {
                    moves.to &= mask;
                }

                for mv in moves {
                    let see = board.see(mv);
                    let score = history.get_capture(board, mv);

                    if see >= 0 {
                        self.captures.push(ScoredMove(mv, score));
                    }
                }
            }

            self.phase = QPhase::YieldCaptures;
        }

        /*----------------------------------------------------------------*/

        if self.phase == QPhase::YieldCaptures {
            if let Some(index) = select_next(&self.captures) {
                return self.captures.swap_pop(index).map(|mv| mv.0);
            }
            
            self.phase = QPhase::Finished;
        }
        
        None
    }
}