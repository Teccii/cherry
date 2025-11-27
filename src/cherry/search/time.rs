use std::{sync::{atomic::*, Mutex}, time::*};
use atomic_time::AtomicInstant;
use crate::*;

/*----------------------------------------------------------------*/

pub const MOVE_OVERHEAD: u64 = 200;

/*----------------------------------------------------------------*/

pub struct TimeManager {
    start: AtomicInstant,
    base_time: AtomicU64,
    soft_time: AtomicU64,
    hard_time: AtomicU64,

    prev_move: Mutex<Option<Move>>,
    move_stability: AtomicU8,

    move_overhead: AtomicU64,
    use_max_depth: AtomicBool,
    use_max_nodes: AtomicBool,
    use_soft_nodes: AtomicBool,
    max_depth: AtomicU8,
    max_nodes: AtomicU64,

    no_manage: AtomicBool,
    pondering: AtomicBool,
    infinite: AtomicBool,
    abort_now: AtomicBool,
}

impl TimeManager {
    #[inline]
    pub fn new() -> TimeManager {
        TimeManager {
            start: AtomicInstant::new(Instant::now()),
            base_time: AtomicU64::new(0),
            soft_time: AtomicU64::new(0),
            hard_time: AtomicU64::new(0),
            prev_move: Mutex::new(None),
            move_stability: AtomicU8::new(0),
            move_overhead: AtomicU64::new(MOVE_OVERHEAD),
            use_max_depth: AtomicBool::new(false),
            use_max_nodes: AtomicBool::new(false),
            use_soft_nodes: AtomicBool::new(false),
            max_depth: AtomicU8::new(MAX_DEPTH),
            max_nodes: AtomicU64::new(u64::MAX),
            no_manage: AtomicBool::new(true),
            pondering: AtomicBool::new(false),
            infinite: AtomicBool::new(true),
            abort_now: AtomicBool::new(false),
        }
    }

    /*----------------------------------------------------------------*/

    pub fn init(&self, stm: Color, limits: &[SearchLimit]) {
        self.abort_now.store(false, Ordering::Relaxed);
        self.move_stability.store(0, Ordering::Relaxed);
        *self.prev_move.lock().unwrap() = None;

        let mut w_time = 0;
        let mut b_time = 0;
        let mut w_inc = 0;
        let mut b_inc = 0;
        let mut move_time = None;
        let mut moves_to_go = None;
        let mut max_depth = None;
        let mut max_nodes = None;
        let mut infinite = true;
        let mut pondering = false;

        for limit in limits {
            match limit {
                SearchLimit::SearchMoves(_) => { },
                SearchLimit::WhiteTime(time) => {
                    w_time = time.as_millis() as u64;
                    infinite = false;
                },
                SearchLimit::BlackTime(time) => {
                    b_time = time.as_millis() as u64;
                    infinite = false;
                },
                SearchLimit::WhiteInc(inc) => {
                    w_inc = inc.as_millis() as u64;
                    infinite = false;
                },
                SearchLimit::BlackInc(inc) => {
                    b_inc = inc.as_millis() as u64;
                    infinite = false;
                },
                SearchLimit::MoveTime(time) => {
                    move_time = Some(time.as_millis() as u64);
                    infinite = false;
                },
                SearchLimit::MovesToGo(moves) => {
                    moves_to_go = Some(*moves);
                },
                SearchLimit::MaxDepth(depth ) => {
                    max_depth = Some(*depth);
                },
                SearchLimit::MaxNodes(nodes ) => {
                    max_nodes = Some(*nodes);
                },
                SearchLimit::Infinite => {
                    infinite = true;
                },
                SearchLimit::Ponder => {
                    pondering = true;
                },
            }
        }

        let use_soft_nodes = self.use_soft_nodes();
        if use_soft_nodes && max_nodes.is_some() {
            infinite = false;
        }

        if pondering {
            infinite = true;
        }

        self.pondering.store(pondering, Ordering::Relaxed);
        self.infinite.store(infinite, Ordering::Relaxed);
        self.use_max_depth.store(max_depth.is_some(), Ordering::Relaxed);
        self.use_max_nodes.store(max_nodes.is_some(), Ordering::Relaxed);
        self.max_depth.store(max_depth.unwrap_or(MAX_DEPTH), Ordering::Relaxed);
        self.max_nodes.store(max_nodes.unwrap_or(u64::MAX), Ordering::Relaxed);
        self.no_manage.store(infinite || move_time.is_some(), Ordering::Relaxed);

        if let Some(time) = move_time {
            self.base_time.store(time, Ordering::Relaxed);
            self.soft_time.store(time, Ordering::Relaxed);
            self.hard_time.store(time, Ordering::Relaxed);
        } else if use_soft_nodes && let Some(nodes) = max_nodes {
            self.base_time.store(nodes, Ordering::Relaxed);
            self.soft_time.store(nodes, Ordering::Relaxed);
            self.hard_time.store(2000*nodes, Ordering::Relaxed);
        } else if let Some(moves_to_go) = moves_to_go {
            let (time, inc) = match stm {
                Color::White => (w_time, w_inc),
                Color::Black => (b_time, b_inc),
            };
            let move_overhead = self.move_overhead.load(Ordering::Relaxed);
            let hard_time = (time * W::hard_time_frac() / 16384).min(time.saturating_sub(move_overhead));
            let soft_time = ((time / moves_to_go as u64).saturating_sub(move_overhead) + inc).min(hard_time);

            self.base_time.store(soft_time, Ordering::Relaxed);
            self.soft_time.store(soft_time, Ordering::Relaxed);
            self.hard_time.store(hard_time, Ordering::Relaxed);
        } else {
            let (time, inc) = match stm {
                Color::White => (w_time, w_inc),
                Color::Black => (b_time, b_inc),
            };
            let move_overhead = self.move_overhead.load(Ordering::Relaxed);
            let hard_time = (time * W::hard_time_frac() / 16384).min(time.saturating_sub(move_overhead));
            let soft_time = ((time * W::soft_time_frac() / 16384).saturating_sub(move_overhead) + inc).min(hard_time);

            self.base_time.store(soft_time, Ordering::Relaxed);
            self.soft_time.store(soft_time, Ordering::Relaxed);
            self.hard_time.store(hard_time, Ordering::Relaxed);
        }

        self.start.store(Instant::now(), Ordering::Relaxed);
    }
    
