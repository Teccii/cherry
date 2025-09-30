use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Position {
    board: Board,
    board_history: Vec<Board>,
    nnue: Nnue,
}

impl Position {
    #[inline]
    pub fn new(board: Board, weights: &NetworkWeights) -> Position {
        let nnue = Nnue::new(&board, weights);

        Position {
            board,
            board_history: Vec::new(),
            nnue,
        }
    }

    #[inline]
    pub fn set_board(&mut self, board: Board, weights: &NetworkWeights) {
        self.nnue.full_reset(&board, weights);
        self.board_history.clear();
        self.board = board;
    }

    #[inline]
    pub fn reset(&mut self, weights: &NetworkWeights) {
        self.nnue.full_reset(&self.board, weights);
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
    pub fn make_move(&mut self, mv: Move, weights: &NetworkWeights) {
        self.board_history.push(self.board.clone());
        self.board.make_move(mv);

        self.nnue.make_move(self.board_history.last().unwrap(), &self.board, weights, mv);
    }

    #[inline]
    pub fn null_move(&mut self) -> bool {
        self.board_history.push(self.board.clone());
        if self.board.null_move() {
            return true;
        }

        self.board_history.pop().unwrap();
        false
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.nnue.unmake_move();
    }
    
    #[inline]
    pub fn unmake_null_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
    }

    /*----------------------------------------------------------------*/
    
    #[inline]
    pub fn eval(&mut self, weights: &NetworkWeights) -> Score {
        self.nnue.apply_updates(&self.board, weights);

        let bucket = OUTPUT_BUCKETS[self.board.occupied().popcnt()];
        let mut eval = self.nnue.eval(weights, bucket, self.stm());
        
        let material = W::pawn_mat_scale() * self.board.pieces(Piece::Pawn).popcnt() as i32
            + W::knight_mat_scale() * self.board.pieces(Piece::Knight).popcnt() as i32
            + W::bishop_mat_scale() * self.board.pieces(Piece::Bishop).popcnt() as i32
            + W::rook_mat_scale() * self.board.pieces(Piece::Rook).popcnt() as i32
            + W::queen_mat_scale() * self.board.pieces(Piece::Queen).popcnt() as i32;
        eval = (i32::from(eval) * (W::mat_scale_base() + material) / 32768) as i16;

        Score::new(eval.clamp(-Score::MIN_TB_WIN.0 + 1, Score::MIN_TB_WIN.0 - 1))
    }

    /*----------------------------------------------------------------*/
    
    #[inline]
    pub fn is_draw(&self) -> bool {
        self.insufficient_material() || self.repetition() || self.board.status() == BoardStatus::Draw
    }
    
    pub fn insufficient_material(&self) -> bool {
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

    pub fn repetition(&self) -> bool {
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