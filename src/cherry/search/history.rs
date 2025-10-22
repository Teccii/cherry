use crate::*;

/*----------------------------------------------------------------*/

pub const MAX_HISTORY: i32 = 16384;
pub const MAX_CORR: i32 = 1024;

const MINOR_CORR_SIZE: usize = 16384;
const MAJOR_CORR_SIZE: usize = 16384;
const PAWN_CORR_SIZE: usize = 1024;

/*----------------------------------------------------------------*/

pub type ColorTo<T> = [T; Color::COUNT];
pub type MoveTo<T> = [[T; Square::COUNT]; Square::COUNT];
pub type PieceTo<T> = [[T; Square::COUNT]; Piece::COUNT];

/*----------------------------------------------------------------*/

#[inline]
pub const fn move_to<T: Copy>(default: T) -> MoveTo<T> {
    [[default; Square::COUNT]; Square::COUNT]
}

#[inline]
pub const fn piece_to<T: Copy>(default: T) -> PieceTo<T> {
    [[default; Square::COUNT]; Piece::COUNT]
}

#[inline]
fn delta(depth: u8, base: i32, mul: i32, max: i32) -> i32 {
    i32::min(base + mul * depth as i32, max)
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct ContIndices {
    pub counter_move: Option<MoveData>,
    pub follow_up: Option<MoveData>,
}

impl ContIndices {
    #[inline]
    pub fn new(ss: &[SearchStack], ply: u16) -> ContIndices {
        ContIndices {
            counter_move: (ply >= 1).then(|| ss[ply as usize - 1].move_played).flatten(),
            follow_up: (ply >= 2).then(|| ss[ply as usize - 2].move_played).flatten(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct History {
    quiets:  Box<ColorTo<MoveTo<i32>>>,  //Indexing: [stm][from][to]
    tactics: Box<ColorTo<PieceTo<[i32; 2]>>>, //Indexing: [stm][piece][to][see index]
    counter_move: Box<ColorTo<PieceTo<PieceTo<i32>>>>, //use for 1-ply, 3-ply, 5-ply, etc. Indexing: [stm][prev piece][prev to][piece][to]
    follow_up:    Box<ColorTo<PieceTo<PieceTo<i32>>>>, //use for 2-ply, 4-ply, 6-ply, etc. Indexing: [stm][prev piece][prev to][piece][to]
    minor_corr: Box<ColorTo<[i32; MINOR_CORR_SIZE]>>, //Indexing: [stm][minor hash % size]
    major_corr: Box<ColorTo<[i32; MAJOR_CORR_SIZE]>>, //Indexing: [stm][major hash % size]
    pawn_corr:  Box<ColorTo<[i32; PAWN_CORR_SIZE]>> //Indexing: [stm][pawn hash % size]
}

impl History {
    #[inline]
    pub fn new() -> History {
        History {
            quiets: new_zeroed(),
            tactics: new_zeroed(),
            counter_move: new_zeroed(),
            follow_up: new_zeroed(),
            minor_corr: new_zeroed(),
            major_corr: new_zeroed(),
            pawn_corr: new_zeroed(),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.quiets.fill(move_to(0));
        self.tactics.fill(piece_to([0; 2]));
        self.counter_move.fill(piece_to(piece_to(0)));
        self.follow_up.fill(piece_to(piece_to(0)));
        self.minor_corr.fill([0; MINOR_CORR_SIZE]);
        self.major_corr.fill([0; MAJOR_CORR_SIZE]);
        self.pawn_corr.fill([0; PAWN_CORR_SIZE]);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_quiet(&self, board: &Board, mv: Move) -> i32 {
        self.quiets[board.stm()]
            [mv.from()]
            [mv.to()]
    }
    
    #[inline]
    fn get_quiet_mut(&mut self, board: &Board, mv: Move) -> &mut i32 {
        &mut self.quiets[board.stm()]
            [mv.from()]
            [mv.to()]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get_tactic(&self, board: &Board, mv: Move, see: bool) -> i32 {
        self.tactics[board.stm()]
            [board.piece_on(mv.from()).unwrap()]
            [mv.to()]
            [see as usize]
    }

    #[inline]
    fn get_tactic_mut(&mut self, board: &Board, mv: Move, see: bool) -> &mut i32 {
        &mut self.tactics[board.stm()]
            [board.piece_on(mv.from()).unwrap()]
            [mv.to()]
            [see as usize]
    }

    /*----------------------------------------------------------------*/

    pub fn get_counter_move(
        &self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<i32> {
        let prev_mv = prev_mv?;

        Some(self.counter_move[board.stm()]
            [prev_mv.piece][prev_mv.to]
            [board.piece_on(mv.from()).unwrap()][mv.to()])
    }

    fn get_counter_move_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<&mut i32> {
        let prev_mv = prev_mv?;

        Some(&mut self.counter_move[board.stm()]
            [prev_mv.piece][prev_mv.to]
            [board.piece_on(mv.from()).unwrap()][mv.to()])
    }

    /*----------------------------------------------------------------*/

    pub fn get_follow_up(
        &self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<i32> {
        let prev_mv = prev_mv?;

        Some(self.follow_up[board.stm()]
            [prev_mv.piece][prev_mv.to]
            [board.piece_on(mv.from()).unwrap()][mv.to()])
    }

    fn get_follow_up_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<&mut i32> {
        let prev_mv = prev_mv?;

        Some(&mut self.follow_up[board.stm()]
            [prev_mv.piece][prev_mv.to]
            [board.piece_on(mv.from()).unwrap()][mv.to()])
    }

    /*----------------------------------------------------------------*/
    
    #[inline]
    pub fn get_quiet_total(
        &self,
        board: &Board,
        mv: Move,
        indices: &ContIndices,
    ) -> i32 {
        self.get_quiet(board, mv)
            + self.get_counter_move(board, mv, indices.counter_move).unwrap_or_default()
            + self.get_follow_up(board, mv, indices.follow_up).unwrap_or_default()
    }

    #[inline]
    pub fn get_corr(&self, board: &Board) -> i32 {
        let stm = board.stm();
        let mut corr = 0;
        corr += W::pawn_corr_frac() * self.pawn_corr[stm][(board.pawn_hash() % PAWN_CORR_SIZE as u64) as usize];
        corr += W::major_corr_frac() * self.major_corr[stm][(board.major_hash() % MAJOR_CORR_SIZE as u64) as usize];
        corr += W::minor_corr_frac() * self.minor_corr[stm][(board.minor_hash() % MINOR_CORR_SIZE as u64) as usize];

        corr / MAX_CORR
    }
    
    /*----------------------------------------------------------------*/
    
    pub fn update(
        &mut self,
        board: &Board,
        indices: &ContIndices,
        best_move: Move,
        best_move_see: bool,
        quiets: &[Move],
        good_tactics: &[Move],
        bad_tactics: &[Move],
        depth: u8
    ) {
        if best_move.is_tactic() {
            History::update_value(
                self.get_tactic_mut(board, best_move, best_move_see),
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
            
            if let Some(value) = self.get_counter_move_mut(board, best_move, indices.counter_move) {
                History::update_value(
                    value,
                    delta(depth, W::cont1_bonus_base(), W::cont1_bonus_mul(), W::cont1_bonus_max())
                );
                
                for &mv in quiets {
                    History::update_value(
                        self.get_counter_move_mut(board, mv, indices.counter_move).unwrap(),
                        -delta(depth, W::cont1_malus_base(), W::cont1_malus_mul(), W::cont1_malus_max()),
                    );
                }
            }

            if let Some(value) = self.get_follow_up_mut(board, best_move, indices.follow_up) {
                History::update_value(
                    value,
                    delta(depth, W::cont2_bonus_base(), W::cont2_bonus_mul(), W::cont2_bonus_max()),
                );

                for &mv in quiets {
                    History::update_value(
                        self.get_follow_up_mut(board, mv, indices.follow_up).unwrap(),
                        -delta(depth, W::cont2_malus_base(), W::cont2_malus_mul(), W::cont2_malus_max()),
                    );
                }
            }
        }

        let tactic_malus = -delta(depth, W::tactic_malus_base(), W::tactic_malus_mul(), W::tactic_malus_max());
        for &mv in good_tactics {
            History::update_value(self.get_tactic_mut(board, mv, true), tactic_malus);
        }

        for &mv in bad_tactics {
            History::update_value(self.get_tactic_mut(board, mv, false), tactic_malus);
        }
    }

    pub fn update_corr(&mut self, board: &Board, depth: u8, best_score: Score, static_eval: Score) {
        let stm = board.stm();
        let pawn_corr = &mut self.pawn_corr[stm as usize][board.pawn_hash() as usize % PAWN_CORR_SIZE];
        let minor_corr = &mut self.minor_corr[stm as usize][board.minor_hash() as usize % MINOR_CORR_SIZE];
        let major_corr = &mut self.major_corr[stm as usize][board.major_hash() as usize % MAJOR_CORR_SIZE];
        let amount = (best_score - static_eval).0 as i32 * depth as i32 / 8;

        History::update_corr_value(pawn_corr, amount);
        History::update_corr_value(minor_corr, amount);
        History::update_corr_value(major_corr, amount);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn update_value(value: &mut i32, amount: i32) {
        let amount = amount.clamp(-MAX_HISTORY, MAX_HISTORY);
        let decay = *value * amount.abs() / MAX_HISTORY;

        *value += amount - decay;
    }

    #[inline]
    fn update_corr_value(value: &mut i32, amount: i32) {
        let amount = amount.clamp(-MAX_CORR / 4, MAX_CORR / 4);
        let decay = *value * amount.abs() / MAX_CORR;

        *value += amount - decay;
    }
}