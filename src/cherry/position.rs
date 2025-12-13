use crate::*;

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct Position {
    current: Board,
    boards: Vec<Board>,
    moves: Vec<Option<MoveData>>,
    nnue: Nnue,
}

impl Position {
    #[inline]
    pub fn new(board: Board, weights: &NetworkWeights) -> Position {
        let nnue = Nnue::new(&board, weights);

        Position {
            current: board,
            boards: Vec::with_capacity(MAX_PLY as usize),
            moves: Vec::with_capacity(MAX_PLY as usize),
            nnue,
        }
    }

    #[inline]
    pub fn set_board(&mut self, board: Board, weights: &NetworkWeights) {
        self.nnue.full_reset(&board, weights);
        self.boards.clear();
        self.moves.clear();
        self.current = board;
    }

    #[inline]
    pub fn reset(&mut self, weights: &NetworkWeights) {
        self.nnue.full_reset(&self.current, weights);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn board(&self) -> &Board {
        &self.current
    }

    #[inline]
    pub fn prev_move(&self, ply: usize) -> Option<MoveData> {
        if self.moves.is_empty() {
            return None;
        }

        let ply = ply as isize - 1; //countermove is last
        let last_index = self.moves.len() as isize - 1;
        let index = last_index - ply;

        (index >= 0)
            .then(|| self.moves.get(index as usize).and_then(|&m| m))
            .flatten()
    }

    #[inline]
    pub fn stm(&self) -> Color {
        self.current.stm()
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.current.hash()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn make_move(&mut self, mv: Move, weights: &NetworkWeights) {
        self.boards.push(self.current.clone());
        self.moves.push(Some(MoveData::new(&self.current, mv)));
        self.current.make_move(mv);

        let prev_board = self.boards.last().unwrap();
        self.nnue.make_move(prev_board, &self.current, weights, mv);
    }

    #[inline]
    pub fn null_move(&mut self) -> bool {
        self.boards.push(self.current.clone());
        self.moves.push(None);

        if self.current.null_move() {
            return true;
        }

        self.boards.pop().unwrap();
        self.moves.pop().unwrap();
        false
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.current = self.boards.pop().unwrap();
        self.moves.pop().unwrap();
        self.nnue.unmake_move();
    }

    #[inline]
    pub fn unmake_null_move(&mut self) {
        self.current = self.boards.pop().unwrap();
        self.moves.pop().unwrap();
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eval(&mut self, weights: &NetworkWeights) -> Score {
        self.nnue.apply_updates(&self.current, weights);

        let bucket = OUTPUT_BUCKETS[self.current.occupied().popcnt()];
        let mut eval = self.nnue.eval(weights, bucket, self.stm());

        let material = W::pawn_mat_scale() * self.current.pieces(Piece::Pawn).popcnt() as i32
            + W::knight_mat_scale() * self.current.pieces(Piece::Knight).popcnt() as i32
            + W::bishop_mat_scale() * self.current.pieces(Piece::Bishop).popcnt() as i32
            + W::rook_mat_scale() * self.current.pieces(Piece::Rook).popcnt() as i32
            + W::queen_mat_scale() * self.current.pieces(Piece::Queen).popcnt() as i32;
        eval = (i32::from(eval) * (W::mat_scale_base() + material) / 32768) as i16;

        Score::new(eval.clamp(-Score::MIN_TB_WIN.0 + 1, Score::MIN_TB_WIN.0 - 1))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn is_draw(&self) -> bool {
        self.insufficient_material()
            || self.repetition()
            || self.current.status() == BoardStatus::Draw
    }

    pub fn insufficient_material(&self) -> bool {
        match self.current.occupied().popcnt() {
            2 => true,
            3 =>
                (self.current.pieces(Piece::Knight) | self.current.pieces(Piece::Bishop)).popcnt()
                    > 0,
            4 => {
                let bishops = self.current.pieces(Piece::Bishop);

                if bishops.popcnt() != 2 || self.current.colors(Color::White).popcnt() != 2 {
                    return false;
                }

                let dark_bishops = bishops.is_subset(Bitboard::DARK_SQUARES);
                let light_bishops = bishops.is_subset(Bitboard::LIGHT_SQUARES);

                dark_bishops || light_bishops
            }
            _ => false,
        }
    }

    pub fn repetition(&self) -> bool {
        let hash = self.hash();
        let hm = self.current.halfmove_clock() as usize;

        if hm < 4 {
            return false;
        }

        self.boards
            .iter()
            .rev()
            .take(hm + 1) //idk if hm or hm + 1
            .skip(3)
            .step_by(2)
            .any(|b| b.hash() == hash)
    }
}
