use std::{sync::atomic::*, time::*};

use crate::*;

/*----------------------------------------------------------------*/

pub const DEFAULT_OVERHEAD: u64 = 100;

#[derive(Debug, Clone)]
pub enum SearchLimit {
    SearchMoves(MoveList),
    WhiteTime(u64),
    BlackTime(u64),
    WhiteInc(u64),
    BlackInc(u64),
    MoveTime(u64),
    MovesToGo(u16),
    MaxDepth(u8),
    MaxNodes(u64),
    Ponder,
}

pub struct TimeManager {
    start: AtomicInstant,
    abort_now: AtomicU32,
    no_manage: AtomicBool,
    infinite: AtomicBool,

    base_time: AtomicU64,
    soft_time: AtomicU64,
    hard_time: AtomicU64,
    soft_nodes: AtomicU64,
    hard_nodes: AtomicU64,
    max_depth: AtomicU8,
}

impl TimeManager {
    #[inline]
    pub fn new() -> TimeManager {
        TimeManager {
            start: AtomicInstant::new(Instant::now()),
            abort_now: AtomicU32::new(0),
            no_manage: AtomicBool::new(true),
            infinite: AtomicBool::new(true),
            base_time: AtomicU64::new(0),
            soft_time: AtomicU64::new(0),
            hard_time: AtomicU64::new(0),
            soft_nodes: AtomicU64::new(u64::MAX),
            hard_nodes: AtomicU64::new(u64::MAX),
            max_depth: AtomicU8::new(MAX_DEPTH),
        }
    }

    /*----------------------------------------------------------------*/

    pub fn init(&self, stm: Color, limits: &[SearchLimit], overhead: u64, soft_target: bool) {
        self.set_abort(false);

        let mut w_time = u64::MAX;
        let mut b_time = u64::MAX;
        let mut w_inc = 0;
        let mut b_inc = 0;
        let mut move_time = None;
        let mut moves_to_go = None;
        let mut max_depth = MAX_DEPTH;
        let mut max_nodes = u64::MAX;
        let mut infinite = true;
        let mut ponder = false;

        for limit in limits {
            match limit {
                SearchLimit::SearchMoves(_) => {}
                SearchLimit::WhiteTime(time) => w_time = *time,
                SearchLimit::BlackTime(time) => b_time = *time,
                SearchLimit::WhiteInc(inc) => w_inc = *inc,
                SearchLimit::BlackInc(inc) => b_inc = *inc,
                SearchLimit::MoveTime(time) => move_time = Some(*time),
                SearchLimit::MovesToGo(num) => moves_to_go = Some(*num),
                SearchLimit::MaxDepth(depth) => max_depth = (*depth).min(MAX_DEPTH),
                SearchLimit::MaxNodes(nodes) => max_nodes = *nodes,
                SearchLimit::Ponder => ponder = true,
            }

            if matches!(
                limit,
                SearchLimit::WhiteTime(_)
                    | SearchLimit::BlackTime(_)
                    | SearchLimit::WhiteInc(_)
                    | SearchLimit::BlackInc(_)
                    | SearchLimit::MoveTime(_)
            ) {
                infinite = false;
            }
        }

        self.infinite.store(infinite | ponder, Ordering::Relaxed);
        self.no_manage
            .store(soft_target | infinite, Ordering::Relaxed);

        self.max_depth.store(max_depth, Ordering::Relaxed);
        self.soft_nodes.store(max_nodes, Ordering::Relaxed);
        if soft_target {
            self.hard_nodes
                .store(max_nodes.saturating_mul(2000), Ordering::Relaxed);
        } else {
            self.hard_nodes.store(max_nodes, Ordering::Relaxed);
        }

        if let Some(time) = move_time {
            let hard_time = if soft_target { u64::MAX } else { time };

            self.base_time.store(time, Ordering::Relaxed);
            self.soft_time.store(time, Ordering::Relaxed);
            self.hard_time.store(hard_time, Ordering::Relaxed);
        } else if let Some(moves_to_go) = moves_to_go {
            let (time, inc) = match stm {
                Color::White => (w_time.saturating_sub(overhead), w_inc),
                Color::Black => (b_time.saturating_sub(overhead), b_inc),
            };

            let hard_time =
                ((time as f64 / (W::hard_time_div() as f64 / 4096.0)) as u64 + inc).min(time);
            let soft_time = (time / moves_to_go as u64 + inc).min(hard_time);

            self.base_time.store(soft_time, Ordering::Relaxed);
            self.soft_time.store(soft_time, Ordering::Relaxed);
            self.hard_time.store(hard_time, Ordering::Relaxed);
        } else {
            let (time, inc) = match stm {
                Color::White => (w_time.saturating_sub(overhead), w_inc),
                Color::Black => (b_time.saturating_sub(overhead), b_inc),
            };

            let hard_time =
                ((time as f64 / (W::hard_time_div() as f64 / 4096.0)) as u64 + inc).min(time);
            let soft_time =
                ((time as f64 / (W::soft_time_div() as f64 / 4096.0)) as u64 + inc).min(hard_time);

            self.base_time.store(soft_time, Ordering::Relaxed);
            self.soft_time.store(soft_time, Ordering::Relaxed);
            self.hard_time.store(hard_time, Ordering::Relaxed);
        }

        self.start.store(Instant::now(), Ordering::Relaxed);
    }

