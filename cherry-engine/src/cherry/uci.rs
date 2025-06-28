use std::{str::FromStr, time::Duration};
use cozy_chess::util::parse_uci_move;
use cozy_chess::*;
use super::SearchLimit;

pub type Result<T> = std::result::Result<T, UciParseError>;

#[derive(Debug, Clone)]
pub enum UciCommand {
    Uci,
    NewGame,
    IsReady,
    PonderHit,
    Position(Board, Vec<Move>),
    Go(Vec<SearchLimit>),
    SetOption(String, String),
    #[cfg(feature = "tune")] Tune(String, String),
    #[cfg(feature = "parse")] Parse(String, String),
    #[cfg(feature = "tune")] DataGen(String, u16, u64),
    Debug(bool),
    Display,
    Eval,
    #[cfg(feature = "trace")] Trace,
    Stop,
    Quit
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UciParseError {
    InvalidCommand,
    InvalidArguments,
}

impl FromStr for UciCommand {
    type Err = UciParseError;

    fn from_str(s: &str) -> Result<Self> {
        let mut reader = s.split_ascii_whitespace();
        let token = reader.next().ok_or(UciParseError::InvalidCommand)?;
        
        match token {
            "uci" => Ok(UciCommand::Uci),
            "ucinewgame" | "newgame" => Ok(UciCommand::NewGame),
            "isready" => Ok(UciCommand::IsReady),
            "eval" => Ok(UciCommand::Eval),
            "stop" => Ok(UciCommand::Stop),
            "quit" | "q" => Ok(UciCommand::Quit),
            "d" | "display" | "print" => Ok(UciCommand::Display),
            "ponderhit" => Ok(UciCommand::PonderHit),
            "position" | "pos" => {
                let board_kind = reader.next().ok_or(UciParseError::InvalidArguments)?;
                let mut moves_token_passed = false;
                let board = match board_kind {
                    "startpos" => Board::default(),
                    "fen" => {
                        let mut fen = String::new();
                        
                        while let Some(part) = reader.next() {
                            if part == "moves" {
                                moves_token_passed = true;
                                break;
                            }
                            
                            fen += part;
                            fen += " ";
                        }
                        
                        Board::from_str(fen.trim()).map_err(|_| UciParseError::InvalidArguments)?
                    },
                    _ => return Err(UciParseError::InvalidArguments)
                };

                if !moves_token_passed {
                    moves_token_passed = reader.next().is_some_and(|s| s == "moves");
                }
                
                let mut moves = Vec::new();
                
                if moves_token_passed {
                    let mut board_copy = board.clone();

                    while let Some(mv_token) = reader.next() {
                        let mv = parse_uci_move(&board_copy, mv_token).map_err(|_| UciParseError::InvalidArguments)?;
                        
                        if board_copy.try_play(mv).is_err() {
                            break;
                        }

                        moves.push(mv);
                    }
                }
                
                Ok(UciCommand::Position(board, moves))
            },
            "go" => {
                let mut options= Vec::new();

                while let Some(token) = reader.next() {
                    options.push(match token {
                        "wtime" => {
                            let millis = reader.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::WhiteTime(Duration::from_millis(millis))
                        },
                        "btime" => {
                            let millis = reader.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::BlackTime(Duration::from_millis(millis))
                        },
                        "winc" => {
                            let millis = reader.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::WhiteInc(Duration::from_millis(millis))
                        },
                        "binc" => {
                            let millis = reader.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::BlackInc(Duration::from_millis(millis))
                        },
                        "movetime" => {
                            let millis = reader.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::MoveTime(Duration::from_millis(millis))
                        }
                        "movestogo" => {
                            let moves_to_go = reader.next().and_then(|s| s.parse::<u16>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::MovesToGo(moves_to_go)
                        },
                        "depth" =>{
                            let depth = reader.next().and_then(|s| s.parse::<u8>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::MaxDepth(depth)
                        },
                        "nodes" => {
                            let nodes = reader.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::MaxNodes(nodes)
                        },
                        "infinite" => SearchLimit::Infinite,
                        "ponder" => SearchLimit::Ponder,
                        _ => return Err(UciParseError::InvalidArguments)
                    });
                }

                Ok(UciCommand::Go(options))
            },
            "setoption" => {
                reader.next().filter(|&s| s == "name").ok_or(UciParseError::InvalidArguments)?;
                
                let mut name = String::new();
                while let Some(token) = reader.next() {
                    if token == "value" {
                        break;
                    }
                    
                    name += token;
                }

                let value = reader.remainder().ok_or(UciParseError::InvalidArguments)?.to_owned();
                
                Ok(UciCommand::SetOption(name, value))
            },
            "debug" => {
                let value = reader.next().is_some_and(|s| s == "on");
                
                Ok(UciCommand::Debug(value))
            }
            #[cfg(feature = "tune")] "tune" => {
                reader.next().filter(|&s| s == "data").ok_or(UciParseError::InvalidArguments)?;
                let data_path = reader.next().ok_or(UciParseError::InvalidArguments)?.to_owned();
                reader.next().filter(|&s| s == "out").ok_or(UciParseError::InvalidArguments)?;
                let out_path = reader.next().ok_or(UciParseError::InvalidArguments)?.to_owned();

                Ok(UciCommand::Tune(data_path, out_path))
            },
            #[cfg(feature = "parse")] "parse" => {
                reader.next().filter(|&s| s == "data").ok_or(UciParseError::InvalidArguments)?;
                let data_path = reader.next().ok_or(UciParseError::InvalidArguments)?.to_owned();
                reader.next().filter(|&s| s == "out").ok_or(UciParseError::InvalidArguments)?;
                let out_path = reader.next().ok_or(UciParseError::InvalidArguments)?.to_owned();

                Ok(UciCommand::Parse(data_path, out_path))
            },
            #[cfg(feature = "tune")] "datagen" => {
                reader.next().filter(|&s| s == "threads").ok_or(UciParseError::InvalidArguments)?;
                let threads = reader.next().and_then(|s| s.parse::<u16>().ok()).ok_or(UciParseError::InvalidArguments)?;
                reader.next().filter(|&s| s == "movetime").ok_or(UciParseError::InvalidArguments)?;
                let move_time = reader.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                reader.next().filter(|&s| s == "out").ok_or(UciParseError::InvalidArguments)?;
                let out_path = reader.next().ok_or(UciParseError::InvalidArguments)?.to_owned();

                Ok(UciCommand::DataGen(out_path, threads, move_time))
            },
            #[cfg(feature = "trace")] "trace" => Ok(UciCommand::Trace),
            _ => Err(UciParseError::InvalidCommand),
        }
    }
}