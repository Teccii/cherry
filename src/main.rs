#![feature(str_split_whitespace_remainder)]

mod cherry;

use std::{
    sync::{Arc, Mutex},
    sync::mpsc::*,
    fmt::Write,
    io
};
use cozy_chess::*;

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

    let (tx, rx): (Sender<ThreadCommand>, Receiver<ThreadCommand>) = channel();

    std::thread::spawn(move || loop {
        if let Ok(cmd) = rx.recv() {
            match cmd {
                ThreadCommand::Go(searcher, limits) => {
                    let mut searcher = searcher.lock().unwrap();
                    let mut output = String::new();
                    
                    let (mv, ponder, _, _, _) = searcher.search(&limits, true);
                    write!(output, "bestmove {}", convert_move(searcher.pos.board(), mv, chess960)).unwrap();

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
                    return Err(e);
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

                for mut mv in moves {
                    mv = convert_move(searcher.pos.board(), mv, chess960);
                    
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
                    }
                    _ => {}
                }
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
            #[cfg(feature="tune")] UciCommand::Tune(data_path, out_path) => tune::tune(&data_path, &out_path),
            #[cfg(feature = "tune")] UciCommand::DataGen(out_path, threads, move_time) => datagen::datagen(&out_path, threads, move_time),
            UciCommand::Eval => {
                let searcher = searcher.lock().unwrap();
                println!("{}", searcher.pos.eval(0));
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