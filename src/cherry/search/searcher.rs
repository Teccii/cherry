use std::{fmt::Write, sync::Arc};
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct SharedData {
    pub time_man: Arc<TimeManager>,
    pub nnue_weights: Arc<NetworkWeights>,
}

#[derive(Clone)]
pub struct ThreadData {
    pub nodes: BatchedAtomicCounter,
    pub search_stack: Vec<SearchStack>,
    pub root_pv: PrincipalVariation,
    pub sel_depth: u16,
    pub abort_now: bool,
}

impl ThreadData {
    #[inline]
    pub fn reset(&mut self) {
        self.nodes.reset();
        self.search_stack = vec![SearchStack::default(); MAX_PLY as usize + 1];
        self.sel_depth = 0;
        self.abort_now = false;
    }
}

impl Default for ThreadData {
    #[inline]
    fn default() -> Self {
        ThreadData {
            nodes: BatchedAtomicCounter::default(),
            search_stack: vec![SearchStack::default(); MAX_PLY as usize + 1],
            root_pv: PrincipalVariation::default(),
            sel_depth: 0,
            abort_now: false,
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct PrincipalVariation {
    pub moves: [Option<Move>; MAX_PLY as usize + 1],
    pub len: usize,
}

impl PrincipalVariation {
    #[inline]
    pub fn update(&mut self, best_move: Move, child_pv: &PrincipalVariation) {
        self.moves[0] = Some(best_move);
        self.len = child_pv.len + 1;
        self.moves[1..self.len].copy_from_slice(&child_pv.moves[..child_pv.len]);

    }

    pub fn display(&self, board: &Board, frc: bool) -> String {
        let mut board = board.clone();
        let mut output = String::new();

        if self.len != 0 {
            for &mv in self.moves[..self.len].iter() {
                if let Some(mv) = mv {
                    write!(output, "{} ", mv.display(&board, frc)).unwrap();
                    board.make_move(mv);
                } else {
                    break;
                }
            }
        }

        output
    }
}

impl Default for PrincipalVariation {
    #[inline]
    fn default() -> Self {
        PrincipalVariation {
            moves: [None; MAX_PLY as usize + 1],
            len: 0,
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Clone, Default)]
pub struct SearchStack {
    pub pv: PrincipalVariation,
}

/*----------------------------------------------------------------*/

pub struct Searcher {
    pub pos: Position,
    pub shared_data: SharedData,
    pub thread_data: ThreadData,
    pub threads: u16,
    pub ponder: bool,
    pub frc: bool,
}

impl Searcher {
    pub fn new(board: Board, time_man: Arc<TimeManager>) -> Searcher {
        let nnue_weights = NetworkWeights::default();
        
        Searcher {
            pos: Position::new(board, &nnue_weights),
            shared_data: SharedData { time_man, nnue_weights },
            thread_data: ThreadData::default(),
            threads: 1,
            ponder: false,
            frc: false,
        }
    }

    /*----------------------------------------------------------------*/

    pub fn search<Info: SearchInfo>(&mut self, limits: Vec<SearchLimit>) -> (Move, Option<Move>, Score, u16, u64) {
        self.thread_data.reset();
        self.shared_data.time_man.init(self.pos.stm(), &limits);
        self.reset_nnue();

        let mut result = (None, None, Score::ZERO, 0u16, 0u64);

        rayon::scope(|s| {
            let frc = self.frc;

            for i in 1..self.threads {
                let pos = self.pos.clone();
                let mut thread = self.thread_data.clone();
                let shared = self.shared_data.clone();

                s.spawn(move |_| {
                    search_worker(
                        pos,
                        &mut thread,
                        &shared,
                        Info::new(frc),
                        i
                    );
                });
            }

            let pos = self.pos.clone();
            let mut thread = self.thread_data.clone();
            let shared = self.shared_data.clone();

            result = search_worker(
                pos,
                &mut thread,
                &shared,
                Info::new(frc),
                0,
            );

            self.thread_data = thread;
        });

        let (best_move, ponder_move, score, depth, nodes) = result;

        (best_move.unwrap(), ponder_move.filter(|_| self.ponder), score, depth, nodes)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn set_board(&mut self, board: Board) {
        self.pos.set_board(board, &self.shared_data.nnue_weights);
    }

    #[inline]
    pub fn reset_nnue(&mut self) {
        self.pos.reset(&self.shared_data.nnue_weights);
    }

    #[inline]
    pub fn make_move(&mut self, mv: Move) {
        self.pos.make_move(mv, &self.shared_data.nnue_weights);
    }
}

pub fn search_worker<Info: SearchInfo>(
    mut pos: Position,
    thread: &mut ThreadData,
    shared: &SharedData,
    mut info: Info,
    worker: u16,
) -> (Option<Move>, Option<Move>, Score, u16, u64) {
    let mut best_move: Option<Move> = None;
    let mut ponder_move: Option<Move> = None;
    let mut score = -Score::INFINITE;
    let mut depth = 1;

    'id: loop {
        thread.sel_depth = 0;
        score = search::<PV>(
            &mut pos,
            thread,
            shared,
            i32::from(depth) * 1024,
            0,
            -Score::INFINITE,
            Score::INFINITE,
        );

        thread.root_pv = thread.search_stack[0].pv.clone();
        best_move = thread.root_pv.moves[0];
        ponder_move = thread.root_pv.moves[1];

        if worker == 0 {
            info.update(
                pos.board(),
                &thread,
                &shared,
                score,
                depth,
            );

            shared.time_man.deepen();
        }

        if (depth > 1 && thread.abort_now) || depth >= MAX_DEPTH || shared.time_man.abort_id(depth, thread.nodes.global()) {
            break 'id;
        }

        depth += 1;
    }

    while shared.time_man.is_infinite()
        && !(shared.time_man.use_max_depth() || shared.time_man.use_max_nodes())
        && !shared.time_man.abort_now() {
        if worker == 0 {
            info.update(
                pos.board(),
                &thread,
                &shared,
                score,
                depth,
            );
        }
    }

    if worker == 0 {
        info.update(
            pos.board(),
            &thread,
            &shared,
            score,
            depth,
        );
    }

    (best_move, ponder_move, score, depth, thread.nodes.global())
}