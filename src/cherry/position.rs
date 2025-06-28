use cozy_chess::*;
use super::*;

#[derive(Debug, Clone)]
pub struct Position {
    board: Board,
    repetition: u8,
    pawn_zobrist: PawnZobrist,
    board_history: Vec<Board>,
    move_history: Vec<Option<Move>>,
    repetition_history: Vec<u8>,
    evaluator: Evaluator,
}

impl Position {
    #[inline(always)]
    pub fn new(board: Board) -> Position {
        let w_pawns = board.colored_pieces(Color::White, Piece::Pawn);
        let b_pawns = board.colored_pieces(Color::Black, Piece::Pawn);
        let repetition = board.halfmove_clock();

        Position {
            board,
            repetition,
            pawn_zobrist: PawnZobrist::new(w_pawns, b_pawns),
            board_history: Vec::new(),
            move_history: Vec::new(),
            repetition_history: Vec::new(),
            evaluator: Evaluator::default(),
        }
    }
    
    #[inline(always)]
    pub fn reset(&mut self, board: Board) {
        let w_pawns = board.colored_pieces(Color::White, Piece::Pawn);
        let b_pawns = board.colored_pieces(Color::Black, Piece::Pawn);

        self.board = board;
        self.repetition = self.board.halfmove_clock();
        self.pawn_zobrist.reset(w_pawns, b_pawns);
        self.board_history.clear();
        self.move_history.clear();
        self.repetition_history.clear();
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn board(&self) -> &Board { &self.board }
    
    #[inline(always)]
    pub fn stm(&self) -> Color { self.board.side_to_move() }

    #[inline(always)]
    pub fn hash(&self) -> u64 { self.board.hash() }

    #[inline(always)]
    pub fn pawn_hash(&self) -> u64 { self.pawn_zobrist.hash() }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn make_move(&mut self, mv: Move) {
        let w_pawns = self.board.colored_pieces(Color::White, Piece::Pawn);
        let b_pawns = self.board.colored_pieces(Color::Black, Piece::Pawn);
        
        self.board_history.push(self.board.clone());
        self.repetition_history.push(self.repetition);
        self.move_history.push(Some(mv));
        self.update_repetition(mv);

        self.board.play_unchecked(mv);
        self.pawn_zobrist.make_move(
            w_pawns ^ self.board.colored_pieces(Color::White, Piece::Pawn),
            b_pawns ^ self.board.colored_pieces(Color::Black, Piece::Pawn),
        );
    }

    #[inline(always)]
    pub fn null_move(&mut self) -> bool {
        if let Some(new_board) = self.board.null_move() {
            self.board_history.push(self.board.clone());
            self.repetition_history.push(self.repetition);
            self.move_history.push(None);
            self.pawn_zobrist.null_move();
            self.board = new_board;
            self.repetition = 0;

            return true;
        }

        false
    }

    #[inline(always)]
    pub fn unmake_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.repetition = self.repetition_history.pop().unwrap();
        self.pawn_zobrist.unmake_move();
        self.move_history.pop();
    }
    
    #[inline(always)]
    pub fn unmake_null_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.repetition = self.repetition_history.pop().unwrap();
        self.pawn_zobrist.unmake_move();
        
