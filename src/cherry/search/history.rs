use crate::*;

/*----------------------------------------------------------------*/

pub const MAX_HISTORY: i32 = 16384;

#[inline]
fn delta(depth: i32, base: i32, mul: i32, max: i32) -> i32 {
    i32::min(base + mul * depth / DEPTH_SCALE, max)
}

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct History {
    quiets: Box<ColorTo<SquareTo<SquareTo<i32>>>>,
    tactics: Box<ColorTo<PieceTo<SquareTo<i32>>>>,
}

impl History {
    #[inline]
    pub fn reset(&mut self) {
        *self = History::default();
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_quiet(&self, board: &Board, mv: Move) -> i32 {
        self.quiets[board.stm()][mv.from()][mv.to()]
    }

    #[inline]
    pub fn get_quiet_mut(&mut self, board: &Board, mv: Move) -> &mut i32 {
        &mut self.quiets[board.stm()][mv.from()][mv.to()]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_tactic(&self, board: &Board, mv: Move) -> i32 {
        self.tactics[board.stm()][board.piece_on(mv.from()).unwrap()][mv.to()]
    }

    #[inline]
    pub fn get_tactic_mut(&mut self, board: &Board, mv: Move) -> &mut i32 {
        &mut self.tactics[board.stm()][board.piece_on(mv.from()).unwrap()][mv.to()]
    }

    /*----------------------------------------------------------------*/

    pub fn update(
        &mut self,
        board: &Board,
        best_move: Move,
        tactics: &[Move],
        quiets: &[Move],
        depth: i32,
    ) {
        if best_move.is_tactic() {
            History::update_value(
                self.get_tactic_mut(board, best_move),
                delta(depth, W::tactic_bonus_base(), W::tactic_bonus_mul(), W::tactic_bonus_max())
            );
        } else {
            History::update_value(
                self.get_quiet_mut(board, best_move),
                delta(depth, W::quiet_bonus_base(), W::quiet_bonus_mul(), W::quiet_bonus_max())
            );

            for &mv in quiets {
                History::update_value(
                    self.get_quiet_mut(board, mv),
                    -delta(depth, W::quiet_malus_base(), W::quiet_malus_mul(), W::quiet_malus_max())
                );
            }
        }

        for &mv in tactics {
            History::update_value(
                self.get_tactic_mut(board, mv),
                -delta(depth, W::tactic_malus_base(), W::tactic_malus_mul(), W::tactic_malus_max())
            );
        }
    }

    #[inline]
    fn update_value(value: &mut i32, amount: i32) {
        let amount = amount.clamp(-MAX_HISTORY, MAX_HISTORY);
        let decay = *value * amount.abs() / MAX_HISTORY;

        *value += amount - decay;
    }
}

impl Default for History {
    #[inline]
    fn default() -> History {
        History {
            quiets: new_zeroed(),
            tactics: new_zeroed(),
        }
    }
}