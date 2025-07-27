use std::{fmt, str::FromStr, time::Duration};
use cherry_core::*;

/*----------------------------------------------------------------*/

pub type Result<T> = std::result::Result<T, UciParseError>;

#[derive(Debug, Clone)]
pub enum UciCommand {
    Uci,
    NewGame,
    IsReady,
    PonderHit,
    Position(Board, Vec<Move>),
    Go(Vec<SearchLimit>),
    SetOption {
        name: String,
        value: String,
    },
    Display,
    Bench {
        depth: u8,
        threads: u16,
        hash: u16,
    },
    #[cfg(feature = "tune")] Tune {
        threads: usize,
        buffer_size: usize,
        queue_size: usize,
        file_paths: Vec<String>,
    },
    Stop,
    Quit
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchLimit {
    SearchMoves(Vec<String>),
    WhiteTime(Duration),
    BlackTime(Duration),
    WhiteInc(Duration),
    BlackInc(Duration),
    MoveTime(Duration),
    MovesToGo(u16),
    MaxDepth(u8),
    MaxNodes(u64),
    Infinite,
    Ponder,
}

/*----------------------------------------------------------------*/

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
            "bench" => Ok(UciCommand::Bench {
                depth: reader.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(12),
                threads: reader.next().and_then(|s| s.parse::<u16>().ok()).unwrap_or(1),
                hash: reader.next().and_then(|s| s.parse::<u16>().ok()).unwrap_or(16)
            }),
            "position" => {
                let board_kind = reader.next().ok_or(UciParseError::InvalidArguments)?;
                let mut moves_token_passed = false;
                let board = match board_kind {
                    "startpos" => Board::default(),
                    "fen" => {
                        let mut fen = String::new();
                        
                        while let Some(part) = reader.next() {
                            //TODO: could maybe use peekable here as well?
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
                let mut tokens = reader.peekable();
                let keywords = &[
                    "searchmoves", "wtime", "btime", "winc", "binc", "movetime",
                    "movestogo", "depth", "nodes", "infinite", "ponder"
                ];

                while let Some(&token) = tokens.peek() {
                    tokens.next();

                    options.push(match token {
                        "searchmoves" => {
                            let mut moves = Vec::new();

                            while let Some(&mv_token) = tokens.peek() {
                                if keywords.contains(&mv_token) {
                                    break;
                                }

                                moves.push(tokens.next().unwrap().to_owned());
                            }

                            SearchLimit::SearchMoves(moves)
                        }
                        "wtime" => {
                            let millis = tokens.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::WhiteTime(Duration::from_millis(millis))
                        },
                        "btime" => {
                            let millis = tokens.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::BlackTime(Duration::from_millis(millis))
                        },
                        "winc" => {
                            let millis = tokens.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::WhiteInc(Duration::from_millis(millis))
                        },
                        "binc" => {
                            let millis = tokens.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::BlackInc(Duration::from_millis(millis))
                        },
                        "movetime" => {
                            let millis = tokens.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::MoveTime(Duration::from_millis(millis))
                        }
                        "movestogo" => {
                            let moves_to_go = tokens.next().and_then(|s| s.parse::<u16>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::MovesToGo(moves_to_go)
                        },
                        "depth" =>{
                            let depth = tokens.next().and_then(|s| s.parse::<u8>().ok()).ok_or(UciParseError::InvalidArguments)?;
                            SearchLimit::MaxDepth(depth)
                        },
                        "nodes" => {
                            let nodes = tokens.next().and_then(|s| s.parse::<u64>().ok()).ok_or(UciParseError::InvalidArguments)?;
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

                let value = reader.remainder()
                    .map(str::to_owned)
                    .unwrap_or(String::from("<empty>"));
                
                Ok(UciCommand::SetOption { name, value })
            },
            #[cfg(feature = "tune")] "tune" => {
                let threads = reader.next().and_then(|s| s.parse::<usize>().ok()).ok_or(UciParseError::InvalidArguments)?;
                let buffer_size = reader.next().and_then(|s| s.parse::<usize>().ok()).ok_or(UciParseError::InvalidArguments)?;
                let queue_size = reader.next().and_then(|s| s.parse::<usize>().ok()).ok_or(UciParseError::InvalidArguments)?;
                let mut file_paths = Vec::new();

                while let Some(file_path) = reader.next() {
                    file_paths.push(file_path.to_owned());
                }

                if file_paths.is_empty() {
                    return Err(UciParseError::InvalidArguments);
                }

                Ok(UciCommand::Tune {
                    threads,
                    buffer_size,
                    queue_size,
                    file_paths
                })
            },
            _ => Err(UciParseError::InvalidCommand),
        }
    }
}