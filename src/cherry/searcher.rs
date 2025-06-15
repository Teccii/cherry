use std::{fmt::Write, sync::Arc};
use cozy_chess::*;
use super::*;

#[derive(Clone)]
pub struct SharedContext {
    pub t_table: Arc<TTable>,
    pub time_man: Arc<TimeManager>,
}

#[derive(Debug, Clone)]
pub struct ThreadContext {
    pub qnodes: BatchedAtomicCounter,
    pub nodes: BatchedAtomicCounter,
    pub tt_hits: BatchedAtomicCounter,
    pub tt_misses: BatchedAtomicCounter,
    pub search_stack: Vec<SearchStack>,
    pub move_nodes: [[u64; Square::NUM]; Square::NUM],
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
        self.search_stack = vec![
            SearchStack {
                eval: -Score::INFINITE,
                skip_move: None,
                killers: Killers::new(),
                pv: [None; MAX_PLY as usize + 1],
                pv_len: 0,
            };
            MAX_PLY as usize + 1
        ];
        self.history.reset();
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

#[derive(Debug, Clone)]
pub struct SearchStack {
    pub eval: Score,
    pub skip_move: Option<Move>,
    pub killers: Killers,
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
}

impl Searcher {
    pub fn new(board: Board, time_man: Arc<TimeManager>) -> Searcher {
        Searcher {
            pos: Position::new(board),
            shared_ctx: SharedContext {
                t_table: Arc::new(TTable::new(256)),
                time_man,
            },
            main_ctx: ThreadContext {
                qnodes: BatchedAtomicCounter::new(),
                nodes: BatchedAtomicCounter::new(),
                tt_hits: BatchedAtomicCounter::new(),
                tt_misses: BatchedAtomicCounter::new(),
                search_stack: vec![
                    SearchStack {
                        eval: -Score::INFINITE,
                        skip_move: None,
                        killers: Killers::new(),
                        pv: [None; MAX_PLY as usize + 1],
                        pv_len: 0,
                    };
                    MAX_PLY as usize + 1
                ],
                move_nodes: [[0; Square::NUM]; Square::NUM],
                history: History::new(),
                sel_depth: 0,
                abort_now: false,
            },
            threads: 1,
            chess960: false,
        }
    }

    pub fn search(&mut self, limits: &[SearchLimit], uci_info: bool) -> (Move, Option<Move>, Score, u8, u64) {
        self.main_ctx.reset();
        self.shared_ctx
            .time_man
            .init(&self.pos, limits);

        let mut join_handlers = Vec::new();
        for i in 0..(self.threads - 1) {
            join_handlers.push(std::thread::spawn(search_worker(
                self.pos.clone(),
                self.main_ctx.clone(),
                self.shared_ctx.clone(),
                i + 1,
                self.chess960,
                uci_info
            )));
        }

        let (best_move, ponder_move, best_score, depth) = search_worker(
            self.pos.clone(),
            self.main_ctx.clone(),
            self.shared_ctx.clone(),
            0,
            self.chess960,
            uci_info
        )();

        for join_handler in join_handlers {
            join_handler.join().unwrap();
        }

        if best_move.is_none() {
            panic!("Search failed!");
        }

        (best_move.unwrap(), ponder_move, best_score, depth, self.main_ctx.nodes.global())
    }

    pub fn resize_ttable(&mut self, mb: usize) {
        self.shared_ctx.t_table = Arc::new(TTable::new(mb));
    }

    pub fn clean_ttable(&mut self) {
        self.shared_ctx.t_table.clean();
    }

    pub fn set_threads(&mut self, count: u16) {
        self.threads = count.max(1)
    }

    pub fn set_chess960(&mut self, value: bool) {
        self.chess960 = value;
    }
}

fn search_worker(
    mut pos: Position,
    mut ctx: ThreadContext,
    shared_ctx: SharedContext,
    thread: u16,
    chess960: bool,
    uci_info: bool,
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
                let score = search::<PV>(&mut pos, &mut ctx, &shared_ctx, depth, 0, alpha, beta);

                if ctx.abort_now {
                    break 'id;
                }

                window.set_midpoint(score);
                let root_move = ctx.search_stack[0].pv[0].unwrap();

                shared_ctx.time_man.deepen(
                    thread,
                    depth,
                    ctx.move_nodes[root_move.from as usize][root_move.to as usize],
                    ctx.nodes.local(),
                    root_move,
                );

                if (score > alpha && score < beta) || score.is_mate() {
                    best_move = Some(root_move);
                    eval = Some(score);
                    if score.is_mate() {
                        ponder_move = None;

                        if thread == 0 && uci_info {
                            info(
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

            if thread == 0 && uci_info {
                info(
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

fn info(
    board: &Board,
    ctx: &ThreadContext,
    shared_ctx: &SharedContext,
    depth: u8,
    eval: Option<Score>,
    chess960: bool,
) {
    let mut info = format!("info depth {} seldepth {} ", depth, ctx.sel_depth);

    if let Some(mut eval) = eval {
        eval = eval * match board.side_to_move() {
            Color::White => 1,
            Color::Black => -1,
        };

        if let Some(ply) = eval.mate_in() {
            write!(info, "score mate {} ", (ply + 1) / 2).unwrap();
        } else {
            write!(info, "score cp {} ", eval.0).unwrap();
        }
    }

    let nodes = ctx.nodes.global();
    let qnodes = ctx.qnodes.global();
    let millis = shared_ctx.time_man.start_time().elapsed().as_millis() as u64;
    write!(info, "nodes {} qnodes {} qratio {:.1}% time {} ", nodes, qnodes, (qnodes as f64 / nodes as f64) * 100.0, millis).unwrap();

    if millis != 0 {
        write!(info, "nps {} ", (nodes / millis) * 1000).unwrap();
    }

    let tt_hits = ctx.tt_hits.global();
    let tt_misses = ctx.tt_misses.global();
    let total_tt_probes = tt_hits + tt_misses;

    write!(
        info,
        "tthits {} ttmisses {} ttratio {:.1}% ",
        tt_hits,
        tt_misses,
        (tt_hits as f64 / total_tt_probes as f64) * 100.0
    )
        .unwrap();

    let root_stack = &ctx.search_stack[0];
    let mut board = board.clone();
    let mut i = 0;

    write!(info, "pv ").unwrap();
    for &mv in &root_stack.pv[..root_stack.pv_len] {
        if i == depth - 1 {
            break;
        }

        if let Some(mv) = mv {
            let display_mv = convert_move(&board, mv, chess960);

            if board.try_play(mv).is_err() || i == MAX_DEPTH {
                break;
            }

            write!(info, "{} ", display_mv).unwrap();
        } else {
            break;
        }

        i += 1;
    }

    println!("{}", info);
}