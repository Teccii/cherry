use std::{
    sync::atomic::*,
    sync::Mutex,
    time::*
};
use atomic_time::AtomicInstant;
use crate::*;

/*----------------------------------------------------------------*/

pub const MOVE_OVERHEAD: u64 = 100;
const EXPECTED_MOVES: u16 = 64;
const STABILITY_FACTOR: [f32; 5] = [2.5, 1.2, 0.9, 0.8, 0.75];

/*----------------------------------------------------------------*/

pub struct TimeManager {
    start: AtomicInstant,

    base_time: AtomicU64,
    target_time: AtomicU64,
    max_time: AtomicU64,
    move_overhead: AtomicU64,
    no_manage: AtomicBool,
    
    prev_move: Mutex<Option<Move>>,
    move_stability: AtomicU16,

    moves_to_go: AtomicU16,
    max_depth: AtomicU8,
    max_nodes: AtomicU64,
    
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
            target_time: AtomicU64::new(0),
            max_time: AtomicU64::new(0),
            move_overhead: AtomicU64::new(MOVE_OVERHEAD),
            no_manage: AtomicBool::new(true),
            prev_move: Mutex::new(None),
            move_stability: AtomicU16::new(0),
            moves_to_go: AtomicU16::new(EXPECTED_MOVES),
            max_depth: AtomicU8::new(MAX_DEPTH),
            max_nodes: AtomicU64::new(u64::MAX),
            infinite: AtomicBool::new(true),
            pondering: AtomicBool::new(false),
            abort_now: AtomicBool::new(false),
        }
    }

    /*----------------------------------------------------------------*/

    pub fn init(&self, stm: Color, limits: &[SearchLimit]) {
        *self.prev_move.lock().unwrap() = None;
        self.move_stability.store(0, Ordering::Relaxed);
        self.abort_now.store(false, Ordering::Relaxed);

        let mut w_time = 0;
        let mut b_time = 0;
        let mut w_inc = 0;
        let mut b_inc = 0;
        let mut move_time = None;
        let mut moves_to_go = None;
        let mut max_depth = MAX_DEPTH;
        let mut max_nodes = u64::MAX;
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
                    max_depth = *depth;
                },
                SearchLimit::MaxNodes(nodes ) => {
                    max_nodes = *nodes;
                },
                SearchLimit::Infinite => {
                    infinite = true;
                },
                SearchLimit::Ponder => {
                    pondering = true;
                },
            }
        }

        if pondering {
            infinite = true;
        }

        self.pondering.store(pondering, Ordering::Relaxed);
        self.infinite.store(infinite, Ordering::Relaxed);
        self.max_depth.store(max_depth, Ordering::Relaxed);
        self.max_nodes.store(max_nodes, Ordering::Relaxed);
        
        let moves_to_go = moves_to_go.unwrap_or(EXPECTED_MOVES);
        self.moves_to_go.store(moves_to_go, Ordering::Relaxed);
        self.no_manage.store(infinite || move_time.is_some(), Ordering::Relaxed);

        if let Some(time) = move_time {
            self.base_time.store(time, Ordering::Relaxed);
            self.target_time.store(time, Ordering::Relaxed);
            self.max_time.store(time, Ordering::Relaxed);
        } else {
            let (time, inc) = match stm {
                Color::White => (w_time, w_inc),
                Color::Black => (b_time, b_inc),
            };
            let move_overhead = self.move_overhead.load(Ordering::Relaxed);
            let max_time = (time * 3 / 5).min(time.saturating_sub(move_overhead));
            let target_time = (time / moves_to_go as u64 + inc).saturating_sub(move_overhead).min(max_time);

            self.base_time.store(target_time, Ordering::Relaxed);
            self.target_time.store(target_time, Ordering::Relaxed);
            self.max_time.store(max_time, Ordering::Relaxed);
        }

        self.start.store(Instant::now(), Ordering::Relaxed);
    }
    
    pub fn deepen(
        &self,
        thread: u16,
        depth: u8,
        move_nodes: u64,
        nodes: u64,
        mv: Move,
    ) {
        if thread != 0 || depth < 4 || self.no_manage.load(Ordering::Relaxed) {
            return;
        }

        let mut prev_move = self.prev_move.lock().unwrap();
        let mut move_stability = self.move_stability.load(Ordering::Relaxed);

        move_stability = if Some(mv) == *prev_move {
            (move_stability + 1).min(4)
        } else {
            0
        };

        *prev_move = Some(mv);
        self.move_stability.store(move_stability, Ordering::Relaxed);

        let move_stability_factor = STABILITY_FACTOR[move_stability as usize];
        let subtree_factor = (1.0 - move_nodes as f32 / nodes as f32) * 3.0 + 0.5;
        let base_time = self.base_time.load(Ordering::Relaxed);
        
        let new_target = (base_time as f32
            * move_stability_factor
            * subtree_factor
        ) as u64;
        
        self.target_time.store(new_target, Ordering::Relaxed);
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
    
    /*----------------------------------------------------------------*/

    #[inline]
    pub fn abort_now(&self) -> bool {
        self.abort_now.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn abort_search(&self, nodes: u64) -> bool {
        self.abort_now() || self.timeout_search()
        || self.max_nodes.load(Ordering::Relaxed) <= nodes
    }

    #[inline]
    pub fn abort_id(&self, depth: u8, nodes: u64) -> bool {
        self.abort_now() || self.timeout_id()
        || self.max_depth.load(Ordering::Relaxed) <= depth
        || self.max_nodes.load(Ordering::Relaxed) <= nodes
    }

    #[inline]
    pub fn timeout_search(&self) -> bool {
        !self.is_infinite() && self.max_time.load(Ordering::Relaxed) < self.elapsed()
    }

    #[inline]
    pub fn timeout_id(&self) -> bool {
        !self.is_infinite() && self.target_time.load(Ordering::Relaxed) < self.elapsed()
    }

    #[inline]
    pub fn start_time(&self) -> Instant {
        self.start.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn elapsed(&self) -> u64 {
        self.start_time().elapsed().as_millis() as u64
    }

    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.infinite.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_pondering(&self) -> bool {
        self.pondering.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn no_manage(&self) -> bool {
        self.no_manage.load(Ordering::Relaxed)
    }
}