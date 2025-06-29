use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BoardBuilderError {
    InvalidBoard,
    InvalidCastleRights,
    InvalidEnPassant,
    InvalidFullMoveCount,
    InvalidHalfMoveClock,
}

#[derive(Debug, Clone)]
pub struct BoardBuilder {
    pieces: [Option<ColorPiece>; Square::COUNT],
    castle_rights: [CastleRights; Color::COUNT],
    en_passant: Option<Square>,
    fullmove_count: u16,
    halfmove_clock: u8,
    stm: Color
}

impl BoardBuilder {
    #[inline(always)]
    pub fn empty() -> BoardBuilder {
        BoardBuilder {
            pieces: [None; Square::COUNT],
            castle_rights: [CastleRights::EMPTY; Color::COUNT],
            en_passant: None,
            fullmove_count: 1,
            halfmove_clock: 0,
            stm: Color::White
        }
    }

    #[inline(always)]
    pub fn from_board(board: &Board) -> BoardBuilder {
        let mut builder = BoardBuilder::empty();

        for &color in &Color::ALL {
            let colors = board.colors(color);

            for &piece in &Piece::ALL {
                let pieces = colors & board.pieces(piece);

                for sq in pieces {
                    builder.set_piece(sq, Some(ColorPiece::new(piece, color)));
                }
            }

            builder.set_castle_rights(color, board.castle_rights(color).short, true);
            builder.set_castle_rights(color, board.castle_rights(color).long, false);
        }

        builder.set_en_passant(board.en_passant()
            .map(|f| Square::new(f, Rank::Sixth.relative_to(board.stm())))
        );

        builder.set_fullmove_count(board.fullmove_count());
        builder.set_halfmove_clock(board.halfmove_clock());
        builder.set_stm(board.stm());

        builder
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn startpos() -> BoardBuilder {
        BoardBuilder::chess960(518)
    }

    #[inline(always)]
    pub fn chess960(scharnagl: u16) -> BoardBuilder {
        BoardBuilder::double_chess960(scharnagl, scharnagl)
    }

    #[inline(always)]
    pub fn double_chess960(white_scharnagl: u16, black_sharnagl: u16) -> BoardBuilder {
        let mut builder = BoardBuilder::empty();
        builder.write_scharnagl(white_scharnagl, Color::White);
        builder.write_scharnagl(black_sharnagl, Color::Black);
        builder
    }

    /*----------------------------------------------------------------*/

    pub fn build(&self) -> Result<Board, BoardBuilderError> {
        let mut board = Board {
            colors: [Bitboard::EMPTY; Color::COUNT],
            pieces: [Bitboard::EMPTY; Piece::COUNT],
            castle_rights: [CastleRights::EMPTY; Color::COUNT],
            en_passant: None,
            checkers: Bitboard::EMPTY,
            pinners: Bitboard::EMPTY,
            pinned: Bitboard::EMPTY,
            fullmove_count: 0,
            halfmove_clock: 0,
            repetition: 0,
            pawn_hash: 0,
            hash: 0,
            stm: Color::White,
        };

        self.add_board(&mut board)?;
        self.add_castle_rights(&mut board)?;
        self.add_en_passant(&mut board)?;
        self.add_fullmove_count(&mut board)?;
        self.add_halfmove_clock(&mut board)?;

        Ok(board)
    }

    fn add_board(&self, board: &mut Board) -> Result<(), BoardBuilderError> {
        for &sq in &Square::ALL {
            if let Some(p) = self.pieces[sq as usize] {
                board.xor_square(p.piece(), p.color(), sq);
            }
        }

        if board.stm() != self.stm {
            board.toggle_stm();
        }

        if !board.board_is_sane() {
            return Err(BoardBuilderError::InvalidBoard);
        }

        let (checkers, pinners, pinned) = board.calc_checks(board.stm());
        board.checkers = checkers;
        board.pinners = pinners;
        board.pinned = pinned;

        if !board.checkers_is_sane() {
            return Err(BoardBuilderError::InvalidBoard);
        }

        Ok(())
    }

    fn add_castle_rights(&self, board: &mut Board) -> Result<(), BoardBuilderError> {
        for &color in &Color::ALL {
            let rights = self.castle_rights[color as usize];
            board.set_castle_rights(color, rights.short, true);
            board.set_castle_rights(color, rights.long, false);
        }

        if !board.castle_rights_is_sane() {
            return Err(BoardBuilderError::InvalidCastleRights);
        }

        Ok(())
    }

    fn add_en_passant(&self, board: &mut Board) -> Result<(), BoardBuilderError> {
        if let Some(sq) = self.en_passant {
            if sq.rank() != Rank::Sixth.relative_to(board.stm()) {
                return Err(BoardBuilderError::InvalidEnPassant);
            }

            board.set_en_passant(Some(sq.file()));
        }

        if !board.en_passant_is_sane() {
            return Err(BoardBuilderError::InvalidEnPassant);
        }

        Ok(())
    }

    fn add_fullmove_count(&self, board: &mut Board) -> Result<(), BoardBuilderError> {
        board.fullmove_count = self.fullmove_count;

        if !board.fullmove_count_is_sane() {
            return Err(BoardBuilderError::InvalidFullMoveCount);
        }

        Ok(())
    }

    fn add_halfmove_clock(&self, board: &mut Board) -> Result<(), BoardBuilderError> {
        board.halfmove_clock = self.halfmove_clock;

        if !board.halfmove_clock_is_sane() {
            return Err(BoardBuilderError::InvalidHalfMoveClock);
        }

        Ok(())
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    fn write_scharnagl(&mut self, scharnagl: u16, color: Color) {
        debug_assert!(scharnagl < 960);

        let n = scharnagl;
        let (n, light_bishop) = (n / 4, n % 4);
        let (n, dark_bishop) = (n / 4, n % 4);
        let (n, queen) = (n / 6, n % 6);
        let knights = n;

        let back_rank = Rank::First.relative_to(color);
        let mut free_squares = back_rank.bitboard();

        let light_bishop = match light_bishop {
            0 => File::B,
            1 => File::D,
            2 => File::F,
            3 => File::H,
            _ => unreachable!()
        };
        let dark_bishop = match dark_bishop {
            0 => File::A,
            1 => File::C,
            2 => File::E,
            3 => File::G,
            _ => unreachable!()
        };

        let light_bishop = Square::new(light_bishop, back_rank);
        let dark_bishop = Square::new(dark_bishop, back_rank);

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

            _ => unreachable!()
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

        self.set_piece(light_bishop, Some(ColorPiece::new(Piece::Bishop, color)));
        self.set_piece(dark_bishop, Some(ColorPiece::new(Piece::Bishop, color)));
        self.set_piece(queen, Some(ColorPiece::new(Piece::Queen, color)));
        self.set_piece(left_knight, Some(ColorPiece::new(Piece::Knight, color)));
        self.set_piece(right_knight, Some(ColorPiece::new(Piece::Knight, color)));
        self.set_piece(left_rook, Some(ColorPiece::new(Piece::Rook, color)));
        self.set_piece(king, Some(ColorPiece::new(Piece::King, color)));
        self.set_piece(right_rook, Some(ColorPiece::new(Piece::Rook, color)));

        for sq in Rank::Second.relative_to(color).bitboard() {
            self.set_piece(sq, Some(ColorPiece::new(Piece::Pawn, color)));
        }

        self.set_castle_rights(color, Some(right_rook.file()), true);
        self.set_castle_rights(color, Some(left_rook.file()), false);
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn set_piece(&mut self, sq: Square, piece: Option<ColorPiece>) {
        self.pieces[sq as usize] = piece;
    }

    #[inline(always)]
    pub fn set_castle_rights(&mut self, color: Color, file: Option<File>, short: bool) {
        let rights = if short {
            &mut self.castle_rights[color as usize].short
        } else {
            &mut self.castle_rights[color as usize].long
        };

        *rights = file;
    }

    #[inline(always)]
    pub fn set_en_passant(&mut self, sq: Option<Square>) {
        self.en_passant = sq;
    }

    #[inline(always)]
    pub fn set_halfmove_clock(&mut self, value: u8) {
        self.halfmove_clock = value.min(100);
    }

    #[inline(always)]
    pub fn set_fullmove_count(&mut self, value: u16) {
        self.fullmove_count = value.max(1);
    }

    #[inline(always)]
    pub fn set_stm(&mut self, color: Color) {
        self.stm = color;
    }
}