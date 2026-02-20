use std::{
    fmt::Write,
    sync::{Arc, atomic::*},
    thread::JoinHandle,
};

use pyrrhic_rs::DtzProbeValue;

use crate::*;

/*----------------------------------------------------------------*/

pub const MAX_THREADS: u32 = 2048;

pub struct SharedData {
    pub ttable: TTable,
    pub time_man: TimeManager,
    pub num_searching: AtomicU32,
    pub best_score: AtomicI32,
    pub best_move: AtomicU16,
    pub nodes: Arc<AtomicU64>,
}

impl SharedData {
    #[inline]
    pub fn best_score(&self) -> Score {
        Score(self.best_score.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn best_move(&self) -> Option<Move> {
        let bits = self.best_move.load(Ordering::Relaxed);

        if bits != 0 {
            Some(Move::from_bits(bits))
        } else {
            None
        }
    }
}

impl Default for SharedData {
    #[inline]
    fn default() -> Self {
        SharedData {
            ttable: TTable::new(16),
            time_man: TimeManager::new(),
            num_searching: AtomicU32::new(0),
            best_score: AtomicI32::new(Score::NONE.0),
            best_move: AtomicU16::new(0),
            nodes: Arc::new(AtomicU64::new(0)),
        }
    }
}

#[derive(Clone)]
pub struct ThreadData {
    pub abort_now: bool,
    pub nodes: BatchedAtomicCounter,
    pub search_stack: Vec<SearchStack>,
    pub root_nodes: [[u64; Square::COUNT]; Square::COUNT],
    pub root_pv: PrincipalVariation,
    pub exclude_moves: MoveList,
    pub root_moves: MoveList,
    pub windows: Vec<Window>,
    pub history: Box<History>,
    pub nmp_min_ply: u16,
    pub sel_depth: u16,
    pub multipv: u8,
    pub eval_scaling: bool,
    pub ponder: bool,
    pub frc: bool,
    pub id: usize,
}

impl ThreadData {
    #[inline]
    pub fn new(nodes: Arc<AtomicU64>, id: usize) -> ThreadData {
        ThreadData {
            abort_now: false,
            nodes: BatchedAtomicCounter::new(nodes),
            search_stack: vec![SearchStack::default(); MAX_PLY as usize + 1],
            windows: Vec::new(),
            root_moves: MoveList::empty(),
            exclude_moves: MoveList::empty(),
            root_nodes: [[0; Square::COUNT]; Square::COUNT],
            root_pv: PrincipalVariation::default(),
            history: unsafe { Box::new_zeroed().assume_init() },
            nmp_min_ply: 0,
            sel_depth: 0,
            eval_scaling: true,
            multipv: 1,
            ponder: false,
            frc: false,
            id,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.abort_now = false;
        self.nodes.reset();
        self.search_stack = vec![SearchStack::default(); MAX_PLY as usize + 1];
        self.root_nodes = [[0; Square::COUNT]; Square::COUNT];
        self.root_pv = PrincipalVariation::default();
        self.exclude_moves.clear();
        self.root_moves.clear();
        self.nmp_min_ply = 0;
        self.windows.clear();
        self.sel_depth = 0;
        self.eval_scaling = true;
        self.multipv = 1;
        self.ponder = false;
        self.frc = false;
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
    pub static_eval: Score,
    pub skip_move: Option<Move>,
    pub pv: PrincipalVariation,
}

#[derive(Copy, Clone)]
pub struct MoveData {
    pub piece: Piece,
    pub mv: Move,
}

impl MoveData {
    #[inline]
    pub fn new(board: &Board, mv: Move) -> MoveData {
        MoveData {
            piece: board.piece_on(mv.src()).unwrap(),
            mv,
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub enum ThreadCommand {
    Go {
        pos: Position,
        options: EngineOptions,
        root_moves: MoveList,
        info: SearchInfo,
    },
    SetShared(Arc<SharedData>),
    NewGame,
    Quit,
}

pub struct Searcher {
    pub shared: Arc<SharedData>,
    command_sender: Sender<ThreadCommand>,
    search_threads: Vec<JoinHandle<()>>,
}

impl Searcher {
    pub fn search(
        &mut self,
        pos: Position,
        limits: Vec<SearchLimit>,
        options: EngineOptions,
        info: SearchInfo,
    ) {
        self.shared.num_searching.store(1, Ordering::Relaxed);
        self.shared.time_man.init(
            pos.stm(),
            &limits,
            options.move_overhead,
            options.soft_target,
        );

        let focused = limits
            .iter()
            .any(|l| matches!(l, SearchLimit::SearchMoves(_)));
        let mut root_moves = limits
            .iter()
            .find_map(|l| match l {
                SearchLimit::SearchMoves(moves) => Some(moves.clone()),
                _ => None,
            })
            .unwrap_or_else(|| pos.board().gen_moves());

        if !focused
            && is_syzygy_enabled()
            && let Some(dtz) = probe_dtz(pos.board())
        {
            let mut best_moves = MoveList::empty();
            let mut best_score = 0u8;

            for value in &dtz.moves[..dtz.num_moves] {
                if let DtzProbeValue::DtzResult(result) = value {
                    let move_score = result.wdl as u8;
                    if move_score < best_score {
                        continue;
                    }
                    if move_score > best_score {
                        best_score = move_score;
                        best_moves.clear();
                    }

                    let src = Square::index(result.from_square as usize);
                    let dest = Square::index(result.to_square as usize);
                    let promotion = match Piece::index(result.promotion as usize) {
                        Piece::Pawn => None,
                        piece => Some(piece),
                    };
                    let move_str = format!(
                        "{}{}{}",
                        src,
                        dest,
                        match promotion {
                            Some(piece) => piece.to_string(),
                            None => String::from(""),
                        }
                    );

                    if let Some(mv) = Move::parse(pos.board(), &move_str)
                        && root_moves.contains(&mv)
                    {
                        best_moves.push(mv);
                    }
                }
            }

            root_moves = best_moves;
        }

        self.command_sender.send(ThreadCommand::Go {
            pos,
            options,
            root_moves,
            info,
        });
    }

    #[inline]
    pub fn set_threads(&mut self, threads: u32) {
        assert!(
            !self.is_searching(),
            "Called `set_threads() while searching"
        );
        assert!(threads > 0, "Threads must be greater than 0");

        self.command_sender.send(ThreadCommand::Quit);
        self.search_threads
            .drain(..)
            .for_each(|t| t.join().unwrap());

        let (tx, rx) = channel(threads);
        self.search_threads = rx
            .enumerate()
            .map(|(i, rx)| {
                std::thread::spawn({
                    let shared = self.shared.clone();
                    move || {
                        if std::panic::catch_unwind(move || thread_loop(rx, shared, i)).is_err() {
                            std::process::exit(1);
                        }
                    }
                })
            })
            .collect();
        self.command_sender = tx;
    }

    #[inline]
    pub fn resize_ttable(&mut self, mb: u64) {
        assert!(
            !self.is_searching(),
            "Called `resize_ttable()` while searching"
        );

        self.shared = Arc::new(SharedData {
            ttable: TTable::new(mb),
            time_man: TimeManager::new(),
            num_searching: AtomicU32::new(0),
            best_score: AtomicI32::new(Score::NONE.0),
            best_move: AtomicU16::new(0),
            nodes: Arc::new(AtomicU64::new(0)),
        });
        self.command_sender
            .send(ThreadCommand::SetShared(self.shared.clone()));
    }

    #[inline]
    pub fn ponderhit(&mut self) {
        assert!(
            self.is_searching(),
            "Called `ponderhit()` while not searching"
        );
        self.shared.time_man.ponderhit();
    }

    #[inline]
    pub fn newgame(&mut self) {
        assert!(!self.is_searching(), "Called `newgame()` while searching");

        self.shared.ttable.clear(self.search_threads.len());
        self.command_sender.send(ThreadCommand::NewGame);
    }

    #[inline]
    pub fn quit(&mut self) {
        self.shared.time_man.set_abort(true);
        self.command_sender.send(ThreadCommand::Quit);
        self.search_threads
            .drain(..)
            .for_each(|t| t.join().unwrap());
    }

    #[inline]
    pub fn stop(&self) {
        assert!(self.is_searching(), "Called `stop()` while not searching");
        self.shared.time_man.set_abort(true);
    }

    #[inline]
    pub fn wait(&self) {
        let mut num_searching = self.shared.num_searching.load(Ordering::Relaxed);
        while num_searching != 0 {
            atomic_wait::wait(&self.shared.num_searching, num_searching);
            num_searching = self.shared.num_searching.load(Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn is_searching(&self) -> bool {
        self.shared.num_searching.load(Ordering::Relaxed) != 0
    }
}

impl Default for Searcher {
    #[inline]
    fn default() -> Self {
        let shared = Arc::new(SharedData::default());
        let (tx, mut rx) = channel(1);
        let search_thread = std::thread::spawn({
            let shared = shared.clone();

            move || {
                if std::panic::catch_unwind(move || thread_loop(rx.next().unwrap(), shared, 0))
                    .is_err()
                {
                    std::process::exit(1);
                }
            }
        });

        Searcher {
            shared,
            search_threads: vec![search_thread],
            command_sender: tx,
        }
    }
}

/*----------------------------------------------------------------*/

fn thread_loop(mut rx: Receiver<ThreadCommand>, mut shared: Arc<SharedData>, id: usize) {
    let mut thread = ThreadData::new(shared.nodes.clone(), id);
    loop {
        match rx.recv(|cmd| cmd.clone()) {
            ThreadCommand::Go {
                pos,
                options,
                root_moves,
                info,
            } => {
                shared.num_searching.fetch_add(1, Ordering::Relaxed);

                thread.reset();
                thread.root_moves = root_moves;
                thread.multipv = options.multipv;
                thread.eval_scaling = options.eval_scaling;
                thread.ponder = options.ponder;
                thread.frc = options.frc;

                id_loop(pos, &mut thread, &shared, info);
            }
            ThreadCommand::SetShared(new_shared) => {
                shared = new_shared;
                thread.nodes = BatchedAtomicCounter::new(shared.nodes.clone());
            }
            ThreadCommand::NewGame => {
                thread.history = unsafe { Box::new_zeroed().assume_init() };
            }
            ThreadCommand::Quit => return,
        }
    }
}
