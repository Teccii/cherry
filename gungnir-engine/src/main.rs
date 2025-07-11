#![feature(str_split_whitespace_remainder)]
#![feature(generic_const_exprs)]

mod gungnir;

use std::{
    sync::{Arc, Mutex},
    sync::mpsc::*,
    fmt::Write as _,
    io::Write,
    io
};
use gungnir_core::*;
use gungnir::*;

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
                    
                    write!(output, "bestmove {}", mv.display(&searcher.pos.board(), chess960)).unwrap();

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
            match UciCommand::parse(buffer.trim(), chess960) {
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
            UciCommand::Debug(value) => debug = value,
            UciCommand::Display => {
                let searcher = searcher.lock().unwrap();
                let board = searcher.pos.board();
                
                println!("{:?}", board);
                println!("FEN: {}", board);
            },
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