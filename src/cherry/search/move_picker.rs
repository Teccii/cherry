use arrayvec::ArrayVec;
use crate::*;

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

pub struct ScoredMove(pub Move, pub i16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    GenMoves,
    YieldGoodTactics,
    YieldQuiets,
    YieldBadTactics,
    Finished
}

pub struct MovePicker {
    stage: Stage,
    good_tactics: ArrayVec<ScoredMove, MAX_MOVES>,
    bad_tactics: ArrayVec<ScoredMove, MAX_MOVES>,
    quiets: ArrayVec<ScoredMove, MAX_MOVES>,
}

impl MovePicker {
    #[inline]
    pub fn new() -> MovePicker {
        MovePicker {
            stage: Stage::GenMoves,
            good_tactics: ArrayVec::new(),
            bad_tactics: ArrayVec::new(),
            quiets: ArrayVec::new(),
        }
    }

    pub fn next(&mut self, pos: &mut Position) -> Option<ScoredMove> {
        if self.stage == Stage::GenMoves {
            self.stage = Stage::YieldGoodTactics;

            let moves = pos.board().gen_moves();
            for &mv in moves.iter() {
                if mv.is_tactic() {
                    self.good_tactics.push(ScoredMove(mv, 0));
                } else {
                    self.quiets.push(ScoredMove(mv, 0));
                }
            }
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::YieldGoodTactics {
            while let Some(index) = select_next(&self.good_tactics) {
                let mv = self.good_tactics.swap_pop(index).unwrap();

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
            if let Some(index) = select_next(&self.quiets) {
                return self.quiets.swap_pop(index);
            }

            self.stage = Stage::YieldBadTactics;
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::YieldBadTactics {
            if let Some(index) = select_next(&self.bad_tactics) {
                return self.bad_tactics.swap_pop(index);
            }

            self.stage = Stage::Finished;
        }

        None
    }
}