use crate::*;

/*----------------------------------------------------------------*/

pub const MAX_CORR: i32 = 1024;
pub const MAX_HISTORY: i32 = 16384;

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct ContIndices {
    pub counter_move: Option<MoveData>,
    pub follow_up: Option<MoveData>,
}

impl ContIndices {
    #[inline]
    pub fn new(pos: &Position) -> ContIndices {
        ContIndices {
            counter_move: pos.prev_move(1),
            follow_up: pos.prev_move(2),
        }
    }
}

/*----------------------------------------------------------------*/

#[inline]
fn gravity<const MAX_BONUS: i32, const MAX_VALUE: i32>(entry: &mut i16, amount: i32) {
    let amount = amount.clamp(-MAX_BONUS, MAX_BONUS);
    let decay = (*entry as i32 * amount.abs() / MAX_VALUE) as i16;
    *entry += amount as i16 - decay;
}

/*----------------------------------------------------------------*/

type ThreatBuckets<T> = [T; 4];

#[inline]
fn threat_index(board: &Board, mv: Move) -> usize {
    let stm = board.stm();
    let (src, dest) = (mv.src(), mv.dest());
    let src_threatened = !board.attack_table(!stm).get(src).is_empty();
    let dest_threatened = !board.attack_table(!stm).get(dest).is_empty();

    2 * src_threatened as usize + dest_threatened as usize
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct QuietEntry {
    buckets: ThreatBuckets<i16>,
}

#[derive(Debug, Copy, Clone)]
pub struct QuietHistory {
    entries: [[[QuietEntry; Square::COUNT]; Square::COUNT]; Color::COUNT], // [stm][src][dest][threat bucket]
}

impl QuietHistory {
    #[inline]
    pub fn bonus(depth: i32) -> i32 {
        let base = W::quiet_bonus_base();
        let scale = W::quiet_bonus_scale();
        let max = W::quiet_bonus_max();

        (base + scale * depth / DEPTH_SCALE).min(max)
    }

    #[inline]
    pub fn malus(depth: i32) -> i32 {
        let base = W::quiet_malus_base();
        let scale = W::quiet_malus_scale();
        let max = W::quiet_malus_max();

        (base + scale * depth / DEPTH_SCALE).min(max)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn entry(&self, board: &Board, mv: Move) -> i16 {
        let stm = board.stm();
        let (src, dest) = (mv.src(), mv.dest());
        let threat_index = threat_index(board, mv);

        self.entries[stm][src][dest].buckets[threat_index]
    }

    #[inline]
    pub fn entry_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        let stm = board.stm();
        let (src, dest) = (mv.src(), mv.dest());
        let threat_index = threat_index(board, mv);

        &mut self.entries[stm][src][dest].buckets[threat_index]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn update(&mut self, board: &Board, depth: i32, mv: Move, bonus: bool) {
        let amount = if bonus {
            Self::bonus(depth)
        } else {
            -Self::malus(depth)
        };

        gravity::<MAX_HISTORY, MAX_HISTORY>(self.entry_mut(board, mv), amount);
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct TacticEntry(i16);

#[derive(Debug, Copy, Clone)]
pub struct TacticHistory {
    entries: [[[TacticEntry; Square::COUNT]; Piece::COUNT]; Color::COUNT], // [stm][piece][dest]
}

impl TacticHistory {
    #[inline]
    pub fn bonus(depth: i32) -> i32 {
        let base = W::tactic_bonus_base();
        let scale = W::tactic_bonus_scale();
        let max = W::tactic_bonus_max();

        (base + scale * depth / DEPTH_SCALE).min(max)
    }

    #[inline]
    pub fn malus(depth: i32) -> i32 {
        let base = W::tactic_malus_base();
        let scale = W::tactic_malus_scale();
        let max = W::tactic_malus_max();

        (base + scale * depth / DEPTH_SCALE).min(max)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn entry(&self, board: &Board, mv: Move) -> i16 {
        let stm = board.stm();
        let piece = board.piece_on(mv.src()).unwrap();
        let dest = mv.dest();

        self.entries[stm][piece][dest].0
    }

    #[inline]
    pub fn entry_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        let stm = board.stm();
        let piece = board.piece_on(mv.src()).unwrap();
        let dest = mv.dest();

        &mut self.entries[stm][piece][dest].0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn update(&mut self, board: &Board, depth: i32, mv: Move, bonus: bool) {
        let amount = if bonus {
            Self::bonus(depth)
        } else {
            -Self::malus(depth)
        };

        gravity::<MAX_HISTORY, MAX_HISTORY>(self.entry_mut(board, mv), amount);
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct PawnEntry(i16);

#[derive(Debug, Copy, Clone)]
pub struct PawnHistory<const SIZE: usize> {
    entries: [[[[PawnEntry; Square::COUNT]; Piece::COUNT]; SIZE]; Color::COUNT], // [stm][pawn hash % size][piece][dest]
}

impl<const SIZE: usize> PawnHistory<SIZE> {
    #[inline]
    pub fn bonus(depth: i32) -> i32 {
        let base = W::pawn_bonus_base();
        let scale = W::pawn_bonus_scale();
        let max = W::pawn_bonus_max();

        (base + scale * depth / DEPTH_SCALE).min(max)
    }

    #[inline]
    pub fn malus(depth: i32) -> i32 {
        let base = W::pawn_malus_base();
        let scale = W::pawn_malus_scale();
        let max = W::pawn_malus_max();

        (base + scale * depth / DEPTH_SCALE).min(max)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn entry(&self, board: &Board, mv: Move) -> i16 {
        let stm = board.stm();
        let pawn_hash = board.pawn_hash();
        let piece = board.piece_on(mv.src()).unwrap();
        let dest = mv.dest();

        self.entries[stm][(pawn_hash % SIZE as u64) as usize][piece][dest].0
    }

    #[inline]
    pub fn entry_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        let stm = board.stm();
        let pawn_hash = board.pawn_hash();
        let piece = board.piece_on(mv.src()).unwrap();
        let dest = mv.dest();

        &mut self.entries[stm][(pawn_hash % SIZE as u64) as usize][piece][dest].0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn update(&mut self, board: &Board, depth: i32, mv: Move, bonus: bool) {
        let amount = if bonus {
            Self::bonus(depth)
        } else {
            -Self::malus(depth)
        };

        gravity::<MAX_HISTORY, MAX_HISTORY>(self.entry_mut(board, mv), amount);
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct ContEntry(i16);

#[derive(Debug, Copy, Clone)]
pub struct ContHistory {
    entries:
        [[[[[ContEntry; Square::COUNT]; Piece::COUNT]; Square::COUNT]; Piece::COUNT]; Color::COUNT], // [stm][prev piece][prev dest][piece][dest]
}

impl ContHistory {
    #[inline]
    pub fn bonus<const PLY: usize>(depth: i32) -> i32 {
        let (base, scale, max) = match PLY {
            1 => (
                W::cont1_bonus_base(),
                W::cont1_bonus_scale(),
                W::cont1_bonus_max(),
            ),
            2 => (
                W::cont2_bonus_base(),
                W::cont2_bonus_scale(),
                W::cont2_bonus_max(),
            ),
            _ => unreachable!(),
        };

        (base + scale * depth / DEPTH_SCALE).min(max)
    }

    #[inline]
    pub fn malus<const PLY: usize>(depth: i32) -> i32 {
        let (base, scale, max) = match PLY {
            1 => (
                W::cont1_malus_base(),
                W::cont1_malus_scale(),
                W::cont1_malus_max(),
            ),
            2 => (
                W::cont2_malus_base(),
                W::cont2_malus_scale(),
                W::cont2_malus_max(),
            ),
            _ => unreachable!(),
        };

        (base + scale * depth / DEPTH_SCALE).min(max)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn entry(&self, board: &Board, mv: Move, prev_mv: Option<MoveData>) -> Option<i16> {
        let prev_mv = prev_mv?;

        let stm = board.stm();
        let piece = board.piece_on(mv.src()).unwrap();
        let dest = mv.dest();

        Some(self.entries[stm][prev_mv.piece][prev_mv.mv.dest()][piece][dest].0)
    }

    #[inline]
    pub fn entry_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>,
    ) -> Option<&mut i16> {
        let prev_mv = prev_mv?;

        let stm = board.stm();
        let piece = board.piece_on(mv.src()).unwrap();
        let dest = mv.dest();

        Some(&mut self.entries[stm][prev_mv.piece][prev_mv.mv.dest()][piece][dest].0)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn update<const PLY: usize>(
        &mut self,
        board: &Board,
        depth: i32,
        mv: Move,
        prev_mv: Option<MoveData>,
        bonus: bool,
    ) {
        let amount = if bonus {
            Self::bonus::<PLY>(depth)
        } else {
            -Self::malus::<PLY>(depth)
        };

        if let Some(entry) = self.entry_mut(board, mv, prev_mv) {
            gravity::<MAX_HISTORY, MAX_HISTORY>(entry, amount);
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct CorrEntry(i16);

#[derive(Debug, Copy, Clone)]
pub struct CorrHistory<const SIZE: usize> {
    entries: [[CorrEntry; SIZE]; Color::COUNT],
}

impl<const SIZE: usize> CorrHistory<SIZE> {
    #[inline]
    pub fn bonus(depth: i32, diff: i64) -> i32 {
        (diff * depth as i64 * W::corr_bonus_scale() / (DEPTH_SCALE as i64 * 1024)) as i32
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn entry(&self, stm: Color, hash: u64) -> i16 {
        self.entries[stm][(hash % SIZE as u64) as usize].0
    }

    #[inline]
    pub fn entry_mut(&mut self, stm: Color, hash: u64) -> &mut i16 {
        &mut self.entries[stm][(hash % SIZE as u64) as usize].0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn update(&mut self, stm: Color, hash: u64, depth: i32, diff: i64) {
        gravity::<{ MAX_CORR / 4 }, MAX_CORR>(self.entry_mut(stm, hash), Self::bonus(depth, diff));
    }
}

/*----------------------------------------------------------------*/

pub const PAWN_HIST_SIZE: usize = 4096;

pub const PAWN_CORR_SIZE: usize = 4096;
pub const MINOR_CORR_SIZE: usize = 16384;
pub const MAJOR_CORR_SIZE: usize = 16384;
pub const NONPAWN_CORR_SIZE: usize = 16384;

#[derive(Debug, Copy, Clone)]
pub struct History {
    quiet: QuietHistory,
    tactic: TacticHistory,
    pawn: PawnHistory<PAWN_HIST_SIZE>,
    counter_move: ContHistory,
    follow_up: ContHistory,
    pawn_corr: CorrHistory<PAWN_CORR_SIZE>,
    minor_corr: CorrHistory<MINOR_CORR_SIZE>,
    major_corr: CorrHistory<MAJOR_CORR_SIZE>,
    white_corr: CorrHistory<NONPAWN_CORR_SIZE>,
    black_corr: CorrHistory<NONPAWN_CORR_SIZE>,
}

impl History {
    #[inline]
    pub fn get_quiet(&self, board: &Board, indices: &ContIndices, mv: Move) -> i32 {
        let mut result = self.quiet.entry(board, mv) as i32;
        result += self.pawn.entry(board, mv) as i32;
        result += self
            .counter_move
            .entry(board, mv, indices.counter_move)
            .unwrap_or_default() as i32;
        result += self
            .follow_up
            .entry(board, mv, indices.follow_up)
            .unwrap_or_default() as i32;

        result
    }

    #[inline]
    pub fn get_tactic(&self, board: &Board, mv: Move) -> i32 {
        self.tactic.entry(board, mv) as i32
    }

    #[inline]
    pub fn get_corr(&self, board: &Board) -> i32 {
        let stm = board.stm();
        let (white_frac, black_frac) = match stm {
            Color::White => (W::stm_corr_frac(), W::ntm_corr_frac()),
            Color::Black => (W::ntm_corr_frac(), W::stm_corr_frac()),
        };

        let mut corr = 0;

        corr += W::pawn_corr_frac() * self.pawn_corr.entry(stm, board.pawn_hash()) as i32;
        corr += W::minor_corr_frac() * self.minor_corr.entry(stm, board.minor_hash()) as i32;
        corr += W::major_corr_frac() * self.major_corr.entry(stm, board.major_hash()) as i32;
        corr += white_frac * self.white_corr.entry(stm, board.white_hash()) as i32;
        corr += black_frac * self.black_corr.entry(stm, board.black_hash()) as i32;

        corr / MAX_CORR
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn update_history(
        &mut self,
        board: &Board,
        indices: &ContIndices,
        depth: i32,
        best_move: Move,
        quiets: &[Move],
        tactics: &[Move],
    ) {
        if best_move.is_tactic() {
            self.update_tactic(board, depth, best_move, true);
        } else {
            self.update_quiet(board, depth, best_move, true);
            self.update_cont(board, indices, depth, best_move, true);

            for &quiet in quiets {
                self.update_quiet(board, depth, quiet, false);
                self.update_cont(board, indices, depth, quiet, false);
            }
        }

        for &tactic in tactics {
            self.update_tactic(board, depth, tactic, false);
        }
    }

    #[inline]
    pub fn update_quiet(&mut self, board: &Board, depth: i32, mv: Move, bonus: bool) {
        self.quiet.update(board, depth, mv, bonus);
        self.pawn.update(board, depth, mv, bonus);
    }

    #[inline]
    pub fn update_tactic(&mut self, board: &Board, depth: i32, mv: Move, bonus: bool) {
        self.tactic.update(board, depth, mv, bonus);
    }

    #[inline]
    pub fn update_cont(
        &mut self,
        board: &Board,
        indices: &ContIndices,
        depth: i32,
        mv: Move,
        bonus: bool,
    ) {
        self.counter_move
            .update::<1>(board, depth, mv, indices.counter_move, bonus);
        self.follow_up
            .update::<2>(board, depth, mv, indices.follow_up, bonus);
    }

    #[inline]
    pub fn update_corr(&mut self, board: &Board, depth: i32, score: Score, static_eval: Score) {
        let stm = board.stm();
        let diff = score.0 as i64 - static_eval.0 as i64;
        self.pawn_corr.update(stm, board.pawn_hash(), depth, diff);
        self.minor_corr.update(stm, board.minor_hash(), depth, diff);
        self.major_corr.update(stm, board.major_hash(), depth, diff);
        self.white_corr.update(stm, board.white_hash(), depth, diff);
        self.black_corr.update(stm, board.black_hash(), depth, diff);
    }
}
