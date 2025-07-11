use std::{str::FromStr, time::Duration};
use super::SearchLimit;
use cherry_core::*;

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
    Debug(bool),
    Display,
    Stop,
    Quit
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UciParseError {
    InvalidCommand,
    InvalidArguments,
}

impl UciCommand {
    pub fn parse(s: &str, chess960: bool) -> Result<Self> {
        let mut reader = s.split_ascii_whitespace();
        let token = reader.next().ok_or(UciParseError::InvalidCommand)?;
        
        match token {
            "uci" => Ok(UciCommand::Uci),
            "ucinewgame" => Ok(UciCommand::NewGame),
            "isready" => Ok(UciCommand::IsReady),
            "stop" => Ok(UciCommand::Stop),
            "quit" | "q" => Ok(UciCommand::Quit),
            "display" | "d" => Ok(UciCommand::Display),
            "ponderhit" => Ok(UciCommand::PonderHit),
            "position" => {
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
                        let mv = Move::parse(&board_copy, chess960, mv_token.trim()).map_err(|_| UciParseError::InvalidArguments)?;
                        
                        if board_copy.is_legal(mv) {
                            board_copy.make_move(mv);
                        } else {
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
            _ => Err(UciParseError::InvalidCommand),
        }
    }
}