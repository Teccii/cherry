use cozy_chess::*;
use super::*;

#[derive(Debug, Clone)]
pub struct Position {
    board: Board,
    pawn_zobrist: PawnZobrist,
    board_history: Vec<Board>,
    move_history: Vec<Option<Move>>,
    evaluator: Evaluator,
}

impl Position {
    #[inline(always)]
    pub fn new(board: Board) -> Position {
        let w_pawns = board.colored_pieces(Color::White, Piece::Pawn);
        let b_pawns = board.colored_pieces(Color::Black, Piece::Pawn);
        
        Position {
            board,
            pawn_zobrist: PawnZobrist::new(w_pawns, b_pawns),
            board_history: Vec::new(),
            move_history: Vec::new(),
            evaluator: Evaluator::default(),
        }
    }
    
    #[inline(always)]
    pub fn reset(&mut self, board: Board) {
        let w_pawns = board.colored_pieces(Color::White, Piece::Pawn);
        let b_pawns = board.colored_pieces(Color::Black, Piece::Pawn);
        
        self.board = board;
        self.pawn_zobrist.reset(w_pawns, b_pawns);
        self.board_history.clear();
        self.move_history.clear();
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
        self.move_history.push(Some(mv));
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
            self.move_history.push(None);
            self.pawn_zobrist.null_move();
            self.board = new_board;

            return true;
        }

        false
    }

    #[inline(always)]
    pub fn unmake_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.pawn_zobrist.unmake_move();
        self.move_history.pop();
    }
    
    #[inline(always)]
    pub fn unmake_null_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.pawn_zobrist.unmake_move();
        
        debug_assert!(self.move_history.pop().unwrap().is_none());
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn eval(&mut self, ply: u16) -> Score {
        if self.is_checkmate() {
            return Score::new_mated(ply);
        }

        if self.is_draw(ply) {
            return Score::ZERO;
        }
        
        self.evaluator.eval(&self.board().clone(), ply)
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
    pub fn is_draw(&self, ply: u16) -> bool {
        self.board.status() == GameStatus::Drawn
        || self.insufficient_material()
        || self.repetition(ply)
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
    
    fn repetition(&self, ply: u16) -> bool {
        let hash = self.hash();
        
        let twofold = self.board_history.iter()
            .rev()
            .take(ply as usize)
            .any(|b| b.hash() == hash);
        
        let threefold = self.board_history.iter()
            .rev()
            .skip(1)
            .step_by(2)
            .filter(|b| b.hash() == hash)
            .count() >= 2;
        
        twofold || threefold
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