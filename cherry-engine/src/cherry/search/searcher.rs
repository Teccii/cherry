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
    pub root_pv: Pv,
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
        self.root_pv = Pv::default();
        self.root_nodes = move_to(0);
        self.ss = vec![
            SearchStack {
                eval: -Score::INFINITE,
                tt_pv: false,
                stat_score: 0,
                extension: 0,
                reduction: 0,
                skip_move: None,
                move_played: None,
                pv: Pv::default(),
            };
            MAX_PLY as usize + 1
        ];
        self.history.reset();
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
    pub promotion: Option<Piece>,
    pub from: Square,
    pub to: Square,

}

impl MoveData {
    pub fn new(board: &Board, mv: Move) -> MoveData {
        let (from, to, promotion) = (mv.from(), mv.to(), mv.promotion());

        MoveData {
            piece: board.piece_on(from).unwrap(),
            victim: board.victim(mv),
            promotion,
            from,
            to,
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Pv {
    pub moves: [Option<Move>; MAX_PLY as usize + 1],
    pub len: usize,
}

impl Pv {
    #[inline]
    pub fn update(&mut self, best_move: Move, child_pv: &[Option<Move>]) {
        self.moves[0] = Some(best_move);
        self.len = child_pv.len() + 1;
        self.moves[1..self.len].copy_from_slice(child_pv);
    }
}

impl Default for Pv {
    #[inline]
    fn default() -> Self {
        Pv {
            moves: [None; MAX_PLY as usize + 1],
            len: 0,
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct SearchStack {
    pub eval: Score,
    pub tt_pv: bool,
    pub extension: i16,
    pub reduction: i32,
    pub stat_score: i32,
    pub skip_move: Option<Move>,
    pub move_played: Option<MoveData>,
    pub pv: Pv,
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
                root_pv: Pv {
                    moves: [None; MAX_PLY as usize + 1],
                    len: 0,
                },
                root_nodes: move_to(0),
                ss: vec![
                    SearchStack {
                        eval: -Score::INFINITE,
                        tt_pv: false,
                        stat_score: 0,
                        extension: 0,
                        reduction: 0,
                        skip_move: None,
                        move_played: None,
                        pv: Pv::default(),
                    };
                    MAX_PLY as usize + 1
                ],
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
    pub fn clean_ttable(&mut self) {
        self.shared_ctx.t_table.clean();
    }

    #[inline]
    pub fn resize_ttable(&mut self, mb: usize) {
        self.shared_ctx.t_table = Arc::new(TTable::new(mb));
    }

    #[inline]
    pub fn set_syzygy_path(&mut self, path: &str) {
        self.shared_ctx.syzygy = Arc::new(Some(TableBases::<SyzygyAdapter>::new(path).unwrap()));
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
                    (-Score::INFINITE, Score::INFINITE)
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
                let root_move = ctx.ss[0].pv.moves[0].unwrap();

                shared_ctx.time_man.deepen(
                    thread,
                    depth,
                    ctx.root_nodes[root_move.from() as usize][root_move.to() as usize],
                    ctx.nodes.local(),
                    root_move,
                );
                if (score > alpha && score < beta) || score.is_decisive() {
                    ctx.root_pv = ctx.ss[0].pv.clone();
                    ponder_move = ctx.ss[0].pv.moves[1];
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

            Info::push(
                thread,
                pos.board(),
                &ctx,
                &shared_ctx,
                eval,
                depth,
                chess960
            );

            if shared_ctx.time_man.abort_id(depth, ctx.nodes.global()) {
                break 'id;
            }

            depth += 1;
        }

        while depth == MAX_DEPTH
            && shared_ctx.time_man.is_infinite()
            && !(shared_ctx.time_man.abort_now() || shared_ctx.time_man.timeout_id()) {
            Info::push(
                thread,
                pos.board(),
                &ctx,
                &shared_ctx,
                eval,
                depth,
                chess960
            );
        }

        Info::push(
            thread,
            pos.board(),
            &ctx,
            &shared_ctx,
            eval,
            depth,
            chess960
        );

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
        thread: u16,
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        eval: Option<Score>,
        depth: u8,
        chess960: bool,
    );
}

pub struct DebugInfo;
pub struct UciInfo;
pub struct NoInfo;

impl SearchInfo for DebugInfo {
    fn push(
        thread: u16,
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        eval: Option<Score>,
        depth: u8,
        chess960: bool,
    ) {
        if thread != 0 {
            return;
        }
        
        let mut info = format!("info depth {} seldepth {} ", depth, ctx.sel_depth);

        if let Some(eval) = eval {
            if let Some(ply) = eval.mate_in() {
                write!(info, "score mate {} ", (ply + 1) / 2).unwrap();
            } else {
                write!(info, "score cp {} ", eval.0).unwrap();
            }
        }
        
        let nodes = ctx.nodes.global();
        let time = shared_ctx.time_man.elapsed();
        
        write!(info, "nodes {} qnodes {} time {} ", nodes, ctx.qnodes.global(), time).unwrap();

        if time != 0 {
            write!(info, "nps {} ", (nodes / time) * 1000).unwrap();
        }

        write!(
            info,
            "tthits {} ttmisses {} tbhits {} ",
            ctx.tt_hits.global(),
            ctx.tt_misses.global(),
            ctx.tb_hits.global(),
        ).unwrap();
        
        let mut board = board.clone();
        let root_pv = &ctx.root_pv;

        if root_pv.len != 0 {
            write!(info, "pv ").unwrap();
            let len = usize::min(root_pv.len, depth as usize);
            
            for &mv in root_pv.moves[..len].iter() {
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
        thread: u16,
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        eval: Option<Score>,
        depth: u8,
        chess960: bool,
    ) {
        if thread != 0 {
            return;
        }

        let mut info = format!("info depth {} seldepth {} ", depth, ctx.sel_depth);

        if let Some(eval) = eval {
            if let Some(ply) = eval.mate_in() {
                write!(info, "score mate {} ", (ply + 1) / 2).unwrap();
            } else {
                write!(info, "score cp {} ", eval.0).unwrap();
            }
        }

        let nodes = ctx.nodes.global();
        let time = shared_ctx.time_man.elapsed();

        write!(info, "nodes {} time {} ", nodes, time).unwrap();
        
        if time != 0 {
            write!(info, "nps {} ", (nodes / time) * 1000).unwrap();
        }

        write!(info, "tbhits {} ", ctx.tb_hits.global()).unwrap();

        let mut board = board.clone();
        let root_pv = &ctx.root_pv;

        if root_pv.len != 0 {
            write!(info, "pv ").unwrap();
            let len = usize::min(root_pv.len, depth as usize);

            for &mv in root_pv.moves[..len].iter() {
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
        _: u16,
        _: &Board,
        _: &ThreadContext,
        _: &SharedContext,
        _: Option<Score>,
        _: u8,
        _: bool,
    ) { }
}