use smallvec::SmallVec;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct ScoredMove(pub Move, pub i32);

fn select_next_64(moves: &SmallVec<[ScoredMove; 64]>) -> Option<usize> {
    if moves.is_empty() {
        return None;
    }

    moves.iter()
        .enumerate()
        .max_by_key(|(_, mv)| mv.1)
        .map(|(i, _)| i)
}

fn select_next_32(moves: &SmallVec<[ScoredMove; 32]>) -> Option<usize> {
    if moves.is_empty() {
        return None;
    }

    moves.iter()
        .enumerate()
        .max_by_key(|(_, mv)| mv.1)
        .map(|(i, _)| i)
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    HashMove,
    GenMoves,
    YieldGoodTactics,
    YieldQuiets,
    YieldBadTactics,
    Finished
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct MovePicker {
    phase: Phase,
    moves: MoveList,
    hash_move: Option<Move>,
    good_tactics: SmallVec<[ScoredMove; 64]>,
    bad_tactics: SmallVec<[ScoredMove; 32]>,
    quiets: SmallVec<[ScoredMove; 64]>,
}

impl MovePicker {
    #[inline]
    pub fn new(moves: MoveList, hash_move: Option<Move>) -> MovePicker {
        MovePicker {
            phase: Phase::HashMove,
            moves,
            hash_move,
            good_tactics: SmallVec::new(),
            bad_tactics: SmallVec::new(),
            quiets: SmallVec::new(),
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
            Phase::YieldQuiets => Phase::YieldBadTactics,
            _ => self.phase
        }
    }

    /*----------------------------------------------------------------*/
    
    pub fn next(&mut self, pos: &mut Position, history: &History, indices: &ContIndices) -> Option<ScoredMove> {
        if self.phase == Phase::HashMove {
            self.phase = Phase::GenMoves;
            
            if let Some(mv) = self.hash_move && self.moves.contains(&mv) {
                let board = pos.board();
                let score = if mv.is_tactic() {
                    history.get_tactic(board, mv, pos.board().cmp_see(mv, 0))
                } else {
                    history.get_quiet_total(board, mv, indices)
                };

                return Some(ScoredMove(mv, score));
            }
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenMoves {
            self.phase = Phase::YieldGoodTactics;

            for &mv in self.moves.iter() {
                if mv.is_tactic() {
                    self.good_tactics.push(ScoredMove(mv, history.get_tactic(pos.board(), mv, true)));
                } else {
                    self.quiets.push(ScoredMove(mv, history.get_quiet_total(pos.board(), mv, indices)));
                }
            }
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldGoodTactics {
            while let Some(index) = select_next_64(&self.good_tactics) {
                let mv = swap_pop(&mut self.good_tactics, index).unwrap();

                if pos.board().cmp_see(mv.0, 0) {
                    return Some(mv);
                } else {
                    self.bad_tactics.push(ScoredMove(mv.0, history.get_tactic(pos.board(), mv.0, false)));
                    continue;
                }
            }
            
            self.phase = Phase::YieldQuiets;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldQuiets {
            if let Some(index) = select_next_64(&self.quiets) {
                return swap_pop(&mut self.quiets, index);
            }
            
            self.phase = Phase::YieldBadTactics;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldBadTactics {
            if let Some(index) = select_next_32(&self.bad_tactics) {
                return swap_pop(&mut self.bad_tactics, index);
            }
            
            self.phase = Phase::Finished;
        }

        None
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QPhase {
    GenMoves,
    GenEvasions,
    YieldEvasions,
    GenTactics,
    YieldTactics,
    Finished,
}

#[derive(Debug, Clone)]
pub struct QMovePicker {
    phase: QPhase,
    moves: MoveList,
    evasions: SmallVec<[ScoredMove; 32]>,
    tactics: SmallVec<[ScoredMove; 32]>,
}

impl QMovePicker {
    #[inline]
    pub fn new(moves: MoveList) -> QMovePicker {
        QMovePicker {
            phase: QPhase::GenMoves,
            moves,
            evasions: SmallVec::new(),
            tactics: SmallVec::new(),
        }
    }

    pub fn next(&mut self, pos: &mut Position, history: &History, indices: &ContIndices) -> Option<Move> {
        if self.phase == QPhase::GenMoves {
            if pos.board().in_check() {
                self.phase = QPhase::GenEvasions;
            } else {
                self.phase = QPhase::GenTactics;
            }
        }

        /*----------------------------------------------------------------*/
        
        if self.phase == QPhase::GenEvasions {
            let board = pos.board();
            for &mv in self.moves.iter() {
                let score = if mv.is_tactic() {
                    history.get_tactic(board, mv, true)
                } else {
                    history.get_quiet_total(board, mv, indices)
                };

                self.evasions.push(ScoredMove(mv, score));
            }
            
            self.phase = QPhase::YieldEvasions;
        }

        /*----------------------------------------------------------------*/
        
        if self.phase == QPhase::YieldEvasions {
            if let Some(index) = select_next_32(&self.evasions) {
                return swap_pop(&mut self.evasions, index).map(|mv| mv.0);
            }

            self.phase = QPhase::Finished;
        }

        /*----------------------------------------------------------------*/
        
        if self.phase == QPhase::GenTactics {
            let board = pos.board();

            for &mv in self.moves.iter() {
                if mv.is_tactic() {
                    self.tactics.push(ScoredMove(mv, history.get_tactic(board, mv, true)));
                }
            }

            self.phase = QPhase::YieldTactics;
        }

        /*----------------------------------------------------------------*/

        if self.phase == QPhase::YieldTactics {
            if let Some(index) = select_next_32(&self.tactics) {
                return swap_pop(&mut self.tactics, index).map(|mv| mv.0);
            }
            
            self.phase = QPhase::Finished;
        }
        
        None
    }
}