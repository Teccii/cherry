use std::{fmt::Write, sync::Arc};
use std::sync::atomic::{AtomicU8, Ordering};
use pyrrhic_rs::TableBases;
use cherry_core::*;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct SharedContext {
    pub t_table: Arc<TTable>,
    pub syzygy: Arc<Option<TableBases<SyzygyAdapter>>>,
    pub syzygy_depth: Arc<AtomicU8>,
    pub time_man: Arc<TimeManager>,
    pub weights: SearchWeights,
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct ThreadContext {
    pub qnodes: BatchedAtomicCounter,
    pub nodes: BatchedAtomicCounter,
    pub tt_hits: BatchedAtomicCounter,
    pub tt_misses: BatchedAtomicCounter,
    pub tb_hits: BatchedAtomicCounter,
    pub search_stack: Vec<SearchStack>,
    pub root_nodes: MoveTo<u64>,
    pub history: History,
    pub sel_depth: u16,
    pub abort_now: bool,
}

impl ThreadContext {
    #[inline(always)]
    pub fn reset(&mut self) {
        self.qnodes.reset();
        self.nodes.reset();
        self.tt_hits.reset();
        self.tt_misses.reset();
        self.tb_hits.reset();
        self.search_stack = vec![
            SearchStack {
                eval: -Score::INFINITE,
                move_played: None,
                extension: 0,
                reduction: 0,
                skip_move: None,
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

    #[inline(always)]
    pub fn update_sel_depth(&mut self, ply: u16) {
        if ply > self.sel_depth {
            self.sel_depth = ply;
        }
    }

    #[inline(always)]
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
    pub move_played: Option<MoveData>,
    pub extension: i16,
    pub reduction: i32,
    pub skip_move: Option<Move>,
    pub pv: [Option<Move>; MAX_PLY as usize + 1],
    pub pv_len: usize,
}

impl SearchStack {
    #[inline(always)]
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
    pub debug: bool,
}

impl Searcher {
    #[inline(always)]
    pub fn new(board: Board, time_man: Arc<TimeManager>) -> Searcher {
        Searcher {
            pos: Position::new(board),
            shared_ctx: SharedContext {
                t_table: Arc::new(TTable::new(256)),
                syzygy: Arc::new(None),
                syzygy_depth: Arc::new(AtomicU8::new(1)),
                weights: SearchWeights::default(),
                time_man,
            },
            main_ctx: ThreadContext {
                qnodes: BatchedAtomicCounter::new(),
                nodes: BatchedAtomicCounter::new(),
                tt_hits: BatchedAtomicCounter::new(),
                tt_misses: BatchedAtomicCounter::new(),
                tb_hits: BatchedAtomicCounter::new(),
                search_stack: vec![
                    SearchStack {
                        eval: -Score::INFINITE,
                        move_played: None,
                        extension: 0,
                        reduction: 0,
                        skip_move: None,
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
            debug: false,
        }
    }

    pub fn search<Info: SearchInfo>(&mut self, limits: &[SearchLimit]) -> (Move, Option<Move>, Score, u8, u64) {
        self.main_ctx.reset();
        self.shared_ctx.time_man.init(self.pos.stm(), limits);

        let mut result = (None, None, Score::ZERO, 0);

        rayon::scope(|s| {
            let chess960 = self.chess960;

            for i in 1..self.threads {
                let pos = self.pos.clone();
                let main_ctx = self.main_ctx.clone();
                let shared_ctx = self.shared_ctx.clone();

                s.spawn(move |_| {
                    let _ = search_worker::<Info>(
                        pos,
                        main_ctx,
                        shared_ctx,
                        i + 1,
                        chess960,
                    )();
                });
            }

            let pos = self.pos.clone();
            let main_ctx = self.main_ctx.clone();
            let shared_ctx = self.shared_ctx.clone();

            result = search_worker::<Info>(
                pos,
                main_ctx,
                shared_ctx,
                0,
                chess960,
            )();
        });

        let (best_move, ponder_move, best_score, depth) = result;

        if best_move.is_none() {
            panic!("Search failed!");
        }

        (best_move.unwrap(), ponder_move, best_score, depth, self.main_ctx.nodes.global())
    }

    #[inline(always)]
    pub fn resize_ttable(&mut self, mb: usize) {
        self.shared_ctx.t_table = Arc::new(TTable::new(mb));
    }

    #[inline(always)]
    pub fn clean_ttable(&mut self) {
        self.shared_ctx.t_table.clean();
    }

    #[inline(always)]
    pub fn set_threads(&mut self, count: u16) {
        self.threads = count.max(1);
    }

    #[inline(always)]
    pub fn set_chess960(&mut self, value: bool) {
        self.chess960 = value;
    }

    #[inline(always)]
    pub fn set_debug(&mut self, value: bool) {
        self.debug = value;
    }

    #[inline(always)]
    pub fn set_syzygy_path(&mut self, path: &str) {
        self.shared_ctx.syzygy = Arc::new(Some(TableBases::<SyzygyAdapter>::new(path).unwrap()));
    }
    
    #[inline(always)]
    pub fn set_syzygy_depth(&mut self, depth: u8) {
        self.shared_ctx.syzygy_depth.store(depth, Ordering::Relaxed);
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
        let mut depth = 1 + (thread % 2 == 1) as u8;

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
                let root_move = ctx.search_stack[0].pv[0].unwrap();

                shared_ctx.time_man.deepen(
                    thread,
                    depth,
                    ctx.root_nodes[root_move.from() as usize][root_move.to() as usize],
                    ctx.nodes.local(),
                    root_move,
                );

                if (score > alpha && score < beta) || score.is_decisive() {
                    best_move = Some(root_move);
                    eval = Some(score);
                    if score.is_decisive() {
                        ponder_move = None;

                        if thread == 0 {
                            Info::push(
                                pos.board(),
                                &ctx,
                                &shared_ctx,
                                depth,
                                eval,
                                chess960,
                            );
                        }

                        break 'id;
                    } else if !shared_ctx.time_man.is_pondering() {
                        ponder_move = ctx.search_stack[0].pv[1];
                    }

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
                    depth,
                    eval,
                    chess960,
                );
            }

            depth += 1;
            if shared_ctx.time_man.abort_id(depth, ctx.nodes.global()) {
                break 'id;
            }
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
        depth: u8,
        eval: Option<Score>,
        chess960: bool,
    );
}

pub struct FullInfo;
pub struct UciOnly;
pub struct NoInfo;

impl SearchInfo for FullInfo {
    fn push(
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
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
            "tthits {} ttmisses {} tbhits {}",
            ctx.tt_hits.global(),
            ctx.tt_misses.global(),
            ctx.tb_hits.global(),
        ).unwrap();

        let root_stack = &ctx.search_stack[0];
        let mut board = board.clone();
        let mut i = 0;

        write!(info, "pv ").unwrap();
        for (i, &mv) in root_stack.pv[..root_stack.pv_len].iter().enumerate() {
            if i as u8 == depth || i as u8 == MAX_DEPTH {
                break;
            }

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

        println!("{}", info);
    }
}

impl SearchInfo for UciOnly {
    fn push(
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
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
            }
        }

        let nodes = ctx.nodes.global();
        let millis = shared_ctx.time_man.start_time().elapsed().as_millis() as u64;
        
        write!(info, "nodes {} time {} ", nodes, millis).unwrap();

        if millis != 0 {
            write!(info, "nps {} ", (nodes / millis) * 1000).unwrap();
        }

        write!(info, "tbhits {} ", ctx.tb_hits.global(), ).unwrap();

        let root_stack = &ctx.search_stack[0];
        let mut board = board.clone();
        let mut i = 0;

        write!(info, "pv ").unwrap();
        for (i, &mv) in root_stack.pv[..root_stack.pv_len].iter().enumerate() {
            if i as u8 == depth || i as u8 == MAX_DEPTH {
                break;
            }

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

        println!("{}", info);
    }
}

impl SearchInfo for NoInfo {
    fn push(_: &Board, _: &ThreadContext, _: &SharedContext, _: u8, _: Option<Score>, _: bool) { }
}