        debug_assert!(self.move_history.pop().unwrap().is_none());
    }

    #[inline(always)]
    fn update_repetition(&mut self, mv: Move) {
        let piece = self.board.piece_on(mv.from).unwrap();
        let victim = self.board.capture_piece(mv);

        if piece == Piece::Pawn || victim.is_some() {
            self.repetition = 0;
        } else {
            self.repetition += 1;
        }

        if self.board.is_castles(mv) {
            self.repetition = 0;
        } else {
            if victim.is_some() && mv.to.rank() == Rank::Eighth.relative_to(self.stm()) {
                let rights = self.board.castle_rights(self.stm());
                let file = mv.to.file();

                if rights.short == Some(file) || rights.long == Some(file) {
                    self.repetition = 0;
                }
            }

            match piece {
                Piece::Rook => {
                    let rights = self.board.castle_rights(self.stm());
                    let file = mv.from.file();

                    if rights.short == Some(file) || rights.long == Some(file) {
                        self.repetition = 0;
                    }
                },
                Piece::King => {
                    let rights = self.board.castle_rights(self.stm());

                    if rights.short.is_some() || rights.long.is_some() {
                        self.repetition = 0;
                    }
                }
                _ => ()
            }
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn eval(&mut self) -> Score {
        self.evaluator.eval(&self.board().clone())
    }
    
    #[inline(always)]
    pub fn evaluator(&self) -> &Evaluator { &self.evaluator }
    
    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn is_checkmate(&self) -> bool {
        self.board.status() == GameStatus::Won
    }
    
    #[inline(always)]
    pub fn in_check(&self) -> bool {
        self.board.in_check()
    }
    
    #[inline(always)]
    pub fn is_draw(&self) -> bool {
        self.board.status() == GameStatus::Drawn
        || self.insufficient_material()
        || self.repetition()
    }

    /*----------------------------------------------------------------*/
    
    fn insufficient_material(&self) -> bool {
        match self.board.occupied().len() {
            2 => true,
            3 => (self.board.pieces(Piece::Knight) | self.board.pieces(Piece::Bishop)).len() > 0,
            4 => {
                let bishops = self.board.pieces(Piece::Bishop);
                
                if bishops.len() != 2 || self.board.colors(Color::White).len() != 2 {
                    return false;
                }

                bishops.is_subset(BitBoard::DARK_SQUARES) || bishops.is_subset(BitBoard::LIGHT_SQUARES)
            },
            _ => false
        }
    }
    
    fn repetition(&self) -> bool {
        let hash = self.hash();

        self.board_history.iter()
            .rev()
            .take(self.repetition as usize)
            .skip(1)
            .step_by(2)
            .filter(|b| b.hash() == hash)
            .count() >= 2
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct PawnZobrist {
    hash: u64,
    history: Vec<u64>,
}

impl PawnZobrist {
    pub fn new(w_pawns: BitBoard, b_pawns: BitBoard) -> PawnZobrist {
        let mut hash = 0;

        for sq in w_pawns {
            hash ^= PAWN_ZOBRIST[0][sq as usize];
        }

        for sq in b_pawns {
            hash ^= PAWN_ZOBRIST[1][sq as usize];
        }

        PawnZobrist {
            hash,
            history: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn reset(&mut self, w_pawns: BitBoard, b_pawns: BitBoard) {
        self.hash = 0;
        self.history.clear();

        for sq in w_pawns {
            self.hash ^= PAWN_ZOBRIST[0][sq as usize];
        }

        for sq in b_pawns {
            self.hash ^= PAWN_ZOBRIST[1][sq as usize];
        }
    }

    #[inline(always)]
    pub fn make_move(&mut self, w_diff: BitBoard, b_diff: BitBoard) {
        self.history.push(self.hash);

        for sq in w_diff {
            self.hash ^= PAWN_ZOBRIST[0][sq as usize];
        }

        for sq in b_diff {
            self.hash ^= PAWN_ZOBRIST[1][sq as usize];
        }
    }

    #[inline(always)]
    pub fn null_move(&mut self) {
        self.history.push(self.hash);
    }

    #[inline(always)]
    pub fn unmake_move(&mut self) {
        self.hash = self.history.pop().unwrap();
    }

    #[inline(always)]
    pub fn hash(&self) -> u64 { self.hash }
}

const PAWN_ZOBRIST: [[u64; Square::NUM]; Color::NUM] = {
    let mut prng = Xorshift64::new(0xB00B5);
    let mut table = [[0; Square::NUM]; Color::NUM];
    let mut i = 0;

    while i < Color::NUM {
        let mut j = 0;
        while j < Color::NUM {
            table[i][j] = prng.next();
            j += 1;
        }

        i += 1;
    }

    table
};