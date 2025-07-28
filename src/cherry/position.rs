use cherry_chess::*;
use super::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Position {
    board: Board,
    board_history: Vec<Board>,
    move_history: Vec<Option<Move>>,
    #[cfg(not(feature = "nnue"))] evaluator: Evaluator,
    #[cfg(feature = "nnue")] nnue: Nnue,
}

impl Position {
    #[inline]
    #[cfg(not(feature = "nnue"))]
    pub fn new(board: Board) -> Position {
        Position {
            board,
            board_history: Vec::new(),
            move_history: Vec::new(),
            evaluator: Evaluator::default(),
        }
    }

    #[inline]
    #[cfg(feature = "nnue")]
    pub fn new(board: Board, weights: &NetworkWeights) -> Position {
        Position {
            board,
            board_history: Vec::new(),
            move_history: Vec::new(),
            nnue: Nnue::new(&board, weights),
        }
    }
    
    #[inline]
    #[cfg(not(feature = "nnue"))]
    pub fn set_board(&mut self, board: Board) {
        self.board = board;
        self.board_history.clear();
        self.move_history.clear();
    }

    #[inline]
    #[cfg(feature = "nnue")]
    pub fn set_board(&mut self, board: Board, weights: &NetworkWeights) {
        self.board = board;
        self.nnue.reset(&board, weights);
        self.board_history.clear();
        self.move_history.clear();
    }

    #[inline]
    #[cfg(feature = "nnue")]
    pub fn reset(&mut self, weights: &NetworkWeights) {
        self.nnue.reset(&self.board, weights);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn board(&self) -> &Board {
        &self.board
    }
    
    #[inline]
    pub fn non_pawn_material(&self) -> bool {
        let pieces = self.board.colors(self.stm());

        pieces != pieces & (self.board.pieces(Piece::Pawn) | self.board.pieces(Piece::King))
    }

    #[inline]
    pub fn can_castle(&self) -> bool {
        for &color in &Color::ALL {
            let rights = self.board.castle_rights(color);

            if rights.short.is_some() || rights.long.is_some() {
                return true;
            }
        }

        false
    }
    
    #[inline]
    pub fn stm(&self) -> Color {
        self.board.stm()
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.board.hash()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    #[cfg(not(feature = "nnue"))]
    pub fn make_move(&mut self, mv: Move) {
        self.board_history.push(self.board.clone());
        self.move_history.push(Some(mv));
        self.board.make_move(mv);
    }

    #[inline]
    #[cfg(feature = "nnue")]
    pub fn make_move(&mut self, mv: Move) {
        self.board_history.push(self.board.clone());
        self.move_history.push(Some(mv));
        self.nnue.make_move(&self.board, mv);
        self.board.make_move(mv);
    }


    #[inline]
    pub fn null_move(&mut self) -> bool {
        if let Some(new_board) = self.board.null_move() {
            self.board_history.push(self.board.clone());
            self.move_history.push(None);
            self.board = new_board;

            return true;
        }

        false
    }

    #[inline]
    #[cfg(not(feature = "nnue"))]
    pub fn unmake_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.move_history.pop();
    }

    #[inline]
    #[cfg(feature = "nnue")]
    pub fn unmake_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.nnue.unmake_move();
        self.move_history.pop();
    }
    
    #[inline]
    pub fn unmake_null_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.move_history.pop();
    }

    /*----------------------------------------------------------------*/

    #[inline]
    #[cfg(not(feature = "nnue"))]
    pub fn eval(&self) -> Score {
        self.evaluator.eval(&self.board)
    }

    #[inline]
    #[cfg(feature = "nnue")]
    pub fn eval(&mut self, weights: &NetworkWeights) -> Score {
        self.nnue.apply_updates(weights);

        Score::new(self.nnue.eval(weights, self.stm()))
    }
    
    /*----------------------------------------------------------------*/
    
    #[inline]
    pub fn in_check(&self) -> bool {
        self.board.in_check()
    }
    
    #[inline]
    pub fn is_draw(&self) -> bool {
        self.board.status() == BoardStatus::Draw
            || self.insufficient_material()
            || self.repetition()
    }

    /*----------------------------------------------------------------*/
    
    fn insufficient_material(&self) -> bool {
        match self.board.occupied().popcnt() {
            2 => true,
            3 => (self.board.pieces(Piece::Knight) | self.board.pieces(Piece::Bishop)).popcnt() > 0,
            4 => {
                let bishops = self.board.pieces(Piece::Bishop);
                
                if bishops.popcnt() != 2 || self.board.colors(Color::White).popcnt() != 2 {
                    return false;
                }

                bishops.is_subset(Bitboard::DARK_SQUARES) || bishops.is_subset(Bitboard::LIGHT_SQUARES)
            },
            _ => false
        }
    }

    fn repetition(&self) -> bool {
        let hash = self.hash();
        let hm = self.board.halfmove_clock() as usize;

        if hm < 4 {
            return false;
        }

        self.board_history.iter()
            .rev()
            .take(hm + 1) //idk if hm or hm + 1
            .skip(3)
            .step_by(2)
            .any(|b| b.hash() == hash)
    }
}