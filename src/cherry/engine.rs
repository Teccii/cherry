use std::{
    sync::{Arc, Mutex, mpsc::*},
    cell::RefCell,
    fmt::Write,
    rc::Rc,
};
use std::time::Instant;
use crate::*;

/*----------------------------------------------------------------*/

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

const BENCH_POSITIONS: &[&str] = &[
    "r3k2r/2pb1ppp/2pp1q2/p7/1nP1B3/1P2P3/P2N1PPP/R2QK2R w KQkq a6 0 14",
    "4rrk1/2p1b1p1/p1p3q1/4p3/2P2n1p/1P1NR2P/PB3PP1/3R1QK1 b - - 2 24",
    "r3qbrk/6p1/2b2pPp/p3pP1Q/PpPpP2P/3P1B2/2PB3K/R5R1 w - - 16 42",
    "6k1/1R3p2/6p1/2Bp3p/3P2q1/P7/1P2rQ1K/5R2 b - - 4 44",
    "8/8/1p2k1p1/3p3p/1p1P1P1P/1P2PK2/8/8 w - - 3 54",
    "7r/2p3k1/1p1p1qp1/1P1Bp3/p1P2r1P/P7/4R3/Q4RK1 w - - 0 36",
    "r1bq1rk1/pp2b1pp/n1pp1n2/3P1p2/2P1p3/2N1P2N/PP2BPPP/R1BQ1RK1 b - - 2 10",
    "3r3k/2r4p/1p1b3q/p4P2/P2Pp3/1B2P3/3BQ1RP/6K1 w - - 3 87",
    "2r4r/1p4k1/1Pnp4/3Qb1pq/8/4BpPp/5P2/2RR1BK1 w - - 0 42",
    "4q1bk/6b1/7p/p1p4p/PNPpP2P/KN4P1/3Q4/4R3 b - - 0 37",
    "2q3r1/1r2pk2/pp3pp1/2pP3p/P1Pb1BbP/1P4Q1/R3NPP1/4R1K1 w - - 2 34",
    "1r2r2k/1b4q1/pp5p/2pPp1p1/P3Pn2/1P1B1Q1P/2R3P1/4BR1K b - - 1 37",
    "r3kbbr/pp1n1p1P/3ppnp1/q5N1/1P1pP3/P1N1B3/2P1QP2/R3KB1R b KQkq b3 0 17",
    "8/6pk/2b1Rp2/3r4/1R1B2PP/P5K1/8/2r5 b - - 16 42",
    "1r4k1/4ppb1/2n1b1qp/pB4p1/1n1BP1P1/7P/2PNQPK1/3RN3 w - - 8 29",
    "8/p2B4/PkP5/4p1pK/4Pb1p/5P2/8/8 w - - 29 68",
    "3r4/ppq1ppkp/4bnp1/2pN4/2P1P3/1P4P1/PQ3PBP/R4K2 b - - 2 20",
    "5rr1/4n2k/4q2P/P1P2n2/3B1p2/4pP2/2N1P3/1RR1K2Q w - - 1 49",
    "1r5k/2pq2p1/3p3p/p1pP4/4QP2/PP1R3P/6PK/8 w - - 1 51",
    "q5k1/5ppp/1r3bn1/1B6/P1N2P2/BQ2P1P1/5K1P/8 b - - 2 34",
    "r1b2k1r/5n2/p4q2/1ppn1Pp1/3pp1p1/NP2P3/P1PPBK2/1RQN2R1 w - - 0 22",
    "r1bqk2r/pppp1ppp/5n2/4b3/4P3/P1N5/1PP2PPP/R1BQKB1R w KQkq - 0 5",
    "r1bqr1k1/pp1p1ppp/2p5/8/3N1Q2/P2BB3/1PP2PPP/R3K2n b Q - 1 12",
    "r1bq2k1/p4r1p/1pp2pp1/3p4/1P1B3Q/P2B1N2/2P3PP/4R1K1 b - - 2 19",
    "r4qk1/6r1/1p4p1/2ppBbN1/1p5Q/P7/2P3PP/5RK1 w - - 2 25",
    "r7/6k1/1p6/2pp1p2/7Q/8/p1P2K1P/8 w - - 0 32",
    "r3k2r/ppp1pp1p/2nqb1pn/3p4/4P3/2PP4/PP1NBPPP/R2QK1NR w KQkq - 1 5",
    "3r1rk1/1pp1pn1p/p1n1q1p1/3p4/Q3P3/2P5/PP1NBPPP/4RRK1 w - - 0 12",
    "5rk1/1pp1pn1p/p3Brp1/8/1n6/5N2/PP3PPP/2R2RK1 w - - 2 20",
    "8/1p2pk1p/p1p1r1p1/3n4/8/5R2/PP3PPP/4R1K1 b - - 3 27",
    "8/4pk2/1p1r2p1/p1p4p/Pn5P/3R4/1P3PP1/4RK2 w - - 1 33",
    "8/5k2/1pnrp1p1/p1p4p/P6P/4R1PK/1P3P2/4R3 b - - 1 38",
    "8/8/1p1kp1p1/p1pr1n1p/P6P/1R4P1/1P3PK1/1R6 b - - 15 45",
    "8/8/1p1k2p1/p1prp2p/P2n3P/6P1/1P1R1PK1/4R3 b - - 5 49",
    "8/8/1p4p1/p1p2k1p/P2npP1P/4K1P1/1P6/3R4 w - - 6 54",
    "8/8/1p4p1/p1p2k1p/P2n1P1P/4K1P1/1P6/6R1 b - - 6 59",
    "8/5k2/1p4p1/p1pK3p/P2n1P1P/6P1/1P6/4R3 b - - 14 63",
    "8/1R6/1p1K1kp1/p6p/P1p2P1P/6P1/1Pn5/8 w - - 0 67",
    "1rb1rn1k/p3q1bp/2p3p1/2p1p3/2P1P2N/PP1RQNP1/1B3P2/4R1K1 b - - 4 23",
    "4rrk1/pp1n1pp1/q5p1/P1pP4/2n3P1/7P/1P3PB1/R1BQ1RK1 w - - 3 22",
    "r2qr1k1/pb1nbppp/1pn1p3/2ppP3/3P4/2PB1NN1/PP3PPP/R1BQR1K1 w - - 4 12",
    "2r2k2/8/4P1R1/1p6/8/P4K1N/7b/2B5 b - - 0 55",
    "6k1/5pp1/8/2bKP2P/2P5/p4PNb/B7/8 b - - 1 44",
    "2rqr1k1/1p3p1p/p2p2p1/P1nPb3/2B1P3/5P2/1PQ2NPP/R1R4K w - - 3 25",
    "r1b2rk1/p1q1ppbp/6p1/2Q5/8/4BP2/PPP3PP/2KR1B1R b - - 2 14",
    "6r1/5k2/p1b1r2p/1pB1p1p1/1Pp3PP/2P1R1K1/2P2P2/3R4 w - - 1 36",
    "rnbqkb1r/pppppppp/5n2/8/2PP4/8/PP2PPPP/RNBQKBNR b KQkq c3 0 2",
    "2rr2k1/1p4bp/p1q1p1p1/4Pp1n/2PB4/1PN3P1/P3Q2P/2RR2K1 w - f6 0 20",
    "3br1k1/p1pn3p/1p3n2/5pNq/2P1p3/1PN3PP/P2Q1PB1/4R1K1 w - - 0 23",
    "2r2b2/5p2/5k2/p1r1pP2/P2pB3/1P3P2/K1P3R1/7R w - - 23 93",
];

