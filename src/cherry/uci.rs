use std::{
    iter::Peekable,
    num::ParseIntError,
    str::{FromStr, ParseBoolError, SplitAsciiWhitespace},
};

use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub enum UciCommand {
    Uci,
    NewGame,
    IsReady,
    PonderHit,
    Eval,
    Display,
    Position {
        board: Board,
        moves: Vec<Move>,
    },
    Go(Vec<SearchLimit>),
    Perft {
        depth: u8,
        bulk: bool,
    },
    SplitPerft {
        depth: u8,
        bulk: bool,
    },
    Bench {
        depth: u8,
    },
    GenFens {
        num: usize,
        seed: u64,
        dfrc: bool,
        moves: usize,
    },
    SetOption {
        name: String,
        value: String,
    },
    Wait,
    Stop,
    Quit,
}

/*----------------------------------------------------------------*/

#[derive(thiserror::Error, Debug, Clone)]
pub enum UciParseError {
    #[error("Missing Command")]
    MissingCommand,
    #[error("Unknown Command: `{0}`")]
    UnknownCommand(String),
    #[error("FRC isn't enabled")]
    FrcNotEnabled,
    #[error("Missing Scharnagl Number")]
    MissingScharnagl,
    #[error("Invalid Scharnagl Number: `{0}`")]
    InvalidScharnagl(u16),
    #[error("Invalid FEN: `{0}`")]
    InvalidFen(String),
    #[error("Invalid Move: `{0}`")]
    InvalidMove(String),
    #[error("Missing position type (e.g. `startpos`, `fen`) in `position` command")]
    MissingPositionType,
    #[error("Missing `moves` token in `position` command")]
    MissingPositionMovesToken,
    #[error("Unknown Search Limit: `{0}`")]
    UnknownLimit(String),
    #[error("Missing value for search limit: `{0}`")]
    MissingLimitValue(String),
    #[error("Missing depth option in `perft` or `splitperft` command")]
    MissingPerftDepth,
    #[error("Missing bulk option in `perft` or `splitperft` command")]
    MissingPerftBulk,
    #[error("Missing Number of Fens")]
    MissingGenFensNumber,
    #[error("Missing `seed` token in `genfens` command")]
    MissingGenFensSeedToken,
    #[error("Missing `book` token in `genfens` command")]
    MissingGenFensBookToken,
    #[error("Missing `dfrc` token in `genfens` command")]
    MissingGenFensDfrcToken,
    #[error("Missing `moves` token in `genfens` command")]
    MissingGenFensMovesToken,
    #[error("Missing `seed` value in `genfens` command")]
    MissingGenFensSeedValue,
    #[error("Missing `book` value in `genfens` command")]
    MissingGenFensBookValue,
    #[error("Missing `dfrc` value in `genfens` command")]
    MissingGenFensDfrcValue,
    #[error("Missing `moves` value in `genfens` command")]
    MissingGenFensMovesValue,
    #[error("Missing `name` token in `setoption` command")]
    MissingOptionNameToken,
    #[error("Missing `value` token in `setoption` command")]
    MissingOptionValueToken,
    #[error("Missing option name in `setoption` command")]
    MissingOptionName,
    #[error("Missing option value in `setoption` command")]
    MissingOptionValue,
    #[error("Error parsing integer: `{0}`")]
    InvalidInteger(#[from] ParseIntError),
    #[error("Error parsing boolean: `{0}`")]
    InvalidBoolean(#[from] ParseBoolError),
}

/*----------------------------------------------------------------*/

impl UciCommand {
    pub fn parse(input: &str, board: &Board, frc: bool) -> Result<UciCommand, UciParseError> {
        use UciCommand::*;
        use UciParseError::*;

        let mut reader = input.split_ascii_whitespace();
        let cmd = reader.next().ok_or(MissingCommand)?;

        match cmd {
            "uci" => Ok(Uci),
            "ucinewgame" => Ok(NewGame),
            "isready" => Ok(IsReady),
            "ponderhit" => Ok(PonderHit),
            "eval" => Ok(Eval),
            "display" | "d" => Ok(Display),
            "wait" => Ok(Wait),
            "stop" => Ok(Stop),
            "quit" | "q" => Ok(Quit),
            "position" => Self::parse_position(reader, frc),
            "go" => Self::parse_go(reader, board),
            "perft" => {
                let depth = reader.next().ok_or(MissingPerftDepth)?.parse::<u8>()?;
                let bulk = reader.next().ok_or(MissingPerftBulk)?.parse::<bool>()?;

                Ok(Perft { depth, bulk })
            }
            "splitperft" => {
                let depth = reader.next().ok_or(MissingPerftDepth)?.parse::<u8>()?;
                let bulk = reader.next().ok_or(MissingPerftBulk)?.parse::<bool>()?;

                Ok(SplitPerft { depth, bulk })
            }
            "bench" => {
                let depth = reader.next().map_or(Ok(12), str::parse)?;

                Ok(Bench { depth })
            }
            "genfens" => {
                let num = reader
                    .next()
                    .ok_or(MissingGenFensNumber)?
                    .parse::<usize>()?;

                if reader.next() != Some("seed") {
                    return Err(MissingGenFensSeedToken);
                }

                let seed = reader
                    .next()
                    .ok_or(MissingGenFensSeedValue)?
                    .parse::<u64>()?;
                if reader.next() != Some("book") {
                    return Err(MissingGenFensBookToken);
                }
                if reader.next().is_none() {
                    return Err(MissingGenFensBookValue);
                }
                if reader.next() != Some("dfrc") {
                    return Err(MissingGenFensDfrcToken);
                }
                let dfrc = reader
                    .next()
                    .ok_or(MissingGenFensDfrcValue)?
                    .parse::<bool>()?;
                if reader.next() != Some("moves") {
                    return Err(MissingGenFensMovesToken);
                }
                let moves = reader
                    .next()
                    .ok_or(MissingGenFensMovesValue)?
                    .parse::<usize>()?;

                Ok(GenFens {
                    num,
                    seed,
                    dfrc,
                    moves,
                })
            }
            "setoption" => {
                if reader.next() != Some("name") {
                    return Err(MissingOptionNameToken);
                }

                let name = reader.next().ok_or(MissingOptionName)?.to_string();
                if reader.next() != Some("value") {
                    return Err(MissingOptionValueToken);
                }

                let value = reader.next().ok_or(MissingOptionValue)?.to_string();
                Ok(SetOption { name, value })
            }
            _ => Err(UnknownCommand(cmd.to_string())),
        }
    }

    fn parse_position(
        mut reader: SplitAsciiWhitespace,
        frc: bool,
    ) -> Result<UciCommand, UciParseError> {
        use UciCommand::*;
        use UciParseError::*;

        let startpos = match reader.next() {
            Some("startpos") => Board::startpos(),
            Some("kiwipete") => Board::from_fen(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            )
            .unwrap(),
            Some("lasker") => Board::from_fen("8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1").unwrap(),
            Some("frc") => {
                if !frc {
                    return Err(FrcNotEnabled);
                }

                let scharnagl = reader.next().ok_or(MissingScharnagl)?.parse()?;
                if scharnagl >= 960 {
                    return Err(InvalidScharnagl(scharnagl));
                }

                Board::frc_startpos(scharnagl)
            }
            Some("dfrc") => {
                if !frc {
                    return Err(FrcNotEnabled);
                }

                let white_scharnagl: u16 = reader.next().ok_or(MissingScharnagl)?.parse()?;
                let black_scharnagl: u16 = reader.next().ok_or(MissingScharnagl)?.parse()?;
                let max = white_scharnagl.max(black_scharnagl);

                if max >= 960 {
                    return Err(InvalidScharnagl(max));
                }

                Board::dfrc_startpos(white_scharnagl, black_scharnagl)
            }
            Some("fen") => {
                let mut fen = String::new();

                for part in reader.by_ref().take(6) {
                    if !fen.is_empty() {
                        fen.push(' ');
                    }

                    fen.push_str(part);
                }

                Board::from_fen(&fen).ok_or(InvalidFen(fen))?
            }
            _ => return Err(MissingPositionType),
        };

        if reader.next().is_some_and(|token| token != "moves") {
            return Err(MissingPositionMovesToken);
        }

        let mut current = startpos.clone();
        let mut moves = Vec::new();

        for token in reader {
            let mv = Move::parse(&current, token.trim())
                .ok_or_else(|| InvalidMove(token.to_string()))?;

            if !current.is_legal(mv) {
                return Err(InvalidMove(token.to_string()));
            }

            moves.push(mv);
            current.make_move(mv);
        }

        Ok(Position {
            board: startpos,
            moves,
        })
    }

    fn parse_go(
        reader: SplitAsciiWhitespace,
        board: &Board,
    ) -> Result<UciCommand, UciParseError> {
        use SearchLimit::*;
        use UciCommand::*;
        use UciParseError::*;

        let keywords = &[
            "searchmoves",
            "wtime",
            "btime",
            "winc",
            "binc",
            "movetime",
            "movestogo",
            "depth",
            "nodes",
            "infinite",
            "ponder",
        ];

        let mut reader = reader.peekable();
        let mut limits = Vec::new();

        #[inline]
        fn parse_int<T: FromStr<Err = ParseIntError>>(
            reader: &mut Peekable<SplitAsciiWhitespace>,
            token: &str,
        ) -> Result<T, UciParseError> {
            Ok(reader
                .next()
                .ok_or_else(|| MissingLimitValue(token.to_string()))?
                .parse::<T>()?)
        }

        while let Some(token) = reader.next() {
            match token {
                "infinite" => {}
                "ponder" => limits.push(Ponder),
                "wtime" => limits.push(WhiteTime(
                    parse_int::<i64>(&mut reader, token)?.max(0) as u64
                )),
                "btime" => limits.push(BlackTime(
                    parse_int::<i64>(&mut reader, token)?.max(0) as u64
                )),
                "winc" => limits.push(WhiteInc(parse_int(&mut reader, token)?)),
                "binc" => limits.push(BlackInc(parse_int(&mut reader, token)?)),
                "movetime" => limits.push(MoveTime(parse_int(&mut reader, token)?)),
                "movestogo" => limits.push(MovesToGo(parse_int(&mut reader, token)?)),
                "depth" => limits.push(MaxDepth(parse_int(&mut reader, token)?)),
                "nodes" => limits.push(MaxNodes(parse_int(&mut reader, token)?)),
                "searchmoves" => {
                    let mut moves = MoveList::empty();
                    while let Some(token) = reader.peek()
                        && !keywords.contains(token)
                    {
                        let mv = Move::parse(board, token.trim())
                            .ok_or_else(|| InvalidMove(token.to_string()))?;
                        if !board.is_legal(mv) {
                            return Err(InvalidMove(token.to_string()));
                        }

                        moves.push(mv);
                        reader.next();
                    }

                    limits.push(SearchMoves(moves))
                }
                _ => return Err(UnknownLimit(token.to_string())),
            }
        }

        Ok(Go(limits))
    }
}
