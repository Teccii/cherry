use std::{
    sync::atomic::*,
    time::{Duration, Instant},
};

use crate::*;

/*----------------------------------------------------------------*/

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

const BENCH_FENS: &[&str] = &[
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

fn perft<const BULK: bool>(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let move_list = board.gen_moves();

    if BULK && depth == 1 {
        nodes += move_list.len() as u64;
    } else {
        for &mv in move_list.iter() {
            let mut board = board.clone();
            board.make_move(mv);

            nodes += perft::<BULK>(&board, depth - 1);
        }
    }

    nodes
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Abort {
    Yes,
    No,
}

#[derive(Copy, Clone)]
pub struct EngineOptions {
    pub multipv: u8,
    pub eval_scaling: bool,
    pub move_overhead: u64,
    pub soft_target: bool,
    pub ponder: bool,
    pub frc: bool,
}

impl Default for EngineOptions {
    #[inline]
    fn default() -> Self {
        EngineOptions {
            multipv: 1,
            eval_scaling: true,
            move_overhead: DEFAULT_OVERHEAD,
            soft_target: false,
            ponder: false,
            frc: false,
        }
    }
}

pub struct Engine {
    pos: Position,
    searcher: Searcher,
    options: EngineOptions,
}

impl Engine {
    #[inline]
    pub fn new() -> Engine {
        Engine {
            pos: Position::new(Board::startpos(), NetworkWeights::default()),
            searcher: Searcher::default(),
            options: EngineOptions::default(),
        }
    }

    #[inline]
    pub fn handle(&mut self, input: &str) -> Abort {
        let cmd = match UciCommand::parse(input, self.pos.board(), self.options.frc) {
            Ok(cmd) => cmd,
            Err(e) => {
                println!("info string {e}");
                return Abort::No;
            }
        };

        match cmd {
            UciCommand::Uci => self.uci(),
            UciCommand::NewGame => self.searcher.newgame(),
            UciCommand::IsReady => self.isready(),
            UciCommand::PonderHit => self.searcher.ponderhit(),
            UciCommand::Eval => self.eval(),
            UciCommand::Display => self.display(),
            UciCommand::Position { board, moves } => self.set_position(board, moves),
            UciCommand::Go(limits) => self.go(limits),
            UciCommand::Perft { depth, bulk } => self.perft(depth, bulk),
            UciCommand::SplitPerft { depth, bulk } => self.splitperft(depth, bulk),
            UciCommand::SetOption { name, value } => self.set_option(name, value),
            UciCommand::Bench { depth } => self.bench(depth),
            UciCommand::Wait => self.wait(),
            UciCommand::Stop => self.stop(),
            UciCommand::Quit => return self.quit(),
        }

        Abort::No
    }

    #[inline]
    fn uci(&self) {
        println!("id name Cherry v{ENGINE_VERSION}-dev");
        println!("id author Tecci");
        println!("option name Threads type spin default 1 min 1 max {MAX_THREADS}");
        println!("option name Hash type spin default 16 min 1 max {MAX_TT_SIZE}");
        println!("option name MultiPV type spin default 1 min 1 max 218");
        println!("option name EvalScaling type check default true");
        println!("option name SyzygyPath type string default <empty>");
        println!("option name MoveOverhead type spin default {DEFAULT_OVERHEAD} min 0 max 5000");
        println!("option name SoftTarget type check default false");
        println!("option name Ponder type check default false");
        println!("option name UCI_Chess960 type check default false");
        println!("uciok");
    }

    #[inline]
    fn isready(&self) {
        println!("readyok");
    }

    #[inline]
    fn eval(&mut self) {
        let raw_eval = self.pos.eval();
        let static_eval = scale_eval(raw_eval, self.pos.board(), self.options.eval_scaling);

        println!("Raw Eval: {}", raw_eval);
        println!("Static Eval: {}", static_eval);
    }

    #[inline]
    fn display(&self) {
        println!("{}", self.pos.board().print(self.options.frc));
    }

    #[inline]
    fn set_position(&mut self, board: Board, moves: Vec<Move>) {
        self.pos.set_board(board);
        for mv in moves {
            self.pos.make_move(mv);
            self.pos.reset_nnue();
        }
    }

    #[inline]
    fn go(&mut self, limits: Vec<SearchLimit>) {
        if self.searcher.is_searching() {
            println!("info string Already Searching");
            return;
        }

        self.searcher.search(
            self.pos.clone(),
            limits,
            self.options,
            SearchInfo::Uci {
                frc: self.options.frc,
            },
        );
    }

    #[inline]
    fn perft(&mut self, depth: u8, bulk: bool) {
        let board = self.pos.board().clone();
        let time = Instant::now();
        let nodes = if bulk {
            perft::<true>(&board, depth)
        } else {
            perft::<false>(&board, depth)
        };
        let elapsed = time.elapsed();
        let nanos = elapsed.as_nanos();
        let nps = if nanos > 0 {
            (nodes as u128 * 1_000_000_000) / nanos
        } else {
            0
        };

        println!("nodes {nodes} time {elapsed:.2?} nps {nps}");
    }

    #[inline]
    fn splitperft(&mut self, depth: u8, bulk: bool) {
        if depth == 0 {
            return;
        }

        let board = self.pos.board().clone();
        let mut perft_data = Vec::new();
        let mut total_time = Duration::ZERO;
        let mut total_nodes = 0u64;

        for &mv in board.gen_moves().iter() {
            let mut board = board.clone();
            board.make_move(mv);

            let time = Instant::now();
            let nodes = if bulk {
                perft::<true>(&board, depth)
            } else {
                perft::<false>(&board, depth)
            };

            total_time += time.elapsed();
            total_nodes += nodes;

            perft_data.push((mv, nodes));
        }

        println!("\n================================================================");
        for (mv, nodes) in perft_data {
            println!("{:<5}: {nodes}", mv.display(&board, self.options.frc));
        }
        println!("================================================================");

        let nanos = total_time.as_nanos();
        let nps = if nanos > 0 {
            (total_nodes as u128 * 1_000_000_000) / nanos
        } else {
            0
        };

        println!("nodes {total_nodes} time {total_time:.2?} nps {nps}");
    }

    #[inline]
    fn bench(&mut self, depth: u8) {
        let limits = vec![SearchLimit::MaxDepth(depth)];
        let mut total_time = Duration::ZERO;
        let mut total_nodes = 0u64;

        for board in BENCH_FENS.iter().map(|&fen| Board::from_fen(fen).unwrap()) {
            self.pos.set_board(board);
            self.searcher.newgame();

            let time = Instant::now();
            self.searcher.search(
                self.pos.clone(),
                limits.clone(),
                self.options,
                SearchInfo::None,
            );
            self.searcher.wait();

            total_time += time.elapsed();
            total_nodes += self.searcher.shared.nodes.load(Ordering::Relaxed);
        }

        let nanos = total_time.as_nanos();
        let nps = if nanos > 0 {
            (total_nodes as u128 * 1_000_000_000) / nanos
        } else {
            0
        };

        println!("nodes {total_nodes} time {total_time:.2?} nps {nps}");
    }

    #[inline]
    fn set_option(&mut self, name: String, value: String) {
        match name.as_str() {
            "Threads" => {
                if self.searcher.is_searching() {
                    println!("info string Not Allowed to set Threads while Searching");
                    return;
                }

                let value = match value.parse::<u32>() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("info string {:?}", UciParseError::InvalidInteger(e));
                        return;
                    }
                };

                if value == 0 || value > MAX_THREADS {
                    println!("info string Invalid Number of Threads: `{value}`");
                    return;
                }

                self.searcher.set_threads(value);
                println!("info string Set Threads to {value}");
            }
            "Hash" => {
                if self.searcher.is_searching() {
                    println!("info string Not Allowed to set Hash while Searching");
                    return;
                }

                let value = match value.parse::<u64>() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("info string {:?}", UciParseError::InvalidInteger(e));
                        return;
                    }
                };

                if value == 0 || value > MAX_TT_SIZE {
                    println!("info string Invalid Hash Size: `{value}`");
                    return;
                }

                self.searcher.resize_ttable(value);
                println!("info string Set Hash to {value}");
            }
            "MultiPV" => {
                let value = match value.parse::<u8>() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("info string {:?}", UciParseError::InvalidInteger(e));
                        return;
                    }
                };

                if value == 0 || value > 218 {
                    println!("info string Invalid MultiPV value: `{value}`");
                    return;
                }

                self.options.multipv = value;
                println!("info string Set MultiPV to {value}");
            }
            "EvalScaling" => {
                let value = match value.parse::<bool>() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("info string {:?}", UciParseError::InvalidBoolean(e));
                        return;
                    }
                };

                self.options.eval_scaling = value;
                println!("info string Set EvalScaling to {value}");
            }
            "SyzygyPath" => set_syzygy_path(value.as_str()),
            "MoveOverhead" => {
                let value = match value.parse::<u64>() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("info string {:?}", UciParseError::InvalidInteger(e));
                        return;
                    }
                };

                if value > 5000 {
                    println!("info string Invalid MoveOverhead value: `{value}`");
                    return;
                }

                self.options.move_overhead = value;
                println!("info string Set MoveOverhead to {value}");
            }
            "SoftTarget" => {
                let value = match value.parse::<bool>() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("info string {:?}", UciParseError::InvalidBoolean(e));
                        return;
                    }
                };

                self.options.soft_target = value;
                println!("info string Set SoftTarget to {value}");
            }
            "Ponder" => {
                let value = match value.parse::<bool>() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("info string {:?}", UciParseError::InvalidBoolean(e));
                        return;
                    }
                };

                self.options.ponder = value;
                println!("info string Set Ponder to {value}");
            }
            "UCI_Chess960" => {
                let value = match value.parse::<bool>() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("info string {:?}", UciParseError::InvalidBoolean(e));
                        return;
                    }
                };

                self.options.frc = value;
                println!("info string Set UCI_Chess960 to {value}");
            }
            _ => println!("info string Unknown Option: `{name}`"),
        }
    }

    #[inline]
    fn wait(&self) {
        if !self.searcher.is_searching() {
            println!("info string Not Searching");
        } else {
            println!("info string Waiting for Search to Stop...");
            self.searcher.wait();
            println!("info string Searcher Stopped");
        }
    }

    #[inline]
    fn stop(&self) {
        if self.searcher.is_searching() {
            self.searcher.stop();
            self.searcher.wait();

            println!("info string Searcher Stopped");
        } else {
            println!("info string Not Searching");
        }
    }

    #[inline]
    fn quit(&mut self) -> Abort {
        self.searcher.quit();
        Abort::Yes
    }
}
