#![feature(str_split_whitespace_remainder)]
#![feature(generic_const_exprs)]

mod cherry;

use std::{
    sync::{Arc, Mutex},
    sync::mpsc::*,
    fmt::Write as _,
    io::Write,
    io
};
#[cfg(feature="parse")] use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::BufWriter,
};
#[cfg(feature = "parse")] use pgn_reader::{BufferedReader, Outcome, RawTag, SanPlus, Skip, Visitor};
use cozy_chess::*;
use cozy_chess::util::display_uci_move;
use cherry::*;

/*----------------------------------------------------------------*/

const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

enum ThreadCommand {
    Go(Arc<Mutex<Searcher>>, Vec<SearchLimit>),
    Quit,
}

fn main() -> Result<()> {
    let mut buffer = String::new();
    let time_man = Arc::new(TimeManager::new());
    let searcher = Arc::new(Mutex::new(Searcher::new(
        Board::default(),
        Arc::clone(&time_man),
    )));
    let mut chess960 = false;
    let mut debug = false;

    let (tx, rx): (Sender<ThreadCommand>, Receiver<ThreadCommand>) = channel();

    std::thread::spawn(move || loop {
        if let Ok(cmd) = rx.recv() {
            match cmd {
                ThreadCommand::Go(searcher, limits) => {
                    let mut searcher = searcher.lock().unwrap();
                    let mut output = String::new();
                    
                    let (mv, ponder, _, _, _) = if debug {
                        searcher.search::<FullInfo>(&limits)
                    } else {
                        searcher.search::<UciOnly>(&limits)
                    };
                    
                    if chess960 {
                        write!(output, "bestmove {}", mv).unwrap();
                    } else {
                        write!(output, "bestmove {}", display_uci_move(&searcher.pos.board(), mv)).unwrap();
                    }

                    if let Some(ponder) = ponder {
                        write!(output, " ponder {}", ponder).unwrap();
                    }

                    println!("{}", output);
                },
                ThreadCommand::Quit => break,
            }
        }
    });
    
    while let Ok(bytes) = io::stdin().read_line(&mut buffer) {
        let cmd = if bytes == 0 { UciCommand::Quit } else {
            match buffer.trim().parse::<UciCommand>() {
                Ok(cmd) => cmd,
                Err(e) => {
                    buffer.clear();
                    println!("{:?}", e);
                    continue;
                }
            }
        };
        
        match cmd {
            UciCommand::Uci => {
                println!("id name Cherry {}", ENGINE_VERSION);
                println!("id author Tecci");
                println!("option name Hash type spin default 16 min 1 max 65535");
                println!("option name Threads type spin default 1 min 1 max 65535");
                println!("option name Move Overhead type spin default 30 min 0 max 65535");
                println!("option name UCI_Chess960 type check default false");
                println!("option name SyzygyPath type string default <empty>");
                println!("option name SyzygyProbeDepth type spin default 1 min 1 max 128");
                println!("uciok");
            },
            UciCommand::NewGame => {
                let mut searcher = searcher.lock().unwrap();
                
                searcher.clean_ttable();
                searcher.pos.reset(Board::default());
            },
            UciCommand::IsReady => println!("readyok"),
            UciCommand::PonderHit => {
                time_man.ponderhit();
            },
            UciCommand::Position(board, moves) => {
                let mut searcher = searcher.lock().unwrap();
                searcher.pos.reset(board);

                for mv in moves {
                    searcher.pos.make_move(mv);
                }
            },
            UciCommand::Go(limits) => tx.send(ThreadCommand::Go(
                searcher.clone(),
                limits
            )).map_err(|_| UciParseError::InvalidArguments)?,
            UciCommand::SetOption(name, value) => {
                time_man.abort_now();

                match name.as_str() {
                    "Hash" => {
                        let mut searcher = searcher.lock().unwrap();
                        searcher.resize_ttable(value.parse::<usize>().unwrap());
                    },
                    "Threads" => {
                        let mut searcher = searcher.lock().unwrap();
                        searcher.set_threads(value.parse::<u16>().unwrap());
                    },
                    "MoveOverhead" => {
                        time_man.set_overhead(value.parse::<u64>().unwrap());
                    },
                    "UCI_Chess960" => {
                        let mut searcher = searcher.lock().unwrap();
                        searcher.set_chess960(value.parse::<bool>().unwrap());
                        chess960 = true;
                    },
                    "SyzygyPath" => {
                        let mut searcher = searcher.lock().unwrap();
                        searcher.set_syzygy_path(&value);
                    },
                    "SyzygyProbeDepth" => {
                        let mut searcher = searcher.lock().unwrap();
                        searcher.set_syzygy_depth(value.parse::<u8>().unwrap());
                    }
                    _ => {}
                }
            },
            UciCommand::Debug(value) => {
                time_man.abort_now();
                debug = value;
            },
            UciCommand::Display => {
                let searcher = searcher.lock().unwrap();
                let board = searcher.pos.board();
                
                println!("+-----------------+");
                for &rank in Rank::ALL.iter().rev() {
                    print!("|");
                    for &file in File::ALL.iter() {
                        let sq = Square::new(file, rank);
                        
                        if !board.occupied().has(sq) {
                            print!(" .");
                        } else {
                            if board.colors(Color::White).has(sq) {
                                print!(" {}", char::from(board.piece_on(sq).unwrap()).to_ascii_uppercase());
                            } else {
                                print!(" {}", board.piece_on(sq).unwrap());
                            }
                        }
                    }
                    
                    println!(" |");
                }

                println!("+-----------------+");
                println!("FEN: {}", board);
            },
            #[cfg(feature="tune")] UciCommand::Tune(data_path, out_path) => tune(&data_path, &out_path),
            #[cfg(feature="tune")] UciCommand::Parse(data_path, out_path) => parse(&data_path, &out_path),
            #[cfg(feature="tune")] UciCommand::DataGen(out_path, threads, move_time) => datagen(&out_path, threads, move_time),
            UciCommand::Eval => {
                let mut searcher = searcher.lock().unwrap();
                println!("{}", searcher.pos.eval());
            },
            #[cfg(feature = "trace")] UciCommand::Trace => {
                let searcher = searcher.lock().unwrap();
                println!("{}", searcher.pos.evaluator().trace());
            }
            UciCommand::Stop => time_man.abort_now(),
            UciCommand::Quit => {
                tx.send(ThreadCommand::Quit).map_err(|_| UciParseError::InvalidArguments)?;
                break;
            }
        }
        
        buffer.clear();
    }

    Ok(())
}

