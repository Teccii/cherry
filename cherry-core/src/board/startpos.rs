use crate::*;

impl Board {
    #[inline]
    pub fn startpos() -> Board {
        Board::frc_startpos(518)
    }

    #[inline]
    pub fn frc_startpos(scharnagl: u16) -> Board {
        Board::dfrc_startpos(scharnagl, scharnagl)
    }

    #[inline]
    pub fn dfrc_startpos(white_scharnagl: u16, black_scharnagl: u16) -> Board {
        assert!(white_scharnagl < 960);
        assert!(black_scharnagl < 960);

        #[inline]
        fn write_scharnagl(board: &mut Board, color: Color, scharnagl: u16) {
            let n = scharnagl;
            let (n, light_bishop) = (n / 4, n % 4);
            let (n, dark_bishop) = (n / 4, n % 4);
            let (n, queen) = (n / 6, n % 6);
            let knights = n;

            let backrank = Rank::First.relative_to(color);
            let mut free_squares = backrank.bitboard();

            let light_bishop = match light_bishop {
                0 => File::B,
                1 => File::D,
                2 => File::F,
                3 => File::H,
                _ => unreachable!(),
            };
            let dark_bishop = match dark_bishop {
                0 => File::A,
                1 => File::C,
                2 => File::E,
                3 => File::G,
                _ => unreachable!(),
            };

            let light_bishop = Square::new(light_bishop, backrank);
            let dark_bishop = Square::new(dark_bishop, backrank);

            free_squares ^= light_bishop;
            free_squares ^= dark_bishop;

            let queen = free_squares.iter().nth(queen as usize).unwrap();
            free_squares ^= queen;

            let (left_knight, right_knight) = match knights {
                0 => (0, 1),
                1 => (0, 2),
                2 => (0, 3),
                3 => (0, 4),

                4 => (1, 2),
                5 => (1, 3),
                6 => (1, 4),

                7 => (2, 3),
                8 => (2, 4),

                9 => (3, 4),

                _ => unreachable!(),
            };

            let left_knight = free_squares.iter().nth(left_knight).unwrap();
            let right_knight = free_squares.iter().nth(right_knight).unwrap();

            free_squares ^= left_knight;
            free_squares ^= right_knight;

            let left_rook = free_squares.next_square();
            free_squares ^= left_rook;

            let king = free_squares.next_square();
            free_squares ^= king;

            let right_rook = free_squares.next_square();
            free_squares ^= right_rook;

            let pieces: [(Square, Piece, PieceIndex); File::COUNT] = [
                (king, Piece::King, PieceIndex(0)),
                (left_knight, Piece::Knight, PieceIndex(1)),
                (right_knight, Piece::Knight, PieceIndex(2)),
                (light_bishop, Piece::Bishop, PieceIndex(3)),
                (dark_bishop, Piece::Bishop, PieceIndex(4)),
                (left_rook, Piece::Rook, PieceIndex(5)),
                (right_rook, Piece::Rook, PieceIndex(6)),
                (queen, Piece::Queen, PieceIndex(7)),
            ];

            for &(sq, piece, index) in &pieces {
                board.set(sq, Place::from_piece(piece, color, index));
                board.index_to_square[color][index] = Some(sq);
                board.index_to_piece[color][index] = Some(piece);
            }

            let mut index = 8;
            for sq in Rank::Second.relative_to(color).bitboard() {
                let piece_index = PieceIndex(index);

                board.set(sq, Place::from_piece(Piece::Pawn, color, piece_index));
                board.index_to_square[color][piece_index] = Some(sq);
                board.index_to_piece[color][piece_index] = Some(Piece::Pawn);

                index += 1;
            }

            board.set_castle_rights(color, true, Some(right_rook.file()));
            board.set_castle_rights(color, false, Some(left_rook.file()));
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

        write_scharnagl(&mut board, Color::White, white_scharnagl);
        write_scharnagl(&mut board, Color::Black, black_scharnagl);
        board.calc_hashes();
        board.calc_attacks();
        board
    }
}