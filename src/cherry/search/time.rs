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
    Nodes(u64),
    Depth(u8),
}

/*----------------------------------------------------------------*/

pub struct TimeManager {
    start: AtomicInstant,
    infinite: AtomicBool,
    check_time: AtomicBool,
    no_manage: AtomicBool,
    stop: AtomicU32,

    base_time: AtomicU64,
    soft_time: AtomicU64,
    hard_time: AtomicU64,

    soft_nodes: AtomicU64,
    hard_nodes: AtomicU64,
    depth: AtomicU8,
}

impl TimeManager {
    #[inline]
    pub fn new() -> Self {
        TimeManager {
            start: AtomicInstant::now(),
            infinite: AtomicBool::new(true),
            check_time: AtomicBool::new(false),
            no_manage: AtomicBool::new(false),
            stop: AtomicU32::new(0),

            base_time: AtomicU64::new(0),
            soft_time: AtomicU64::new(0),
            hard_time: AtomicU64::new(0),

            soft_nodes: AtomicU64::new(u64::MAX),
            hard_nodes: AtomicU64::new(u64::MAX),
            depth: AtomicU8::new(MAX_DEPTH),
        }
    }

    #[inline]
    pub fn init(&self, stm: Color, limits: &[SearchLimit], overhead: u64, soft_target: bool) {
        self.set_stop(false);

        let mut inc = [0; Color::COUNT];
        let mut time = [u64::MAX; Color::COUNT];
        let mut moves_to_go = None;
        let mut move_time = None;
        let mut nodes = u64::MAX;
        let mut depth = MAX_DEPTH;
        let mut infinite = true;
        let mut check_time = false;
        let mut no_manage = false;

        for limit in limits {
            use SearchLimit::*;

            match *limit {
                SearchMoves(_) => {}
                WhiteTime(t) => time[Color::White] = t,
                BlackTime(t) => time[Color::Black] = t,
                WhiteInc(i) => inc[Color::White] = i,
                BlackInc(i) => inc[Color::Black] = i,
                MoveTime(t) => move_time = Some(t),
                MovesToGo(n) => moves_to_go = Some(n),
                Nodes(n) => nodes = nodes.min(n),
                Depth(d) => depth = depth.min(d),
            }

            if matches!(
                limit,
                WhiteTime(..) | BlackTime(..) | MoveTime(..) | Depth(..) | Nodes(..)
            ) {
                infinite = false;
            }

            if matches!(limit, WhiteTime(..) | BlackTime(..) | MoveTime(..)) {
                check_time = true;
            }
            
            if matches!(limit, MoveTime(..)) {
                no_manage = true;
            }
        }

        self.infinite.store(infinite, Ordering::Relaxed);
        self.check_time.store(check_time, Ordering::Relaxed);
        self.no_manage.store(no_manage, Ordering::Relaxed);

        self.depth.store(depth, Ordering::Relaxed);
        self.soft_nodes.store(nodes, Ordering::Relaxed);
        if soft_target {
            self.hard_nodes
                .store(nodes.saturating_mul(2000), Ordering::Relaxed);
        } else {
            self.hard_nodes.store(nodes, Ordering::Relaxed);
        }

        if let Some(time) = move_time {
            self.base_time.store(time, Ordering::Relaxed);
            self.soft_time.store(time, Ordering::Relaxed);

            if soft_target {
                self.hard_time
                    .store(time.saturating_mul(2).max(time + 2000), Ordering::Relaxed);
            } else {
                self.hard_time.store(time, Ordering::Relaxed);
            }
        } else if let Some(moves_to_go) = moves_to_go {
            let (time, inc) = (time[stm].saturating_sub(overhead), inc[stm]);
            let hard_time =
                ((time as f64 / (W::hard_time_div() as f64 / 4096.0)) as u64 + inc).min(time);
            let soft_time = (time / moves_to_go as u64 + inc).min(hard_time);

            self.base_time.store(soft_time, Ordering::Relaxed);
            self.soft_time.store(soft_time, Ordering::Relaxed);
            self.hard_time.store(hard_time, Ordering::Relaxed);
        } else {
            let (time, inc) = (time[stm].saturating_sub(overhead), inc[stm]);
            let hard_time = ((time as f64 / (W::hard_time_div() as f64 / 4096.0)) as u64
                + inc * W::hard_time_inc() / 4096)
                .min(time);
            let soft_time = ((time as f64 / (W::soft_time_div() as f64 / 4096.0)) as u64
                + inc * W::soft_time_inc() / 4096)
                .min(hard_time);

            self.base_time.store(soft_time, Ordering::Relaxed);
            self.soft_time.store(soft_time, Ordering::Relaxed);
            self.hard_time.store(hard_time, Ordering::Relaxed);
        }

        self.start.store(Instant::now(), Ordering::Relaxed);
    }

    #[inline]
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
    pub fn set_stop(&self, stop: bool) {
        self.stop.store(stop as u32, Ordering::Relaxed);
        if self.infinite() {
            atomic_wait::wake_all(&self.stop);
        }
    }

    #[inline]
    pub fn wait_for_stop(&self) {
        while !self.should_stop() {
            atomic_wait::wait(&self.stop, 0);
        }
    }

    #[inline]
    pub fn stop_search(&self, thread: &ThreadData) -> bool {
        self.should_stop()
            || thread.nodes.global() >= self.hard_nodes.load(Ordering::Relaxed)
            || (thread.nodes.local().is_multiple_of(1024)
            && thread.id == 0
            && self.check_time.load(Ordering::Relaxed)
            && self.elapsed().as_millis() as u64 > self.hard_time.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn stop_id(&self, depth: u8, nodes: u64) -> bool {
        self.should_stop()
            || depth >= self.depth.load(Ordering::Relaxed)
            || nodes >= self.soft_nodes.load(Ordering::Relaxed)
            || (self.check_time.load(Ordering::Relaxed)
            && self.elapsed().as_millis() as u64 > self.soft_time.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.load(Ordering::Relaxed).elapsed()
    }

    #[inline]
    pub fn should_stop(&self) -> bool {
        self.stop.load(Ordering::Relaxed) != 0
    }

    #[inline]
    pub fn infinite(&self) -> bool {
        self.infinite.load(Ordering::Relaxed)
    }
}