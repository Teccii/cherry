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
            println!("castle_rights");

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

        if en_passant != "-" {
            let ep_dest = en_passant.parse::<Square>().ok()?;
            if ep_dest.rank() != Rank::Sixth.relative_to(board.stm) {
                return None;
            }
            let ep_file = ep_dest.file();
            let ep_victim = Square::new(ep_file, Rank::Fifth.relative_to(board.stm));

            let our_pawns = board.index_to_piece[board.stm].mask_eq(Piece::Pawn);
            let our_attacks = board.attack_table(board.stm).get(ep_dest);
            let mut left = false;
            let mut right = false;

            let king = board.king(board.stm);
            let (ray_coords, ray_valid) = ray_perm(king);

            for index in our_pawns & our_attacks {
                let ep_src = board.index_to_square[board.stm][index].unwrap();
                let pawn_place = board.inner.get(ep_src);
                let mut ep_board = board.inner.clone();
                ep_board.set(ep_src, Place::EMPTY);
                ep_board.set(ep_victim, Place::EMPTY);
                ep_board.set(ep_dest, pawn_place);

                let ray_places = ep_board.permute(ray_coords);
                let their_color = match board.stm {
                    Color::White => ray_places.msb(),
                    Color::Black => !ray_places.msb(),
                };
                let blockers = ray_places.nonzero().to_bitmask();
                let attackers = ray_attackers(ray_places);
                let closest = extend_bitrays(blockers, ray_valid) & blockers;

                let their_attackers = their_color & attackers & closest;
                if their_attackers.to_bitmask() != 0 {
                    if ep_src.file() < ep_file {
                        left = true;
                    } else {
                        right = true;
                    }
                }
            }

            let ep = if left || right {
                Some(EnPassant::new(ep_file, left, right))
            } else {
                None
            };

            board.set_en_passant(ep);
        }

        board.halfmove_clock = halfmove_clock.parse::<u8>().ok()?.min(100);
        board.fullmove_count = fullmove_count.parse::<u16>().ok()?.max(1);
        board.calc_hashes();
        board.calc_attacks();

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
            castle_rights.push(if chess960 { file.into() } else { 'K' });
        }
        if let Some(file) = self.castle_rights[Color::White].long {
            castle_rights.push(if chess960 { file.into() } else { 'Q' });
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