/*----------------------------------------------------------------*/

pub enum ThreadCommand {
    Go(Arc<Mutex<Searcher>>, Vec<SearchLimit>),
    SetOption(Arc<Mutex<Searcher>>, String, String),
    Position(Arc<Mutex<Searcher>>, Board, Vec<Move>),
    NewGame(Arc<Mutex<Searcher>>),
    Quit,
}

pub struct Engine {
    searcher: Arc<Mutex<Searcher>>,
    time_man: Arc<TimeManager>,
    sender: Sender<ThreadCommand>,
    chess960: Rc<RefCell<bool>>,
}

impl Engine {
    pub fn new() -> Engine {
        #[cfg(feature = "nnue")] let nnue_weights = NetworkWeights::default();
        let time_man = Arc::new(TimeManager::new());
        let searcher = Arc::new(Mutex::new(Searcher::new(
            Board::default(),
            Arc::clone(&time_man),
            #[cfg(feature = "nnue")] nnue_weights,
        )));

        let (tx, rx): (Sender<ThreadCommand>, Receiver<ThreadCommand>) = channel();
        std::thread::spawn(move || loop {
            if let Ok(cmd) = rx.recv() {
                match cmd {
                    ThreadCommand::Go(searcher, limits) => {
                        let mut searcher = searcher.lock().unwrap();
                        let mut output = String::new();

                        let (mv, ponder, _, _, _) = searcher.search::<UciInfo>(limits);
                        write!(output, "bestmove {}", mv.display(&searcher.pos.board(), searcher.chess960)).unwrap();

                        if let Some(ponder) = ponder {
                            write!(output, " ponder {}", ponder).unwrap();
                        }

                        println!("{}", output);
                    },
                    ThreadCommand::Position(searcher, board, moves) => {
                        let mut searcher = searcher.lock().unwrap();
                        let searcher = &mut *searcher; //???

                        #[cfg(not(feature = "nnue"))] searcher.pos.set_board(board);
                        #[cfg(feature = "nnue")] searcher.pos.set_board(board, &searcher.shared_ctx.nnue_weights);

                        for mv in moves {
                            searcher.pos.make_move(mv);

                            #[cfg(feature = "nnue")] searcher.pos.reset(&searcher.shared_ctx.nnue_weights);
                        }
                    },
                    ThreadCommand::SetOption(searcher, name, value) => {
                        let mut searcher = searcher.lock().unwrap();

                        match name.as_str() {
                            "Threads" => searcher.threads = value.parse::<u16>().unwrap(),
                            "Hash" => searcher.resize_ttable(value.parse::<usize>().unwrap()),
                            "SyzygyPath" => searcher.set_syzygy_path(value.as_str()),
                            "SyzygyProbeDepth" => searcher.shared_ctx.syzygy_depth = value.parse::<u8>().unwrap(),
                            "Ponder" => searcher.ponder = value.parse::<bool>().unwrap(),
                            "UCI_Chess960" => searcher.chess960 = value.parse::<bool>().unwrap(),
                            _ => { }
                        }
                    },
                    ThreadCommand::NewGame(searcher) => {
                        let mut searcher = searcher.lock().unwrap();
                        searcher.clean_ttable();
                    },
                    ThreadCommand::Quit => return,
                }
            }
        });

        Engine {
            searcher,
            time_man,
            sender: tx,
            chess960: Rc::new(RefCell::new(false)),
        }
    }

