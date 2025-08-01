use std::sync::Arc;
use arrayvec::ArrayVec;
use pyrrhic_rs::TableBases;
use crate::*;

/*----------------------------------------------------------------*/

pub type LmrLookup = LookUp<i32, {MAX_PLY as usize}, MAX_MOVES>;
pub type SyzygyTable = Option<TableBases<SyzygyAdapter>>;

#[derive(Clone)]
pub struct SharedContext {
    pub t_table: Arc<TTable>,
    pub time_man: Arc<TimeManager>,
    #[cfg(feature = "nnue")] pub nnue_weights: Arc<NetworkWeights>,
    pub syzygy: Arc<SyzygyTable>,
    pub syzygy_depth: u8,
    pub root_moves: ArrayVec<Move, MAX_MOVES>,
    pub weights: Arc<SearchWeights>,
    pub lmr_quiet: Arc<LmrLookup>,
    pub lmr_tactical: Arc<LmrLookup>,
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
}

impl Searcher {
    #[inline]
    pub fn new(
        board: Board,
        time_man: Arc<TimeManager>,
        #[cfg(feature = "nnue")] weights: NetworkWeights
    ) -> Searcher {
        Searcher {
            #[cfg(not(feature = "nnue"))] pos: Position::new(board),
            #[cfg(feature = "nnue")] pos: Position::new(board, &weights),
            shared_ctx: SharedContext {
                t_table: Arc::new(TTable::new(16)),
                time_man,
                #[cfg(feature = "nnue")] nnue_weights: Arc::new(weights),
                syzygy: Arc::new(None),
                syzygy_depth: 1,
                root_moves: ArrayVec::new(),
                weights: Arc::new(SearchWeights::default()),
                lmr_quiet: Arc::new(LookUp::new(|i, j|
                    1024 * (0.5 + (i as f32).ln() * (j as f32).ln() / 2.5) as i32
                )),
                lmr_tactical: Arc::new(LookUp::new(|i, j|
                    1024 * (0.4 + (i as f32).ln() * (j as f32).ln() / 3.5) as i32
                ))
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
        }
    }

    pub fn search<Info: SearchInfo>(&mut self, limits: Vec<SearchLimit>) -> (Move, Option<Move>, Score, u8, u64) {
        self.main_ctx.reset();
        self.shared_ctx.time_man.init(self.pos.stm(), &limits);
        self.shared_ctx.root_moves.clear();

        for limit in &limits {
            match limit {
                SearchLimit::SearchMoves(moves) => for mv in moves {
                    self.shared_ctx.root_moves.push(Move::parse(self.pos.board(), self.chess960, mv).unwrap());
                },
                _ => { }
            }
        }

        #[cfg(feature = "nnue")] self.pos.reset(&self.shared_ctx.nnue_weights);

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
        let mut eval = -Score::INFINITE;
        let mut depth = 1;

        'id: loop {
            window.reset();
            let mut fails = 0;

            'asp: loop {
                let (alpha, beta) = if depth > 4
                    && eval.abs() < 1000
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
                    eval = score;

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

            Info::update(
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
            && !(shared_ctx.time_man.use_max_depth() || shared_ctx.time_man.use_max_nodes())
            && !ctx.abort_now {
            Info::update(
                thread,
                pos.board(),
                &ctx,
                &shared_ctx,
                eval,
                depth,
                chess960
            );
        }

        Info::update(
            thread,
            pos.board(),
            &ctx,
            &shared_ctx,
            eval,
            depth,
            chess960
        );

        (best_move, ponder_move, eval, depth)
    }
}