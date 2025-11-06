use crate::*;

/*----------------------------------------------------------------*/

pub const MAX_HISTORY: i32 = 16384;

#[inline]
fn delta(depth: i32, base: i32, mul: i32, max: i32) -> i32 {
    i32::min(base + mul * depth / DEPTH_SCALE, max)
}

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct ContIndices {
    pub counter_move: Option<MoveData>,
}

impl ContIndices {
    #[inline]
    pub fn new(search_stack: &[SearchStack], ply: u16) -> ContIndices {
        ContIndices {
            counter_move: (ply >= 1).then(|| search_stack[ply as usize - 1].move_played).flatten(),
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct History {
    quiets: Box<ColorTo<SquareTo<BoolTo<SquareTo<i32>>>>>, //Indexing: [stm][src][dest threatened][dest]
    tactics: Box<ColorTo<PieceTo<SquareTo<i32>>>>, //Indexing: [stm][piece][dest]
    counter_move: Box<ColorTo<PieceTo<SquareTo<PieceTo<SquareTo<i32>>>>>>, //Indexing: [stm][prev piece][prev dest][piece][dest]
}

impl History {
    #[inline]
    pub fn reset(&mut self) {
        *self = History::default();
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_quiet(&self, board: &Board, mv: Move) -> i32 {
        let stm = board.stm();
        let dest = mv.dest();
        let dest_threatened = !board.attack_table(!stm).get(dest).is_empty();

        self.quiets[stm][mv.src()][dest_threatened as usize][dest]
    }

    #[inline]
    pub fn get_quiet_mut(&mut self, board: &Board, mv: Move) -> &mut i32 {
        let stm = board.stm();
        let dest = mv.dest();
        let dest_threatened = !board.attack_table(!stm).get(dest).is_empty();

        &mut self.quiets[stm][mv.src()][dest_threatened as usize][dest]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_tactic(&self, board: &Board, mv: Move) -> i32 {
        self.tactics[board.stm()][board.piece_on(mv.src()).unwrap()][mv.dest()]
    }

    #[inline]
    pub fn get_tactic_mut(&mut self, board: &Board, mv: Move) -> &mut i32 {
        &mut self.tactics[board.stm()][board.piece_on(mv.src()).unwrap()][mv.dest()]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_counter_move(&self, board: &Board, mv: Move, prev_mv: Option<MoveData>) -> Option<i32> {
        let prev_mv = prev_mv?;

        Some(self.counter_move[board.stm()]
            [prev_mv.piece][prev_mv.mv.dest()]
            [board.piece_on(mv.src()).unwrap()][mv.dest()]
        )
    }

    #[inline]
     fn get_counter_move_mut(&mut self, board: &Board, mv: Move, prev_mv: Option<MoveData>) -> Option<&mut i32> {
        let prev_mv = prev_mv?;

        Some(&mut self.counter_move[board.stm()]
            [prev_mv.piece][prev_mv.mv.dest()]
            [board.piece_on(mv.src()).unwrap()][mv.dest()]
        )
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_quiet_total(&self, board: &Board, indices: &ContIndices, mv: Move) -> i32 {
        self.get_quiet(board, mv) + self.get_counter_move(board, mv, indices.counter_move).unwrap_or_default()
    }

    /*----------------------------------------------------------------*/

    pub fn update(
        &mut self,
        board: &Board,
        indices: &ContIndices,
        depth: i32,
        best_move: Move,
        tactics: &[Move],
        quiets: &[Move],
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

            /*----------------------------------------------------------------*/

            if let Some(value) = self.get_counter_move_mut(board, best_move, indices.counter_move) {
                History::update_value(
                    value,
                    delta(depth, W::cont1_bonus_base(), W::cont1_bonus_mul(), W::cont1_bonus_max())
                );

                for &mv in quiets {
                    History::update_value(
                        self.get_counter_move_mut(board, mv, indices.counter_move).unwrap(),
                        -delta(depth, W::cont1_malus_base(), W::cont1_malus_mul(), W::cont1_malus_max())
                    );
                }
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
            counter_move: new_zeroed(),
        }
    }
}