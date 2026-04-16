use smallvec::{Array, SmallVec};

use crate::*;

/*----------------------------------------------------------------*/

fn swap_pop<A: Array>(vec: &mut SmallVec<A>, index: usize) -> Option<A::Item> {
    let len = vec.len();

    if index >= len {
        return None;
    }

    vec.swap(index, len - 1);
    vec.pop()
}

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
    GenNoisies,
    YieldGoodNoisies,
    GenQuiets,
    YieldQuiets,
    YieldBadNoisies,
    Finished,
}

pub struct MovePicker {
    stage: Stage,
    see_margin: i32,
    skip_quiets: bool,
    skip_bad_noisies: bool,
    tt_move: Option<Move>,
    good_noisies: SmallVec<[ScoredMove; 64]>,
    bad_noisies: SmallVec<[ScoredMove; 32]>,
    quiets: SmallVec<[ScoredMove; 64]>,
}

impl MovePicker {
    #[inline]
    pub fn new(tt_move: Option<Move>, see_margin: i32) -> MovePicker {
        MovePicker {
            stage: Stage::TTMove,
            see_margin,
            skip_quiets: false,
            skip_bad_noisies: false,
            tt_move,
            good_noisies: SmallVec::new(),
            bad_noisies: SmallVec::new(),
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
    pub fn skip_bad_noisies(&mut self) {
        self.skip_bad_noisies = true;
    }

    pub fn next(
        &mut self,
        pos: &mut Position,
        history: &History,
        indices: &ContIndices,
    ) -> Option<ScoredMove> {
        if self.stage == Stage::TTMove {
            self.stage = Stage::GenNoisies;

            if let Some(mv) = self.tt_move
                && pos.board().is_legal(mv)
            {
                let score = if mv.is_noisy() {
                    history.noisy(pos.board(), mv)
                } else {
                    history.quiet(pos.board(), indices, mv)
                };

                return Some(ScoredMove(mv, score));
            }
        }

        if self.stage == Stage::GenNoisies {
            self.stage = Stage::YieldGoodNoisies;

            for &mv in pos.board().gen_noisies().iter() {
                if self.tt_move == Some(mv) {
                    continue;
                }

                self.good_noisies
                    .push(ScoredMove(mv, history.noisy(pos.board(), mv)));
            }
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::YieldGoodNoisies {
            while let Some(index) = select_next(&self.good_noisies) {
                let mv = swap_pop(&mut self.good_noisies, index).unwrap();

                if pos.cmp_see(mv.0, self.see_margin) {
                    return Some(mv);
                } else {
                    if !self.skip_bad_noisies {
                        self.bad_noisies.push(mv);
                    }

                    continue;
                }
            }

            if !self.skip_quiets {
                self.stage = Stage::GenQuiets;
            } else {
                self.stage = Stage::YieldBadNoisies;
            }
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::GenQuiets {
            if self.skip_quiets {
                self.stage = Stage::YieldBadNoisies;
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
                self.stage = Stage::YieldBadNoisies;
            } else {
                if let Some(index) = select_next(&self.quiets) {
                    return swap_pop(&mut self.quiets, index);
                }

                self.stage = Stage::YieldBadNoisies;
            }
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::YieldBadNoisies {
            if self.skip_bad_noisies {
                self.stage = Stage::Finished;
            } else {
                if let Some(index) = select_next(&self.bad_noisies) {
                    return swap_pop(&mut self.bad_noisies, index);
                }

                self.stage = Stage::Finished;
            }
        }

        None
    }
}
