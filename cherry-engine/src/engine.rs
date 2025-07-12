use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::*;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum ThreadCommand {
    Go(Arc<Mutex<Searcher>>, Vec<SearchLimit>),
    Quit,
}

pub struct Engine {
    searcher: Arc<Mutex<Searcher>>,
    time_man: Arc<TimeManager>,
    sender: Sender<ThreadCommand>,
    options: HashMap<String, (UciOptionType, Box<dyn Fn(&Self, String)>)>
}

impl Engine {
    pub fn new() -> Engine {
        let time_man = Arc::new(TimeManager::new());
        let searcher = Arc::new(Mutex::new(Searcher::new(
            Board::default(),
            Arc::clone(&time_man),
        )));

        let (tx, rx): (Sender<ThreadCommand>, Receiver<ThreadCommand>) = channel();
        std::thread::spawn(move || loop {
            if let Ok(cmd) = rx.recv() {
                match cmd {
                    ThreadCommand::Go(searcher, limits) => {
                        let mut searcher = searcher.lock().unwrap();
                        let mut output = String::new();

                        let (mv, ponder, _, _, _) = if searcher.debug {
                            searcher.search::<FullInfo>(&limits)
                        } else {
                            searcher.search::<UciOnly>(&limits)
                        };

                        write!(output, "bestmove {}", mv.display(&searcher.pos.board(), searcher.chess960)).unwrap();

                        if let Some(ponder) = ponder {
                            write!(output, " ponder {}", ponder).unwrap();
                        }

                        println!("{}", output);
                    },
                    ThreadCommand::Quit => break,
                }
            }
        });

        let mut options: HashMap<String, (UciOptionType, Box<dyn Fn(&Self, String)>)> = HashMap::new();

        macro_rules! add_option {
            ($options:ident, $engine:ident, $value:ident, $name:expr => $func:block; $option_type:expr) => {
                $options.insert(
                    String::from($name),
                    ($option_type, Box::new(|$engine: &Engine, $value: String| $func))
                );
            }
        }

        add_option!(options, engine, value, "Hash" => {
            let mut searcher = engine.searcher.lock().unwrap();
            searcher.resize_ttable(value.parse::<usize>().unwrap());
        }; UciOptionType::Spin { default: 64, min: 1, max: 65535 });
        add_option!(options, engine, value, "Threads" => {
            let mut searcher = engine.searcher.lock().unwrap();
            searcher.set_threads(value.parse::<u16>().unwrap());
        }; UciOptionType::Spin { default: 1, min: 1, max: 65535 });
        add_option!(options, engine, value, "Move Overhead" => {
            engine.time_man.set_overhead(value.parse::<u64>().unwrap());
        }; UciOptionType::Spin { default: 30, min: 0, max: 65535 });
        add_option!(options, engine, value, "UCI_Chess960" => {
            let mut searcher = engine.searcher.lock().unwrap();
            searcher.set_chess960(value.parse::<bool>().unwrap());
        }; UciOptionType::Spin { default: 30, min: 0, max: 65535 });
        add_option!(options, engine, value, "SyzygyPath" => {
            let mut searcher = engine.searcher.lock().unwrap();
            searcher.set_syzygy_path(&value);
        }; UciOptionType::String { default: String::from("<empty>") });
        add_option!(options, engine, value, "SyzygyProbeDepth" => {
            let mut searcher = engine.searcher.lock().unwrap();
            searcher.set_syzygy_depth(value.parse::<u8>().unwrap());
        }; UciOptionType::Spin { default: 1, min: 1, max: MAX_DEPTH as i32 });

        Engine {
            searcher,
            time_man,
            sender: tx,
            options,
        }
    }

    pub fn parse(&self, input: &str) -> Result<UciCommand> {
        let searcher = self.searcher.lock().unwrap();

        UciCommand::parse(input, searcher.chess960)
    }

    pub fn input(&self, cmd: UciCommand) -> Result<()> {
        match cmd {
            UciCommand::Uci => {
                println!("id name Cherry {}", ENGINE_VERSION);
                println!("id author Tecci");

                for (name, (option_type, _)) in self.options.iter() {
                    println!("option name {} {}", name, option_type);
                }

                println!("uciok");
            },
            UciCommand::NewGame => {
                let mut searcher = self.searcher.lock().unwrap();

                searcher.clean_ttable();
                searcher.pos.reset(Board::default());
            },
            UciCommand::IsReady => println!("readyok"),
            UciCommand::PonderHit => {
                self.time_man.ponderhit();
            },
            UciCommand::Position(board, moves) => {
                let mut searcher = self.searcher.lock().unwrap();
                searcher.pos.reset(board);

                for mv in moves {
                    searcher.pos.make_move(mv);
                }
            },
            UciCommand::Go(limits) => self.sender.send(ThreadCommand::Go(
                self.searcher.clone(),
                limits
            )).map_err(|_| UciParseError::InvalidArguments)?,
            UciCommand::SetOption(name, value) => {
                self.time_man.abort_now();

                if let Some((_, func)) = self.options.get(&name) {
                    func(self, value);
                }
            },
            UciCommand::Debug(value) => {
                let mut searcher = self.searcher.lock().unwrap();
                searcher.set_debug(value);
            },
            UciCommand::Display => {
                let searcher = self.searcher.lock().unwrap();
                let board = searcher.pos.board();

                println!("{:?}", board);
                println!("FEN: {}", board);
            },
            UciCommand::Stop => {
                self.time_man.abort_now()
            },
            UciCommand::Quit => self.sender.send(ThreadCommand::Quit).map_err(|_| UciParseError::InvalidArguments)?,
        }

        Ok(())
    }
}