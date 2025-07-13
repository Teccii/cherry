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
const STABILITY_FACTOR: [f32; 5] = [2.0, 1.3, 0.7, 0.5, 0.3];

/*----------------------------------------------------------------*/

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchLimit {
    SearchMoves(Vec<String>),
    WhiteTime(Duration),
    BlackTime(Duration),
    WhiteInc(Duration),
    BlackInc(Duration),
    MoveTime(Duration),
    MovesToGo(u16),
    MaxDepth(u8),
    MaxNodes(u64),
    Infinite,
    Ponder,
}

/*----------------------------------------------------------------*/

pub struct TimeManager {
    start: AtomicInstant,
    time: AtomicU64,
    otime: AtomicU64,

    base_time: AtomicU64,
    target_time: AtomicU64,
    max_time: AtomicU64,
    flag_factor: AtomicU64,
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
    #[inline(always)]
    pub fn new() -> TimeManager {
        TimeManager {
            start: AtomicInstant::new(Instant::now()),
            time: AtomicU64::new(0),
            otime: AtomicU64::new(0),
            base_time: AtomicU64::new(0),
            target_time: AtomicU64::new(0),
            max_time: AtomicU64::new(0),
            flag_factor: AtomicU64::new(0),
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
                }
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
            let (time, otime, inc) = match stm {
                Color::White => (w_time, b_time, w_inc),
                Color::Black => (b_time, w_time, b_inc),
            };
            let move_overhead = self.move_overhead.load(Ordering::Relaxed);
            let flag_factor = self.flag_factor(time, otime);

            let max_time = ((time - move_overhead) * 3 / 5).max(move_overhead);
            let target_time = (
                (time - move_overhead) / moves_to_go as u64 / flag_factor + inc
            ).saturating_sub(move_overhead).min(max_time);

            self.time.store(time, Ordering::Relaxed);
            self.otime.store(otime, Ordering::Relaxed);
            self.flag_factor.store(flag_factor, Ordering::Relaxed);
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
            *self.prev_move.lock().unwrap() = Some(mv);
            return;
        }
        
        let mut prev_move = self.prev_move.lock().unwrap();
        let mut move_stability = self.move_stability.load(Ordering::Relaxed);
        
        move_stability = if Some(mv) == *prev_move {
            (move_stability + 1).min(4)
        } else {
            0
        };

        let flag_factor = self.flag_factor.load(Ordering::Relaxed);
        let new_flag_factor = self.flag_factor(
            self.time.load(Ordering::Relaxed) - self.elapsed(),
            self.otime.load(Ordering::Relaxed)
        );
        self.flag_factor.store(new_flag_factor, Ordering::Relaxed);

        let move_stability_factor = STABILITY_FACTOR[move_stability as usize];
        let subtree_factor = (1.0 - move_nodes as f32 / nodes as f32) * 1.5 + 0.5;
        let flag_factor = if flag_factor != new_flag_factor {
            0.5 * flag_factor as f32 / new_flag_factor as f32
        } else {
            1.0
        };

        let base_time = self.base_time.load(Ordering::Relaxed);
        
        *prev_move = Some(mv);
        self.move_stability.store(move_stability, Ordering::Relaxed);
        
        let new_target = (base_time as f32
            * move_stability_factor
            * subtree_factor
            * flag_factor
        ) as u64;
        
        self.target_time.store(new_target, Ordering::Relaxed);
    }
    
    #[inline(always)]
    pub fn ponderhit(&self) {
        self.pondering.store(false, Ordering::Relaxed);
        self.infinite.store(false, Ordering::Relaxed);
        self.no_manage.store(false, Ordering::Relaxed);
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn set_overhead(&self, millis: u64) {
        self.move_overhead.store(millis, Ordering::Relaxed);
    }
    
    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn abort_now(&self) {
        self.abort_now.store(true, Ordering::Relaxed);
    }
    
    #[inline(always)]
    pub fn abort_search(&self, nodes: u64) -> bool {
        self.abort_now.load(Ordering::Relaxed)
        || self.timeout_search()
        || self.max_nodes.load(Ordering::Relaxed) <= nodes
    }

    #[inline(always)]
    pub fn abort_id(&self, depth: u8, nodes: u64) -> bool {
        self.abort_now.load(Ordering::Relaxed)
        || self.timeout_id()
        || self.max_depth.load(Ordering::Relaxed) < depth
        || self.max_nodes.load(Ordering::Relaxed) <= nodes
    }

    #[inline(always)]
    pub fn start_time(&self) -> Instant {
        self.start.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn elapsed(&self) -> u64 {
        self.start_time().elapsed().as_millis() as u64
    }

    #[inline(always)]
    pub fn is_infinite(&self) -> bool {
        self.infinite.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn is_pondering(&self) -> bool {
        self.pondering.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn no_manage(&self) -> bool {
        self.no_manage.load(Ordering::Relaxed)
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    fn flag_factor(&self, time: u64, otime: u64) -> u64 {
        1 + (3 * time < 2 * otime) as u64
            + (2 * time < otime) as u64
            + (3 * time < otime) as u64
    }

    #[inline(always)]
    fn timeout_search(&self) -> bool {
        !self.is_infinite() && self.max_time.load(Ordering::Relaxed) < self.elapsed()
    }

    #[inline(always)]
    fn timeout_id(&self) -> bool {
        !self.is_infinite() && self.target_time.load(Ordering::Relaxed) < self.elapsed()
    }
}