/*----------------------------------------------------------------*/

#[cfg(feature="parse")]
struct PgnParser {
    current: Board,
    boards: Vec<Board>,
    outcome: Option<Outcome>
}

#[cfg(feature="parse")]
impl Visitor for PgnParser {
    type Result = Option<(Vec<Board>, f32)>;

    fn begin_game(&mut self) {
        self.boards.clear();
        self.current = Board::default();
    }

    fn tag(&mut self, name: &[u8], value: RawTag<'_>) {
        if name == b"FEN" {
            self.boards.clear();
            self.current = str::from_utf8(value.as_bytes()).unwrap().parse::<Board>().unwrap();
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true)
    }

    fn san(&mut self, san: SanPlus) {
        let mut san_str = String::new();
        san.append_to_string(&mut san_str);

        let mv = cozy_chess::util::parse_san_move(&self.current, &san_str).unwrap();
        self.boards.push(self.current.clone());
        self.current.play_unchecked(mv);
    }

    fn outcome(&mut self, outcome: Option<Outcome>) {
        self.outcome = outcome;
    }

    fn end_game(&mut self) -> Self::Result {
        if let Some(outcome) = self.outcome {
            let mut boards = self.boards.clone();
            boards.push(self.current.clone());

            let result = match outcome {
                Outcome::Decisive { winner } => match winner as usize {
                    0 => 1.0,
                    1 => 0.0,
                    _ => 0.5
                },
                Outcome::Draw => 0.5,
            };

            return Some((boards, result));
        }

        None
    }
}

#[cfg(feature="parse")]
fn parse(data_path: &str, out_path: &str) {
    let data_file = OpenOptions::new()
        .read(true)
        .open(data_path)
        .unwrap();
    
    let mut board_map: HashMap<Board, (f32, u64)> = HashMap::new();
    let mut reader = BufferedReader::new(data_file);
    let mut parser = PgnParser {
        current: Board::default(),
        boards: Vec::new(),
        outcome: None
    };
    
    println!("Parsing data...");
    
    let mut i = 0;
    while let Some((boards, result)) = reader.read_game(&mut parser).unwrap().flatten() {
        for board in boards.iter().skip(24).cloned().filter(|b| !b.in_check()) {
            let pos = Position::new(board.clone());
            
            if pos.is_checkmate() || pos.is_draw() {
                continue;
            }
            
            let data = board_map.entry(board).or_default();
            data.0 += result;
            data.1 += 1;
        }
        
        i += 1;
        
        if i % 1000 == 0 {
            println!("Found and parsed {} games so far", i);
        }
        
        if i % 80000 == 0 {
            println!("Writing to file...");

            let out_file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(out_path)
                .unwrap();
            let mut writer = BufWriter::new(out_file);
            let len = board_map.len();

            for (board, (result, count)) in board_map.iter() {
                writeln!(writer, "{} | {}", board, *result / *count as f32).unwrap();
            }
            
            println!("Wrote {} unique positions to file", len);
            
            board_map.clear();
        }
    }
    
    if board_map.is_empty() {
        return;
    }

    println!("Writing final games to file...");

    let out_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(out_path)
        .unwrap();
    
    let mut writer = BufWriter::new(out_file);
    let len = board_map.len();

    for (board, (result, count)) in board_map.iter() {
        writeln!(writer, "{} | {}", board, *result / *count as f32).unwrap();
    }

    println!("Wrote {} unique positions to file", len);
    println!("Parsed a total of {} games from file", i);
}