    pub fn deepen(
        &self,
        depth: u8,
        _score: Score,
        _static_eval: Score,
        move_stability: u8,
        score_stability: u8,
        move_nodes: u64,
        nodes: u64,
    ) {
        if depth < 4 || self.no_manage.load(Ordering::Relaxed) {
            return;
        }

        let move_stability_factor = (W::move_stability_base()
            - W::move_stability_scale() * move_stability as i64)
            .max(W::move_stability_min());
        let score_stability_factor = (W::score_stability_base()
            - W::score_stability_scale() * score_stability as i64)
            .max(W::score_stability_min());
        let subtree_factor = (W::subtree_base()
            - (W::subtree_scale() as f64 * move_nodes as f64 / nodes as f64) as i64)
            .max(W::subtree_min());

        let base_time = self.base_time.load(Ordering::Relaxed);
        let hard_time = self.hard_time.load(Ordering::Relaxed);
        let new_target = ((base_time as u128
            * move_stability_factor as u128
            * score_stability_factor as u128
            * subtree_factor as u128)
            / 4096u128.pow(3)) as u64;

        self.soft_time
            .store(new_target.min(hard_time), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_abort(&self, value: bool) {
        self.abort_now.store(value as u32, Ordering::Relaxed);
        if self.is_infinite() {
            atomic_wait::wake_all(&self.abort_now);
        }
    }

    #[inline]
    pub fn wait_for_abort(&self) {
        while !self.should_abort() {
            atomic_wait::wait(&self.abort_now, 0);
        }
    }

    #[inline]
    pub fn ponderhit(&self) {
        self.infinite.store(false, Ordering::Relaxed);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn should_abort(&self) -> bool {
        self.abort_now.load(Ordering::Relaxed) != 0
    }

    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.infinite.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn elapsed(&self) -> u64 {
        self.start.load(Ordering::Relaxed).elapsed().as_millis() as u64
    }

    #[inline]
    pub fn abort_id(&self, depth: u8, nodes: u64) -> bool {
        self.should_abort()
            || depth >= self.max_depth.load(Ordering::Relaxed)
            || nodes >= self.soft_nodes.load(Ordering::Relaxed)
            || (!self.is_infinite() && self.elapsed() >= self.soft_time.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn abort_search(&self, nodes: &BatchedAtomicCounter) -> bool {
        self.should_abort()
            || nodes.global() >= self.hard_nodes.load(Ordering::Relaxed)
            || (!self.is_infinite()
                && nodes.local().is_multiple_of(1024)
                && self.elapsed() >= self.hard_time.load(Ordering::Relaxed))
    }
}
