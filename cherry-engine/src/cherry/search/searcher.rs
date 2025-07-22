use std::{fmt::Write, sync::Arc};
use arrayvec::ArrayVec;
use pyrrhic_rs::TableBases;
use cherry_core::*;
use crate::*;

/*----------------------------------------------------------------*/

pub type LmrLookup = LookUp<i32, {MAX_PLY as usize}, MAX_MOVES>;
pub type SyzygyTable = Option<TableBases<SyzygyAdapter>>;

#[derive(Clone)]
pub struct SharedContext {
    pub t_table: Arc<TTable>,
    pub time_man: Arc<TimeManager>,
    pub syzygy: Arc<SyzygyTable>,
    pub syzygy_depth: u8,
    pub search_moves: ArrayVec<Move, MAX_MOVES>,
    pub weights: Arc<SearchWeights>,
    pub lmr_lookup: Arc<LmrLookup>,
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct ThreadContext {
    pub qnodes: BatchedAtomicCounter,
    pub nodes: BatchedAtomicCounter,
    pub tt_hits: BatchedAtomicCounter,
    pub tt_misses: BatchedAtomicCounter,
    pub tb_hits: BatchedAtomicCounter,
    pub root_nodes: MoveTo<u64>,
    pub ss: Vec<SearchStack>,
    pub history: History,
    pub sel_depth: u16,
    pub abort_now: bool,
}

impl ThreadContext {
    #[inline]
    pub fn reset(&mut self) {
        self.qnodes.reset();
        self.nodes.reset();
        self.tt_hits.reset();
        self.tt_misses.reset();
        self.tb_hits.reset();
        self.ss = vec![
            SearchStack {
                eval: -Score::INFINITE,
                stat_score: 0,
                extension: 0,
                reduction: 0,
                skip_move: None,
                move_played: None,
                pv: [None; MAX_PLY as usize + 1],
                pv_len: 0,
            };
            MAX_PLY as usize + 1
        ];
        self.history.reset();
        self.root_nodes = move_to(0);
        self.sel_depth = 0;
        self.abort_now = false;
    }

    #[inline]
    pub fn update_sel_depth(&mut self, ply: u16) {
        if ply > self.sel_depth {
            self.sel_depth = ply;
        }
    }

