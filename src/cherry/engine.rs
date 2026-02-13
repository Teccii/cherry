use std::time::*;

use crate::*;

/*----------------------------------------------------------------*/

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    pub pos: Position,
    pub searcher: Searcher,
    pub options: EngineOptions,
}

impl Engine {
    #[inline]
    pub fn new() -> Engine {
        Engine {
            pos: Position::new(Board::startpos()),
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
            UciCommand::IsReady => println!("readyok"),
            UciCommand::PonderHit => self.searcher.ponderhit(),
            UciCommand::Eval => self.eval(),
            UciCommand::Display => self.display(),
            UciCommand::Position { board, moves } => self.set_position(board, moves),
            UciCommand::Go(limits) => self.go(limits),
            UciCommand::Perft { depth, bulk } => self.perft(depth, bulk),
            UciCommand::SplitPerft { depth, bulk } => self.splitperft(depth, bulk),
            UciCommand::SetOption { name, value } => self.set_option(name, value),
            UciCommand::Bench { depth } => self.bench(depth),
            UciCommand::GenFens {
                num,
                seed,
                dfrc,
                moves,
            } => self.gen_fens(num, seed, dfrc, moves),
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

        for (mv, nodes) in perft_data {
            println!("{:<5}: {nodes}", mv.display(&board, self.options.frc));
        }

        let nanos = total_time.as_nanos();
        let nps = if nanos > 0 {
            (total_nodes as u128 * 1_000_000_000) / nanos
        } else {
            0
        };

        println!("\nnodes {total_nodes} time {total_time:.2?} nps {nps}");
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