    pub fn deepen(
        &self,
        depth: u8,
        score: Score,
        static_eval: Score,
        best_move: Move,
        move_nodes: u64,
        nodes: u64,
    ) {
        if depth < 4 || self.no_manage.load(Ordering::Relaxed) {
            return;
        }

        let mut prev_move = self.prev_move.lock().unwrap();
        let mut move_stability = self.move_stability.load(Ordering::Relaxed);

        move_stability = (move_stability + 1).min(8);
        if *prev_move != Some(best_move) {
            move_stability = 0;
        }

        *prev_move = Some(best_move);
        self.move_stability.store(move_stability, Ordering::Relaxed);

        let complexity = (static_eval - score).abs().0 as f32;

        let stability_factor = W::stability_tm_base() - W::stability_tm_scale() * move_stability as f32;
        let subtree_factor = W::subtree_tm_base() - W::subtree_tm_scale() * move_nodes as f32 / nodes as f32;
        let complexity_factor = if !score.is_decisive() {
            (W::complexity_tm_base() + W::complexity_tm_scale() * complexity * (depth as f32).ln()).clamp(W::complexity_tm_min(), W::complexity_tm_max())
        } else if score.is_win() {
            W::complexity_tm_win()
        } else {
            W::complexity_tm_loss()
        };
        
        let base_time = self.base_time.load(Ordering::Relaxed);
        let hard_time = self.hard_time.load(Ordering::Relaxed);
        let new_target = (base_time as f32 * stability_factor * subtree_factor * complexity_factor) as u64;

        self.soft_time.store(new_target.min(hard_time), Ordering::Relaxed);
    }

    #[inline]
    pub fn stop(&self) {
        self.abort_now.store(true, Ordering::Relaxed);
    }
    
    #[inline]
    pub fn ponderhit(&self) {
        self.pondering.store(false, Ordering::Relaxed);
        self.infinite.store(false, Ordering::Relaxed);
        self.no_manage.store(false, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_overhead(&self, millis: u64) {
        self.move_overhead.store(millis, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_soft_nodes(&self, value: bool) {
        self.use_soft_nodes.store(value, Ordering::Relaxed);
    }
    
    /*----------------------------------------------------------------*/

    #[inline]
    pub fn abort_now(&self) -> bool {
        self.abort_now.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn abort_search(&self, nodes: u64) -> bool {
        self.abort_now() || self.timeout_search(nodes)
            || (!self.use_soft_nodes() && self.use_max_nodes() && self.max_nodes.load(Ordering::Relaxed) <= nodes)
    }

    #[inline]
    pub fn abort_id(&self, depth: u8, nodes: u64) -> bool {
        self.abort_now() || self.timeout_id(nodes)
            || (self.use_max_depth() && self.max_depth.load(Ordering::Relaxed) <= depth)
            || (!self.use_soft_nodes() && self.use_max_nodes() && self.max_nodes.load(Ordering::Relaxed) <= nodes)
    }

    #[inline]
    pub fn timeout_search(&self, nodes: u64) -> bool {
        let elapsed = if self.use_soft_nodes() && self.use_max_nodes() {
            nodes
        } else {
            self.elapsed()
        };

        !self.is_infinite() && self.hard_time.load(Ordering::Relaxed) < elapsed
    }

    #[inline]
    pub fn timeout_id(&self, nodes: u64) -> bool {
        let elapsed = if self.use_soft_nodes() && self.use_max_nodes() {
            nodes
        } else {
            self.elapsed()
        };

        !self.is_infinite() && self.soft_time.load(Ordering::Relaxed) < elapsed
    }

    #[inline]
    pub fn elapsed(&self) -> u64 {
        self.start.load(Ordering::Relaxed)
            .elapsed()
            .as_millis() as u64
    }
    
    #[inline]
    pub fn use_max_depth(&self) -> bool {
        self.use_max_depth.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn use_max_nodes(&self) -> bool {
        self.use_max_nodes.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn use_soft_nodes(&self) -> bool {
        self.use_soft_nodes.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.infinite.load(Ordering::Relaxed)
    }
}