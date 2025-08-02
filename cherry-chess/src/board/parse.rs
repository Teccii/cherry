use std::str::FromStr;
use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FenParseError {
    InvalidBoard,
    InvalidCastleRights,
    InvalidEnPassant,
    InvalidHalfMoveClock,
    InvalidFullMoveCount,
    InvalidSideToMove,
    MissingField,
    TooManyFields,
}

impl Board {
    pub fn from_fen(fen: &str, shredder: bool) -> Result<Board, FenParseError> {
        let mut reader = fen.split(' ');
        let mut board = Board {
            colors: [Bitboard::EMPTY; Color::COUNT],
            pieces: [Bitboard::EMPTY; Piece::COUNT],
            castle_rights: [CastleRights { short: None, long: None }; Color::COUNT],
            pinned: Bitboard::EMPTY,
            checkers: Bitboard::EMPTY,
            en_passant: None,
            fullmove_count: 0,
            halfmove_clock: 0,
            pawn_hash: 0,
            hash: 0,
            stm: Color::White,
        };

        let mut next = || reader.next().map(str::trim).ok_or(FenParseError::MissingField);

        board.parse_board(next()?)?;
        board.parse_stm(next()?)?;

        if !board.board_is_sane() {
            return Err(FenParseError::InvalidBoard);
        }

        let (checkers, pinned) = board.checks_and_pins(board.stm);
        board.checkers = checkers;
        board.pinned = pinned;

        if !board.checkers_is_sane() {
            return Err(FenParseError::InvalidBoard);
        }

        board.parse_castle_rights(next()?, shredder)?;
        if !board.castle_rights_is_sane() {
            return Err(FenParseError::InvalidCastleRights);
        }

        board.parse_ep(next()?)?;
        if !board.en_passant_is_sane() {
            return Err(FenParseError::InvalidEnPassant);
        }

        board.parse_halfmove_clock(next()?)?;
        if !board.halfmove_clock_is_sane() {
            return Err(FenParseError::InvalidHalfMoveClock);
        }

        board.parse_fullmove_count(next()?)?;
        if !board.fullmove_count_is_sane() {
            return Err(FenParseError::InvalidFullMoveCount);
        }

        if reader.next().is_some() {
            return Err(FenParseError::TooManyFields);
        }

        Ok(board)
    }

    fn parse_board(&mut self, s: &str) -> Result<(), FenParseError> {
        for (rank, row) in s.rsplit('/').enumerate() {
            let rank = Rank::try_index(rank).ok_or(FenParseError::InvalidBoard)?;
            let mut file = 0;

            for p in row.chars() {
                if let Some(empty) = p.to_digit(10) {
                    file += empty as usize;
                } else {
                    let piece = p.try_into().map_err(|_| FenParseError::InvalidBoard)?;
                    let color = Color::index(p.is_ascii_lowercase() as usize);

                    let sq = Square::new(File::try_index(file).ok_or(FenParseError::InvalidBoard)?, rank);
                    self.xor_square(piece, color, sq);

                    file += 1;
                }
            }

            if file != File::COUNT {
                return Err(FenParseError::InvalidBoard);
            }
        }

        Ok(())
    }

    fn parse_stm(&mut self, s: &str) -> Result<(), FenParseError> {
        if s.len() != 1 {
            return Err(FenParseError::InvalidSideToMove);
        }

        if Color::try_from(s.chars().next().unwrap())
            .map_err(|_| FenParseError::InvalidSideToMove)? != self.stm {
            self.toggle_stm();
        }

        Ok(())
    }

    fn parse_castle_rights(&mut self, s: &str, shredder: bool) -> Result<(), FenParseError> {
        if s.len() < 1 || s.len() > 4 {
            return Err(FenParseError::InvalidCastleRights);
        }

        if s != "-" {
            for c in s.chars() {
                let color = Color::index(c.is_ascii_lowercase() as usize);
                let king = self.king(color).file();

                let (short, file) = if shredder {
                    let file = c.try_into().map_err(|_| FenParseError::InvalidCastleRights)?;

                    (king < file, file)
                } else {
                    match c.to_ascii_lowercase() {
                        'k' => (true, File::H),
                        'q' => (false, File::A),
                        _ => return Err(FenParseError::InvalidCastleRights),
                    }
                };

                let rights = self.castle_rights(color);
                let old = if short {
                    rights.short
                } else {
                    rights.long
                };

                if old.is_some() {
                    return Err(FenParseError::InvalidCastleRights);
                }

                self.set_castle_rights(color, Some(file), short);
            }
        }

        Ok(())
    }

    fn parse_ep(&mut self, s: &str) -> Result<(), FenParseError> {
        if s.len() < 1 || s.len() > 2 {
            return Err(FenParseError::InvalidEnPassant);
        }

        if s != "-" {
            let sq = s.parse::<Square>().map_err(|_| FenParseError::InvalidEnPassant)?;
            if sq.rank() != Rank::Sixth.relative_to(self.stm) {
                return Err(FenParseError::InvalidEnPassant);
            }

            self.set_en_passant(Some(sq.file()))
        }

        Ok(())
    }

    fn parse_halfmove_clock(&mut self, s: &str) -> Result<(), FenParseError> {
        self.halfmove_clock = s.parse::<u8>().map_err(|_| FenParseError::InvalidHalfMoveClock)?;

        if self.halfmove_clock > 100 {
            return Err(FenParseError::InvalidHalfMoveClock);
        }

        Ok(())
    }

    fn parse_fullmove_count(&mut self, s: &str) -> Result<(), FenParseError> {
        self.fullmove_count = s.parse::<u16>().map_err(|_| FenParseError::InvalidFullMoveCount)?;
        if self.fullmove_count == 0 {
            return Err(FenParseError::InvalidFullMoveCount);
        }

        Ok(())
    }
}

impl FromStr for Board {
    type Err = FenParseError;

    fn from_str(s: &str) -> Result<Board, FenParseError> {
        match Board::from_fen(s, false) {
            Ok(board) => Ok(board),
            Err(FenParseError::InvalidCastleRights) => Board::from_fen(s, true),
            err => err,
        }
    }
}

/*----------------------------------------------------------------*/

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn valid_fens() {
        let positions = include_str!("../../perft/valid.sfens");

        for fen in positions.lines() {
            let board = Board::from_fen(fen, true).unwrap();

            assert!(board.is_sane(), "FEN \"{}\" is valid but insane", fen);
        }
    }

    #[test]
    fn invalid_fens() {
        let positions = include_str!("../../perft/invalid.sfens");

        for fen in positions.lines() {
            assert!(fen.parse::<Board>().is_err(), "FEN \"{}\" is invalid but did not fail to parse", fen);
        }
    }
}