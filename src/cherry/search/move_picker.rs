use smallvec::SmallVec;

use crate::*;

#[inline]
fn select_next(moves: &[ScoredMove]) -> Option<usize> {
    moves
        .iter()
        .enumerate()
        .max_by_key(|(_, mv)| mv.1)
        .map(|(i, _)| i)
}

/*----------------------------------------------------------------*/

pub struct ScoredMove(pub Move, pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    TTMove,
    GenTactics,
    YieldGoodTactics,
    GenQuiets,
    YieldQuiets,
    YieldBadTactics,
    Finished,
}

pub struct MovePicker {
    stage: Stage,
    skip_quiets: bool,
    skip_bad_tactics: bool,
    tt_move: Option<Move>,
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

    pub fn next(
        &mut self,
        pos: &mut Position,
        history: &History,
        indices: &ContIndices,
    ) -> Option<ScoredMove> {
        if self.stage == Stage::TTMove {
            self.stage = Stage::GenTactics;

            if let Some(mv) = self.tt_move
                && pos.board().is_legal(mv)
            {
                let score = if mv.is_tactic() {
                    history.tactic(pos.board(), mv)
                } else {
                    history.quiet(pos.board(), indices, mv)
                };

                return Some(ScoredMove(mv, score));
            }
        }

        if self.stage == Stage::GenTactics {
            self.stage = Stage::YieldGoodTactics;

            for &mv in pos.board().gen_tactics().iter() {
                if self.tt_move == Some(mv) {
                    continue;
                }

                self.good_tactics
                    .push(ScoredMove(mv, history.tactic(pos.board(), mv)));
            }
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::YieldGoodTactics {
            while let Some(index) = select_next(&self.good_tactics) {
                let mv = swap_pop(&mut self.good_tactics, index).unwrap();

                if pos.cmp_see(mv.0, 0) {
                    return Some(mv);
                } else {
                    if !self.skip_bad_tactics {
                        self.bad_tactics.push(mv);
                    }

                    continue;
                }
            }

            if !self.skip_quiets {
                self.stage = Stage::GenQuiets;
            } else {
                self.stage = Stage::YieldBadTactics;
            }
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::GenQuiets {
            if self.skip_quiets {
                self.stage = Stage::YieldBadTactics;
            } else {
                self.stage = Stage::YieldQuiets;

                for &mv in pos.board().gen_quiets().iter() {
                    if self.tt_move == Some(mv) {
                        continue;
                    }

                    self.quiets
                        .push(ScoredMove(mv, history.quiet(pos.board(), indices, mv)));
                }
            }
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::YieldQuiets {
            if self.skip_quiets {
                self.stage = Stage::YieldBadTactics;
            } else {
                if let Some(index) = select_next(&self.quiets) {
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
                if let Some(index) = select_next(&self.bad_tactics) {
                    return swap_pop(&mut self.bad_tactics, index);
                }

                self.stage = Stage::Finished;
            }
        }

        None
    }
}