    pub fn input(&mut self, input: &str, bytes: usize) -> bool {
        let cmd = if bytes == 0 { UciCommand::Quit } else {
            match UciCommand::parse(input, *self.chess960.borrow()) {
                Ok(cmd) => cmd,
                Err(e) => {
                    println!("{:?}", e);
                    return true;
                }
            }
        };

        match cmd {
            UciCommand::Uci => {
                println!("id name Cherry v{}", ENGINE_VERSION);
                println!("id author Tecci");
                println!("option name Threads type spin default 1 min 1 max 65535");
                println!("option name Hash type spin default 16 min 1 max 65535");
                println!("option name SyzygyPath type string default <empty>");
                println!("option name SyzygyProbeDepth type spin default 1 min 0 max 128");
                println!("option name Move Overhead type spin default 100 min 0 max 5000");
                println!("option name Ponder type check default false");
                println!("option name UCI_Chess960 type check default false");
                println!("uciok");
            },
            UciCommand::IsReady => println!("readyok"),
            UciCommand::PonderHit => self.time_man.ponderhit(),
            UciCommand::Stop => self.time_man.stop(),
            UciCommand::NewGame => self.sender.send(ThreadCommand::NewGame(Arc::clone(&self.searcher))).unwrap(),
            UciCommand::Display => {
                let searcher = self.searcher.lock().unwrap();
                let board = searcher.pos.board();

                println!("\n{:?}", board);
                println!("FEN: {}", board);
            },
            UciCommand::Position(board, moves) => self.sender.send(ThreadCommand::Position(
                Arc::clone(&self.searcher),
                board,
                moves
            )).unwrap(),
            UciCommand::Go(limits) => self.sender.send(ThreadCommand::Go(
                Arc::clone(&self.searcher),
                limits
            )).unwrap(),
            UciCommand::SetOption { name, value } => {
                match name.as_str() {
                    "Move Overhead" => self.time_man.set_overhead(value.parse::<u64>().unwrap()),
                    "Chess960" => *self.chess960.borrow_mut() = value.parse::<bool>().unwrap(),
                    _ => { }
                }

                self.sender.send(ThreadCommand::SetOption(
                    Arc::clone(&self.searcher),
                    name,
                    value
                )).unwrap();
            },
            UciCommand::Bench { depth, threads, hash } => {
                let mut searcher = self.searcher.lock().unwrap();
                let searcher = &mut *searcher;
                let mut bench_data = Vec::new();
                let limits = vec![SearchLimit::MaxDepth(depth)];

                searcher.resize_ttable(hash as usize);
                searcher.threads = threads;

                let start_time = Instant::now();
                for pos in BENCH_POSITIONS.iter().map(|fen| fen.parse::<Board>().unwrap()) {
                    #[cfg(not(feature = "nnue"))] searcher.pos.set_board(pos.clone());
                    #[cfg(feature = "nnue")] searcher.pos.set_board(pos.clone(), &searcher.shared_ctx.nnue_weights);
                    searcher.clean_ttable();

                    let start_time = Instant::now();
                    let (best_move, _, score, _, nodes) = searcher.search::<NoInfo>(limits.clone());
                    bench_data.push((
                        best_move.display(&pos, false),
                        start_time.elapsed().as_millis() as u64,
                        score.0,
                        nodes
                    ));
                }

                let total_time = start_time.elapsed().as_millis() as u64;

                println!("\n================================================================");
                for (i, (best_move, time, score, nodes)) in bench_data.iter().enumerate() {
                    println!(
                        "[#{:>3}]{:>8} cp Best: {:>8} {:>8} nodes {:>8} nps",
                        i + 1,
                        score,
                        best_move,
                        nodes,
                        (*nodes / *time) * 1000,
                    );
                }
                println!("==================================================================");
                let total_nodes = bench_data.iter()
                    .fold(0u64, |acc, (_, _, _, nodes)| acc + nodes);

                println!(
                    "OVERALL: {:>30} nodes {:>8} nps",
                    total_nodes,
                    (total_nodes / total_time) * 1000
                );
            },
            #[cfg(feature = "datagen")] UciCommand::DataGen {
                count,
                seed,
                moves,
            } => datagen(count, seed, moves),
            #[cfg(feature = "tune")] UciCommand::Tune {
                threads,
                buffer_size,
                queue_size,
                file_paths
            } => {
                let file_paths = file_paths.iter().map(String::as_str).collect::<Vec<_>>();

                tune(threads, buffer_size, queue_size, &file_paths);
            },
            UciCommand::Quit => {
                self.time_man.stop();
                self.sender.send(ThreadCommand::Quit).unwrap();
                return false;
            },
        }
        
        true
    }
}