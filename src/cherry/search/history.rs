use crate::*;

/*----------------------------------------------------------------*/

pub const MAX_HISTORY: i32 = 8192;
pub const MAX_CORRECTION: i16 = 1024;

const PAWN_CORRECTION_SIZE: usize = 1024;

/*----------------------------------------------------------------*/

pub type MoveTo<T> = [[T; Square::COUNT]; Square::COUNT];
pub type PieceTo<T> = [[T; Square::COUNT]; Piece::COUNT];
pub type ButterflyTable = [MoveTo<i32>; Color::COUNT];
pub type PieceToTable = [PieceTo<i32>; Color::COUNT];
pub type ContinuationTable = [PieceTo<PieceTo<i32>>; Color::COUNT];
pub type CorrectionTable<const SIZE: usize> = [[i16; SIZE]; Color::COUNT];

/*----------------------------------------------------------------*/

#[inline]
pub const fn move_to<T: Copy>(default: T) -> MoveTo<T> {
    [[default; Square::COUNT]; Square::COUNT]
}

#[inline]
pub const fn piece_to<T: Copy>(default: T) -> PieceTo<T> {
    [[default; Square::COUNT]; Piece::COUNT]
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct ContIndices {
    pub counter_move: Option<MoveData>,
    pub counter_move2: Option<MoveData>,
    pub follow_up: Option<MoveData>,
}

impl ContIndices {
    #[inline]
    pub fn new(ss: &[SearchStack], ply: u16) -> ContIndices {
        ContIndices {
            counter_move: (ply >= 1).then(|| ss[ply as usize - 1].move_played).flatten(),
            counter_move2: (ply >= 3).then(|| ss[ply as usize - 3].move_played).flatten(),
            follow_up: (ply >= 2).then(|| ss[ply as usize - 2].move_played).flatten(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct History {
    quiets: Box<ButterflyTable>,
    tactical: Box<PieceToTable>,
    counter_move: Box<ContinuationTable>, //use for 1-ply, 3-ply, 5-ply, etc.
    follow_up: Box<ContinuationTable>, //use for 2-ply, 4-ply, 6-ply, etc.
    pawn_corr: Box<CorrectionTable<PAWN_CORRECTION_SIZE>>,
}

impl History {
    #[inline]
    pub fn new() -> History {
        History {
            quiets: Box::new([move_to(0); Color::COUNT]),
            tactical: Box::new([piece_to(0); Color::COUNT]),
            counter_move: Box::new([piece_to(piece_to(0)); Color::COUNT]),
            follow_up: Box::new([piece_to(piece_to(0)); Color::COUNT]),
            pawn_corr: Box::new([[0; PAWN_CORRECTION_SIZE]; Color::COUNT]),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.quiets = Box::new([move_to(0); Color::COUNT]);
        self.tactical = Box::new([piece_to(0); Color::COUNT]);
        self.counter_move = Box::new([piece_to(piece_to(0)); Color::COUNT]);
        self.follow_up = Box::new([piece_to(piece_to(0)); Color::COUNT]);
        self.pawn_corr = Box::new([[0; PAWN_CORRECTION_SIZE]; Color::COUNT]);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_quiet(&self, board: &Board, mv: Move) -> i32 {
        self.quiets[board.stm() as usize]
            [mv.from() as usize]
            [mv.to() as usize]
    }
    
    #[inline]
    fn get_quiet_mut(&mut self, board: &Board, mv: Move) -> &mut i32 {
        &mut self.quiets[board.stm() as usize]
            [mv.to() as usize]
            [mv.to() as usize]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_tactical(&self, board: &Board, mv: Move) -> i32 {
        self.tactical[board.stm() as usize]
            [board.piece_on(mv.from()).unwrap() as usize]
            [mv.to() as usize]
    }

    #[inline]
    fn get_tactical_mut(&mut self, board: &Board, mv: Move) -> &mut i32 {
        &mut self.tactical[board.stm() as usize]
            [board.piece_on(mv.from()).unwrap() as usize]
            [mv.to() as usize]
    }

    /*----------------------------------------------------------------*/

    pub fn get_counter_move(
        &self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<i32> {
        let prev_mv = prev_mv?;

        Some(self.counter_move[board.stm() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from()).unwrap() as usize][mv.to() as usize])
    }

    fn get_counter_move_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<&mut i32> {
        let prev_mv = prev_mv?;

        Some(&mut self.counter_move[board.stm() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from()).unwrap() as usize][mv.to() as usize])
    }

    /*----------------------------------------------------------------*/

    pub fn get_follow_up(
        &self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<i32> {
        let prev_mv = prev_mv?;

        Some(self.follow_up[board.stm() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from()).unwrap() as usize][mv.to() as usize])
    }

    fn get_follow_up_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<&mut i32> {
        let prev_mv = prev_mv?;

        Some(&mut self.follow_up[board.stm() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from()).unwrap() as usize][mv.to() as usize])
    }

    /*----------------------------------------------------------------*/
    
    #[inline]
    pub fn get_non_tactical(
        &self,
        board: &Board,
        mv: Move,
        indices: &ContIndices,
        weights: &SearchWeights,
    ) -> i32 {
        self.get_quiet(board, mv)
            + weights.cont1_frac * self.get_counter_move(board, mv, indices.counter_move).unwrap_or_default() / 512
            + weights.cont3_frac * self.get_counter_move(board, mv, indices.counter_move2).unwrap_or_default() / 512
            + weights.cont2_frac * self.get_follow_up(board, mv, indices.follow_up).unwrap_or_default() / 512
    }

    #[inline]
    pub fn get_corr(&self, board: &Board, weights: &SearchWeights) -> i16 {
        let pawn_hash = board.pawn_hash();
        let stm = board.stm();

        weights.pawn_corr_frac * self.pawn_corr[stm as usize][pawn_hash as usize % PAWN_CORRECTION_SIZE] / MAX_CORRECTION
    }
    
    /*----------------------------------------------------------------*/
    
    pub fn update(
        &mut self,
        board: &Board,
        indices: &ContIndices,
        weights: &SearchWeights,
        best_move: Move,
        quiets: &[Move],
        tactics: &[Move],
        depth: u8
    ) {
        if board.is_tactical(best_move) {
            History::update_value(
                self.get_tactical_mut(board, best_move),
                weights.tactic_bonus_base + weights.tactic_bonus_mul * depth as i32
            );
        } else {
            History::update_value(
                self.get_quiet_mut(board, best_move),
                weights.quiet_bonus_base + weights.quiet_bonus_mul * depth as i32
            );
            
            for &mv in quiets {
                History::update_value(
                    self.get_quiet_mut(board, mv),
                    -weights.quiet_malus_base - weights.quiet_malus_mul * depth as i32
                );
            }
            
            if let Some(value) = self.get_counter_move_mut(board, best_move, indices.counter_move) {
                History::update_value(
                    value,
                    weights.cont1_bonus_base + weights.cont1_bonus_mul * depth as i32
                );
                
                for &mv in quiets {
                    History::update_value(
                        self.get_counter_move_mut(board, mv, indices.counter_move).unwrap(),
                        -weights.cont1_malus_base - weights.cont1_malus_mul * depth as i32
                    );
                }
            }

            if let Some(value) = self.get_follow_up_mut(board, best_move, indices.follow_up) {
                History::update_value(
                    value,
                    weights.cont2_bonus_base + weights.cont2_bonus_mul * depth as i32
                );

                for &mv in quiets {
                    History::update_value(
                        self.get_follow_up_mut(board, mv, indices.follow_up).unwrap(),
                        -weights.cont2_malus_base - weights.cont2_malus_mul * depth as i32
                    );
                }
            }

            if let Some(value) = self.get_counter_move_mut(board, best_move, indices.counter_move2) {
                History::update_value(
                    value,
                    -weights.cont3_bonus_base - weights.cont3_bonus_mul * depth as i32
                );

                for &mv in quiets {
                    History::update_value(
                        self.get_counter_move_mut(board, mv, indices.counter_move2).unwrap(),
                        -weights.cont3_malus_base - weights.cont3_malus_mul * depth as i32
                    );
                }
            }
        }

        for &mv in tactics {
            History::update_value(
                self.get_tactical_mut(board, mv),
                -weights.tactic_malus_base - weights.tactic_malus_mul * depth as i32
            );
        }
    }

    pub fn update_corr(&mut self, board: &Board, depth: u8, best_score: Score, static_eval: Score) {
        let amount = (best_score - static_eval).0 * depth as i16 / 8;
        let pawn_hash = board.pawn_hash();
        let stm = board.stm();

        History::update_corr_value(
            &mut self.pawn_corr[stm as usize][pawn_hash as usize % PAWN_CORRECTION_SIZE],
            amount,
        );
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn update_value(value: &mut i32, amount: i32) {
        let amount = amount.clamp(-MAX_HISTORY, MAX_HISTORY);
        let decay = *value * amount.abs() / MAX_HISTORY;

        *value += amount - decay;
    }

    #[inline]
    fn update_corr_value(value: &mut i16, amount: i16) {
        let amount = amount.clamp(-MAX_CORRECTION / 4, MAX_CORRECTION / 4);
        let decay = (*value as i32 * amount.abs() as i32 / MAX_CORRECTION as i32) as i16;

        *value += amount - decay;
    }
}