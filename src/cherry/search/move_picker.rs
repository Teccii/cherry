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
    GenMoves,
    YieldGoodTactics,
    YieldQuiets,
    YieldBadTactics,
    Finished
}

pub struct MovePicker {
    stage: Stage,
    good_tactics: SmallVec<[ScoredMove; 64]>,
    bad_tactics: SmallVec<[ScoredMove; 32]>,
    quiets: SmallVec<[ScoredMove; 64]>,
}

impl MovePicker {
    #[inline]
    pub fn new() -> MovePicker {
        MovePicker {
            stage: Stage::GenMoves,
            good_tactics: SmallVec::new(),
            bad_tactics: SmallVec::new(),
            quiets: SmallVec::new(),
        }
    }

    pub fn next(&mut self, pos: &mut Position, history: &History) -> Option<ScoredMove> {
        if self.stage == Stage::GenMoves {
            self.stage = Stage::YieldGoodTactics;

            let moves = pos.board().gen_moves();
            for &mv in moves.iter() {
                if mv.is_tactic() {
                    self.good_tactics.push(ScoredMove(mv, history.get_tactic(pos.board(), mv)));
                } else {
                    self.quiets.push(ScoredMove(mv, history.get_quiet(pos.board(), mv)));
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
            if let Some(index) = select_next_64(&self.quiets) {
                return swap_pop(&mut self.quiets, index);
            }

            self.stage = Stage::YieldBadTactics;
        }

        /*----------------------------------------------------------------*/

        if self.stage == Stage::YieldBadTactics {
            if let Some(index) = select_next_32(&self.bad_tactics) {
                return swap_pop(&mut self.bad_tactics, index)
            }

            self.stage = Stage::Finished;
        }

        None
    }
}