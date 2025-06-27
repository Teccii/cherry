use std::{
    sync::atomic::*,
    sync::Mutex,
    time::*
};
use atomic_time::AtomicInstant;
use cozy_chess::*;
use crate::*;

/*----------------------------------------------------------------*/

const MOVE_OVERHEAD: u64 = 30;
const EXPECTED_MOVES: u16 = 50;
const STABILITY_FACTOR: [f32; 5] = [2.5, 1.2, 0.9, 0.8, 0.75];

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SearchLimit {
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

//it breaks my heart that these have to be atomic and mutex
//it just feels like it shouldn't need to be, since only the main thread can write
//and the helper threads just read it to check if they should abort
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
    #[inline(always)]
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

    pub fn init(&self, pos: &mut Position, limits: &[SearchLimit]) {
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

        for &limit in limits {
            match limit {
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
                    moves_to_go = Some(moves);
                },
                SearchLimit::MaxDepth(depth ) => {
                    max_depth = depth;
                },
                SearchLimit::MaxNodes(nodes ) => {
                    max_nodes = nodes;
                },
                SearchLimit::Infinite => {
                    infinite = true;
                },
                SearchLimit::Ponder => {
                    infinite = true;
                    pondering = true;
                }
            }
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
            let (time, otime, inc) = match pos.board().side_to_move() {
                Color::White => (w_time, b_time, w_inc),
                Color::Black => (b_time, w_time, b_inc),
            };
            
            let flag_factor = 1 + (otime < 15000 && time > 2 * otime && pos.eval(0) >= 0) as u64;
            let move_overhead = self.move_overhead.load(Ordering::Relaxed);
            
            let target_time = (time + inc) / moves_to_go as u64 / flag_factor - move_overhead;
            let max_time = (target_time * 7 / 2).min(time - move_overhead);
            
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
            move_stability + 1
        } else {
            0
        };

        let move_stability_factor = STABILITY_FACTOR[move_stability.min(4) as usize];
        let subtree_factor = (1.0 - move_nodes as f32 / nodes as f32) * 3.2 + 0.5;
        let base_time = self.base_time.load(Ordering::Relaxed);
        
        *prev_move = Some(mv);
        self.move_stability.store(move_stability, Ordering::Relaxed);
        
        let new_target = (base_time as f32
            * move_stability_factor
            * subtree_factor
        ) as u64;
        
        self.target_time.store(new_target, Ordering::Relaxed);
    }
    
    #[inline(always)]
    pub fn ponderhit(&self) {
        self.pondering.store(false, Ordering::Relaxed);
        self.infinite.store(false, Ordering::Relaxed);
        self.start.store(Instant::now(), Ordering::Relaxed);
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
        || self.timeout_search(self.start_time())
        || self.max_nodes.load(Ordering::Relaxed) <= nodes
    }

    #[inline(always)]
    pub fn abort_id(&self, depth: u8, nodes: u64) -> bool {
        self.abort_now.load(Ordering::Relaxed)
        || self.timeout_id(self.start_time())
        || self.max_depth.load(Ordering::Relaxed) < depth
        || self.max_nodes.load(Ordering::Relaxed) <= nodes
    }

    #[inline(always)]
    pub fn start_time(&self) -> Instant {
        self.start.load(Ordering::Relaxed)
    }
    
    #[inline(always)]
    pub fn is_pondering(&self) -> bool {
        self.pondering.load(Ordering::Relaxed)
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    fn timeout_search(&self, start: Instant) -> bool {
        !self.infinite.load(Ordering::Relaxed)
            && self.max_time.load(Ordering::Relaxed) < start.elapsed().as_millis() as u64
    }

    #[inline(always)]
    fn timeout_id(&self, start: Instant) -> bool {
        !self.infinite.load(Ordering::Relaxed)
            && self.target_time.load(Ordering::Relaxed) < start.elapsed().as_millis() as u64
    }
}