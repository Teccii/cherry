use cherry_core::*;
use super::*;

#[derive(Debug, Clone)]
pub struct Position {
    board: Board,
    board_history: Vec<Board>,
    move_history: Vec<Option<Move>>,
    evaluator: Evaluator,
}

impl Position {
    #[inline]
    pub fn new(board: Board) -> Position {
        Position {
            board,
            board_history: Vec::new(),
            move_history: Vec::new(),
            evaluator: Evaluator::default(),
        }
    }
    
    #[inline]
    pub fn reset(&mut self, board: Board) {
        self.board = board;
        self.board_history.clear();
        self.move_history.clear();
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
    pub fn make_move(&mut self, mv: Move) {
        self.board_history.push(self.board.clone());
        self.move_history.push(Some(mv));
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
    pub fn unmake_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.move_history.pop();
    }
    
    #[inline]
    pub fn unmake_null_move(&mut self) {
        self.board = self.board_history.pop().unwrap();
        self.move_history.pop();
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eval(&self) -> Score {
        self.evaluator.eval(&self.board)
    }
    
    /*----------------------------------------------------------------*/
    
    #[inline]
    pub fn in_check(&self) -> bool {
        self.board.in_check()
    }
    
    #[inline]
    pub fn is_draw(&self, ply: u16) -> bool {
        self.board.status() == BoardStatus::Draw
            || self.insufficient_material()
            || self.repetition(ply)
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
    
    fn repetition(&self, ply: u16) -> bool {
        let hash = self.hash();
        let ply = ply as usize - 1;

        let two_fold = self.board_history.iter()
            .rev()
            .take(ply)
            .any(|b| b.hash() == hash);

        let three_fold = || self.board_history.iter()
            .rev()
            .take(self.board.halfmove_clock() as usize)
            .skip(ply)
            .filter(|b| b.hash() == hash)
            .count() >= 2;

        two_fold || three_fold()
    }
}