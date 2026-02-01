use core::fmt::Write;

use crate::*;

impl Board {
    #[inline]
    pub fn from_fen(fen: &str) -> Option<Board> {
        let mut parts = fen.trim().split_ascii_whitespace();
        let pieces = parts.next()?;
        let stm = parts.next()?;
        let castle_rights = parts.next()?;
        let en_passant = parts.next()?;
        let halfmove_clock = parts.next()?;
        let fullmove_count = parts.next()?;

        if parts.next().is_some() {
            return None;
        }

        let mut board = Board {
            inner: Byteboard(u8x64::splat(0)),
            attack_table: [Wordboard(u16x64::splat(0)); Color::COUNT],
            index_to_square: [IndexToSquare::default(); Color::COUNT],
            index_to_piece: [IndexToPiece::default(); Color::COUNT],
            castle_rights: [CastleRights::default(); Color::COUNT],
            en_passant: None,
            pinned_mask: Wordboard(u16x64::splat(0)),
            pinned: Bitboard::EMPTY,
            fullmove_count: 1,
            halfmove_clock: 0,
            pawn_hash: 0,
            minor_hash: 0,
            major_hash: 0,
            white_hash: 0,
            black_hash: 0,
            hash: 0,
            stm: Color::White,
        };

        let mut white_index = 0;
        let mut black_index = 0;

        for (rank, row) in pieces.rsplit('/').enumerate() {
            let rank = Rank::try_index(rank)?;
            let mut file = 0;

            for p in row.chars() {
                if let Some(empty) = p.to_digit(10) {
                    file += empty as usize;
                } else {
                    let piece = p.try_into().ok()?;
                    let color = Color::index(p.is_ascii_lowercase() as usize);
                    let sq = Square::new(File::try_index(file)?, rank);
                    let index = if piece == Piece::King {
                        PieceIndex(0)
                    } else {
                        let index = match color {
                            Color::White => &mut white_index,
                            Color::Black => &mut black_index,
                        };

                        *index += 1;
                        if *index >= PieceIndex::COUNT {
                            return None;
                        }

                        PieceIndex(*index as u8)
                    };

                    board.set(sq, Place::from_piece(piece, color, index));
                    board.index_to_square[color][index] = Some(sq);
                    board.index_to_piece[color][index] = Some(piece);

                    file += 1;
                }
            }

            if file != File::COUNT {
                return None;
            }
        }

        if board.index_to_piece[Color::White][PieceIndex(0)].is_none()
            || board.index_to_piece[Color::Black][PieceIndex(0)].is_none()
        {
            return None;
        }

        if stm.len() != 1 {
            return None;
        }

        board.stm = stm.chars().next().unwrap().try_into().ok()?;

        if castle_rights.len() < 1 || castle_rights.len() > 4 {
            return None;
        }

        if castle_rights != "-" {
            for c in castle_rights.chars() {
                let color = Color::index(c.is_ascii_lowercase() as usize);
                let our_backrank = Rank::First.relative_to(color);
                let our_king = board.king(color);

                if our_king.rank() != our_backrank {
                    return None;
                }

                let rook_file = match c.to_ascii_lowercase() {
                    'a'..='h' => c.try_into().ok()?,
                    'k' => {
                        let corner_rook = Square::new(File::H, our_backrank);
                        let rook_mask = between(our_king, corner_rook) | corner_rook;
                        let valid_rooks = board.color_pieces(color, Piece::Rook) & rook_mask;

                        valid_rooks.try_next_square_back().map(Square::file)?
                    }
                    'q' => {
                        let corner_rook = Square::new(File::A, our_backrank);
                        let rook_mask = between(our_king, corner_rook) | corner_rook;
                        let valid_rooks = board.color_pieces(color, Piece::Rook) & rook_mask;

                        valid_rooks.try_next_square().map(Square::file)?
                    }
                    _ => return None,
                };

                board.set_castle_rights(color, rook_file > our_king.file(), Some(rook_file));
            }
        }

        board.calc_hashes();
        board.calc_attacks();
        board.calc_pinned();

        if en_passant != "-" {
            let ep_sq = en_passant.parse::<Square>().ok()?;
            if ep_sq.rank() != Rank::Sixth.relative_to(board.stm) {
                return None;
            }

            board.calc_ep(Some(ep_sq));
        }

        board.halfmove_clock = halfmove_clock.parse::<u8>().ok()?.min(100);
        board.fullmove_count = fullmove_count.parse::<u16>().ok()?.max(1);

        Some(board)
    }

    #[inline]
    pub fn to_fen(&self, chess960: bool) -> String {
        let mut fen = String::new();

        for &rank in Rank::ALL.iter().rev() {
            let mut empty = 0;

            for &file in File::ALL.iter() {
                let sq = Square::new(file, rank);

                if let Some(piece) = self.piece_on(sq) {
                    if empty > 0 {
                        write!(fen, "{}", empty).unwrap();
                        empty = 0;
                    }

                    let mut piece: char = piece.into();
                    if self.color_on(sq).unwrap() == Color::White {
                        piece = piece.to_ascii_uppercase();
                    }

                    write!(fen, "{}", piece).unwrap();
                } else {
                    empty += 1;
                }
            }

            if empty > 0 {
                write!(fen, "{}", empty).unwrap();
            }

            if rank > Rank::First {
                write!(fen, "/").unwrap();
            }
        }

        write!(fen, " {}", char::from(self.stm)).unwrap();

        let mut castle_rights = String::new();
        if let Some(file) = self.castle_rights[Color::White].short {
            castle_rights.push(if chess960 {
                char::from(file).to_ascii_uppercase()
            } else {
                'K'
            });
        }
        if let Some(file) = self.castle_rights[Color::White].long {
            castle_rights.push(if chess960 {
                char::from(file).to_ascii_uppercase()
            } else {
                'Q'
            });
        }
        if let Some(file) = self.castle_rights[Color::Black].short {
            castle_rights.push(if chess960 {
                char::from(file).to_ascii_lowercase()
            } else {
                'k'
            });
        }
        if let Some(file) = self.castle_rights[Color::Black].long {
            castle_rights.push(if chess960 {
                char::from(file).to_ascii_lowercase()
            } else {
                'q'
            });
        }

        if castle_rights.is_empty() {
            castle_rights.push('-');
        }

        write!(fen, " {}", castle_rights).unwrap();

        if let Some(sq) = self.ep_square() {
            write!(fen, " {}", sq).unwrap();
        } else {
            write!(fen, " -").unwrap();
        }

        write!(fen, " {} {}", self.halfmove_clock, self.fullmove_count).unwrap();

        fen
    }
}
