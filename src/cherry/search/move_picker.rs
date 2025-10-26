use smallvec::SmallVec;
use crate::*;

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

pub struct ScoredMove(pub Move, pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    TTMove,
    GenMoves,
    YieldGoodTactics,
    YieldQuiets,
    YieldBadTactics,
    Finished
}

pub struct MovePicker {
    stage: Stage,
    skip_quiets: bool,
    skip_bad_tactics: bool,
    tt_move: Option<Move>,
    move_list: MoveList,
    good_tactics: SmallVec<[ScoredMove; 64]>,
    bad_tactics: SmallVec<[ScoredMove; 32]>,
    quiets: SmallVec<[ScoredMove; 64]>,
}

impl MovePicker {
    #[inline]
    pub fn new(tt_move: Option<Move>) -> MovePicker {
        MovePicker {
            stage: Stage::TTMove,
            skip_quiets: false,
            skip_bad_tactics: false,
            tt_move,
            move_list: MoveList::empty(),
            good_tactics: SmallVec::new(),
            bad_tactics: SmallVec::new(),
            quiets: SmallVec::new(),
        }
    }
    
    #[inline]
    pub fn stage(&self) -> Stage {
        self.stage
    }

    #[inline]
    pub fn skip_quiets(&mut self) {
        self.skip_quiets = true;
    }

    #[inline]
    pub fn skip_bad_tactics(&mut self) {
        self.skip_bad_tactics = true;
    }

    pub fn next(&mut self, pos: &mut Position, history: &History, indices: &ContIndices) -> Option<ScoredMove> {
        if self.stage == Stage::TTMove {
            self.stage = Stage::GenMoves;
            
            if let Some(mv) = self.tt_move {
                self.move_list = pos.board().gen_moves();
                
                if self.move_list.contains(&mv) {
                    let score = if mv.is_tactic() {
                        history.get_tactic(pos.board(), mv)
                    } else {
                        history.get_quiet_total(pos.board(), indices, mv)
                    };

                    return Some(ScoredMove(mv, score));
                }
            }
        }
        
        if self.stage == Stage::GenMoves {
            self.stage = Stage::YieldGoodTactics;

            if self.move_list.is_empty() {
                self.move_list = pos.board().gen_moves();
            }
            
            for &mv in self.move_list.iter() {
                if self.tt_move == Some(mv) {
                    continue;
                }

                if mv.is_tactic() {
                    self.good_tactics.push(ScoredMove(mv, history.get_tactic(pos.board(), mv)));
                } else {
                    self.quiets.push(ScoredMove(mv, history.get_quiet_total(pos.board(), indices, mv)));
                }
            }
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::YieldGoodTactics {
            while let Some(index) = select_next_64(&self.good_tactics) {
                let mv = swap_pop(&mut self.good_tactics, index).unwrap();

                if pos.board().cmp_see(mv.0, 0) {
                    return Some(mv);
                } else {
                    self.bad_tactics.push(mv);
                    continue;
                }
            }

            self.stage = Stage::YieldQuiets;
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::YieldQuiets {
            if self.skip_quiets {
                self.stage = Stage::YieldBadTactics;
            } else {
                if let Some(index) = select_next_64(&self.quiets) {
                    return swap_pop(&mut self.quiets, index);
                }

                self.stage = Stage::YieldBadTactics;
            }
        }

        /*----------------------------------------------------------------*/
        if self.stage == Stage::YieldBadTactics {
            if self.skip_bad_tactics {
                self.stage = Stage::Finished;
            } else {
                if let Some(index) = select_next_32(&self.bad_tactics) {
                    return swap_pop(&mut self.bad_tactics, index)
                }

                self.stage = Stage::Finished;
            }
        }

        None
    }
}