    #[inline]
    pub fn abort_now(&mut self) {
        self.abort_now = true;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MoveData {
    pub piece: Piece,
    pub victim: Option<Piece>,
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
}

impl MoveData {
    pub fn new(board: &Board, mv: Move) -> MoveData {
        let (from, to, promotion) = (mv.from(), mv.to(), mv.promotion());
        
        MoveData {
            piece: board.piece_on(from).unwrap(),
            victim: if board.is_en_passant(mv) {
                Some(Piece::Pawn)
            } else if board.is_capture(mv) {
                Some(board.piece_on(to).unwrap())
            } else {
                None
            },
            from,
            to,
            promotion
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct SearchStack {
    pub eval: Score,
    pub extension: i16,
    pub reduction: i32,
    pub stat_score: i32,
    pub skip_move: Option<Move>,
    pub move_played: Option<MoveData>,
    pub pv: [Option<Move>; MAX_PLY as usize + 1],
    pub pv_len: usize,
}

impl SearchStack {
    #[inline]
    pub fn update_pv(&mut self, best_move: Move, child_pv: &[Option<Move>]) {
        self.pv[0] = Some(best_move);
        self.pv_len = child_pv.len() + 1;
        self.pv[1..self.pv_len].copy_from_slice(child_pv);
    }
}

/*----------------------------------------------------------------*/

pub struct Searcher {
    pub pos: Position,
    pub shared_ctx: SharedContext,
    pub main_ctx: ThreadContext,
    pub threads: u16,
    pub chess960: bool,
    pub ponder: bool,
    pub debug: bool,
}

impl Searcher {
    #[inline]
    pub fn new(board: Board, time_man: Arc<TimeManager>) -> Searcher {
        Searcher {
            pos: Position::new(board),
            shared_ctx: SharedContext {
                t_table: Arc::new(TTable::new(16)),
                time_man,
                syzygy: Arc::new(None),
                syzygy_depth: 1,
                search_moves: ArrayVec::new(),
                weights: Arc::new(SearchWeights::default()),
                lmr_lookup: Arc::new(LookUp::new(|i, j|
                    1024 * (0.5 + (i as f32).ln() * (j as f32).ln() / 2.5) as i32
                )),
            },
            main_ctx: ThreadContext {
                qnodes: BatchedAtomicCounter::new(),
                nodes: BatchedAtomicCounter::new(),
                tt_hits: BatchedAtomicCounter::new(),
                tt_misses: BatchedAtomicCounter::new(),
                tb_hits: BatchedAtomicCounter::new(),
                ss: vec![
                    SearchStack {
                        eval: -Score::INFINITE,
                        stat_score: 0,
                        extension: 0,
                        reduction: 0,
                        skip_move: None,
                        move_played: None,
                        pv: [None; MAX_PLY as usize + 1],
                        pv_len: 0,
                    };
                    MAX_PLY as usize + 1
                ],
                root_nodes: move_to(0),
                history: History::new(),
                sel_depth: 0,
                abort_now: false,
            },
            threads: 1,
            chess960: false,
            ponder: false,
            debug: false,
        }
    }

    pub fn search<Info: SearchInfo>(&mut self, limits: Vec<SearchLimit>) -> (Move, Option<Move>, Score, u8, u64) {
        self.shared_ctx.time_man.init(self.pos.stm(), &limits);
        self.shared_ctx.search_moves.clear();

        for limit in &limits {
            match limit {
                SearchLimit::SearchMoves(moves) => for mv in moves {
                    self.shared_ctx.search_moves.push(Move::parse(self.pos.board(), self.chess960, mv).unwrap());
                },
                _ => { }
            }
        }
        
        self.main_ctx.reset();

        let mut result = (None, None, Score::ZERO, 0);
        rayon::scope(|s| {
            let chess960 = self.chess960;

            for i in 1..self.threads {
                let pos = self.pos.clone();
                let ctx = self.main_ctx.clone();
                let shared_ctx = self.shared_ctx.clone();

                s.spawn(move |_| {
                    let _ = search_worker::<Info>(
                        pos,
                        ctx,
                        shared_ctx,
                        i + 1,
                        chess960,
                    )();
                });
            }

            let pos = self.pos.clone();
            let ctx = self.main_ctx.clone();
            let shared_ctx = self.shared_ctx.clone();

            result = search_worker::<Info>(
                pos,
                ctx,
                shared_ctx,
                0,
                chess960,
            )();
        });

        let (best_move, ponder_move, best_score, depth) = result;

        if best_move.is_none() {
            panic!("Search failed!");
        }

        (best_move.unwrap(), ponder_move.filter(|_| self.ponder), best_score, depth, self.main_ctx.nodes.global())
    }

    #[inline]
    pub fn resize_ttable(&mut self, mb: usize) {
        self.shared_ctx.t_table = Arc::new(TTable::new(mb));
    }

    #[inline]
    pub fn clean_ttable(&mut self) {
        self.shared_ctx.t_table.clean();
    }

    #[inline]
    pub fn set_threads(&mut self, count: u16) {
        self.threads = count.max(1);
    }

    #[inline]
    pub fn set_chess960(&mut self, value: bool) {
        self.chess960 = value;
    }

    #[inline]
    pub fn set_ponder(&mut self, value: bool) {
        self.ponder = value;
    }

    #[inline]
    pub fn set_debug(&mut self, value: bool) {
        self.debug = value;
    }

    #[inline]
    pub fn set_syzygy_path(&mut self, path: &str) {
        self.shared_ctx.syzygy = Arc::new(Some(TableBases::<SyzygyAdapter>::new(path).unwrap()));
    }
    
    #[inline]
    pub fn set_syzygy_depth(&mut self, depth: u8) {
        self.shared_ctx.syzygy_depth = depth;
    }
}

fn search_worker<Info: SearchInfo>(
    mut pos: Position,
    mut ctx: ThreadContext,
    shared_ctx: SharedContext,
    thread: u16,
    chess960: bool,
) -> impl FnMut() -> (Option<Move>, Option<Move>, Score, u8) {
    move || {
        let mut window = Window::new(10);
        let mut best_move: Option<Move> = None;
        let mut ponder_move: Option<Move> = None;
        let mut eval: Option<Score> = None;
        let mut depth = 1;

        'id: loop {
            window.reset();
            let mut fails = 0;

            'asp: loop {
                let (alpha, beta) = if depth > 4
                    && eval.is_some_and(|e| e.abs() < 1000)
                    && fails < 10 {
                    window.get()
                } else {
                    (-Score::MAX_MATE, Score::MAX_MATE)
                };

                ctx.sel_depth = 0;
                let score = search::<PV>(
                    &mut pos,
                    &mut ctx,
                    &shared_ctx,
                    depth,
                    0,
                    alpha,
                    beta,
                    false,
                );

                if depth > 1 && ctx.abort_now {
                    break 'id;
                }

                window.set_midpoint(score);

                let root_move = ctx.ss[0].pv[0].unwrap();
                shared_ctx.time_man.deepen(
                    thread,
                    depth,
                    ctx.root_nodes[root_move.from() as usize][root_move.to() as usize],
                    ctx.nodes.local(),
                    root_move,
                );

                if (score > alpha && score < beta) || score.is_decisive() {
                    ponder_move = ctx.ss[0].pv[1];
                    best_move = Some(root_move);
                    eval = Some(score);

                    break 'asp;
                }

                if score <= alpha {
                    fails += 1;
                    window.fail_low();
                } else if score >= beta {
                    fails += 1;
                    window.fail_high();
                }
            }

            if thread == 0 {
                Info::push(
                    pos.board(),
                    &ctx,
                    &shared_ctx,
                    TTBound::Exact,
                    depth,
                    eval,
                    chess960,
                );
            }

            if shared_ctx.time_man.abort_id(depth, ctx.nodes.global()) {
                break 'id;
            }

            depth += 1;
        }

        while depth == MAX_DEPTH
            && shared_ctx.time_man.is_infinite()
            && !(shared_ctx.time_man.abort_now() || shared_ctx.time_man.timeout_id()) {
            if thread == 0 {
                Info::push(
                    pos.board(),
                    &ctx,
                    &shared_ctx,
                    TTBound::Exact,
                    depth,
                    eval,
                    chess960,
                );
            }
        }

        if thread == 0 {
            Info::push(
                pos.board(),
                &ctx,
                &shared_ctx,
                TTBound::Exact,
                depth,
                eval,
                chess960,
            );
        }

        if let Some(best_score) = eval {
            (best_move, ponder_move, best_score, depth)
        } else {
            panic!("Search Worker {} has failed!", thread);
        }
    }
}

/*----------------------------------------------------------------*/

pub trait SearchInfo {
    fn push(
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        bound: TTBound,
        depth: u8,
        eval: Option<Score>,
        chess960: bool,
    );
}

pub struct DebugInfo;
pub struct UciInfo;
pub struct NoInfo;

impl SearchInfo for DebugInfo {
    fn push(
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        bound: TTBound,
        depth: u8,
        eval: Option<Score>,
        chess960: bool,
    ) {
        let mut info = format!("info depth {} seldepth {} ", depth, ctx.sel_depth);

        if let Some(eval) = eval {
            if let Some(ply) = eval.mate_in() {
                write!(info, "score mate {} ", (ply + 1) / 2).unwrap();
            } else {
                write!(info, "score cp {} ", eval.0).unwrap();

                match bound {
                    TTBound::LowerBound => write!(info, "lowerbound ").unwrap(),
                    TTBound::UpperBound => write!(info, "upperbound ").unwrap(),
                    _ => { }
                }
            }
        }

        let nodes = ctx.nodes.global();
        let qnodes = ctx.qnodes.global();
        let millis = shared_ctx.time_man.start_time().elapsed().as_millis() as u64;
        write!(info, "nodes {} qnodes {} time {} ", nodes, qnodes, millis).unwrap();

        if millis != 0 {
            write!(info, "nps {} ", (nodes / millis) * 1000).unwrap();
        }

        write!(
            info,
            "tthits {} ttmisses {} tbhits {} ",
            ctx.tt_hits.global(),
            ctx.tt_misses.global(),
            ctx.tb_hits.global(),
        ).unwrap();

        let root_stack = &ctx.ss[0];
        let mut board = board.clone();

        if root_stack.pv_len != 0 {
            write!(info, "pv ").unwrap();
            let len = usize::min(root_stack.pv_len, depth as usize);
            
            for &mv in root_stack.pv[..len].iter() {
                if let Some(mv) = mv {
                    if !board.is_legal(mv) {
                        break;
                    }

                    write!(info, "{} ", mv.display(&board, chess960)).unwrap();
                    board.make_move(mv);
                } else {
                    break;
                }
            }
        }

        println!("{}", info);
    }
}

impl SearchInfo for UciInfo {
    fn push(
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        bound: TTBound,
        depth: u8,
        eval: Option<Score>,
        chess960: bool,
    ) {
        let mut info = format!("info depth {} seldepth {} ", depth, ctx.sel_depth);

        if let Some(eval) = eval {
            if let Some(ply) = eval.mate_in() {
                write!(info, "score mate {} ", (ply + 1) / 2).unwrap();
            } else {
                write!(info, "score cp {} ", eval.0).unwrap();

                match bound {
                    TTBound::LowerBound => write!(info, "lowerbound ").unwrap(),
                    TTBound::UpperBound => write!(info, "upperbound ").unwrap(),
                    _ => { }
                }
            }
        }

        let nodes = ctx.nodes.global();
        let millis = shared_ctx.time_man.start_time().elapsed().as_millis() as u64;
        
        write!(info, "nodes {} time {} ", nodes, millis).unwrap();

        if millis != 0 {
            write!(info, "nps {} ", (nodes / millis) * 1000).unwrap();
        }

        write!(info, "tbhits {} ", ctx.tb_hits.global(), ).unwrap();

        let root_stack = &ctx.ss[0];
        let mut board = board.clone();

        if root_stack.pv_len != 0 {
            write!(info, "pv ").unwrap();
            let len = usize::min(root_stack.pv_len, depth as usize);

            for &mv in root_stack.pv[..len].iter() {
                if let Some(mv) = mv {
                    if !board.is_legal(mv) {
                        break;
                    }

                    write!(info, "{} ", mv.display(&board, chess960)).unwrap();
                    board.make_move(mv);
                } else {
                    break;
                }
            }
        }

        println!("{}", info);
    }
}

impl SearchInfo for NoInfo {
    fn push(
        _: &Board,
        _: &ThreadContext,
        _: &SharedContext,
        _: TTBound,
        _: u8,
        _: Option<Score>,
        _: bool
    ) { }
}