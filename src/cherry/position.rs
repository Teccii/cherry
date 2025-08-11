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
        Position {
            board,
            board_history: Vec::new(),
            nnue: Nnue::new(&board, weights),
        }
    }

    #[inline]
    pub fn set_board(&mut self, board: Board, weights: &NetworkWeights) {
        self.board = board;
        self.nnue.full_reset(&board, weights);
        self.board_history.clear();
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
        if let Some(new_board) = self.board.null_move() {
            self.board_history.push(self.board.clone());
            self.board = new_board;

            return true;
        }

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

        let mut eval = self.nnue.eval(weights, self.stm());
        let material = W::pawn_mat_scale() * self.board.pieces(Piece::Pawn).popcnt() as i32
            + W::knight_mat_scale() * self.board.pieces(Piece::Knight).popcnt() as i32
            + W::bishop_mat_scale() * self.board.pieces(Piece::Bishop).popcnt() as i32
            + W::rook_mat_scale() * self.board.pieces(Piece::Rook).popcnt() as i32
            + W::queen_mat_scale() * self.board.pieces(Piece::Queen).popcnt() as i32;
        eval = (i32::from(eval) * (W::mat_scale_base() + material) / W::mat_scale_max()) as i16;

        Score::new(eval.clamp(-Score::MIN_TB_WIN.0 + 1, Score::MIN_TB_WIN.0 - 1))
    }

    /*----------------------------------------------------------------*/

    /*
    Adapted from Viridithas and Ethereal:
    https://github.com/cosmobobak/viridithas/blob/master/src/search.rs#L1734
    https://github.com/AndyGrant/Ethereal/blob/master/src/search.c#L929
    */
    pub fn cmp_see(&self, mv: Move, threshold: i16) -> bool {
        let (from, to, flag, promotion) = (mv.from(), mv.to(), mv.flag(), mv.promotion());
        let board = &self.board;

        let mut next_victim = promotion.unwrap_or_else(|| board.piece_on(from).unwrap());
        let mut balance = -threshold + match flag {
            MoveFlag::None => board.piece_on(to).map_or(0, |p| see_value(p)),
            MoveFlag::EnPassant => see_value(Piece::Pawn),
            MoveFlag::Promotion => see_value(promotion.unwrap()),
            MoveFlag::Castling => 0,
        };

        //best case fail
        if balance < 0 {
            return false;
        }

        balance -= see_value(next_victim);
        //worst case pass
        if balance >= 0 {
            return true;
        }

        let mut occupied = board.occupied() ^ from | to;
        if flag == MoveFlag::EnPassant {
            occupied ^= board.ep_square().map_or(Bitboard::EMPTY, |sq| sq.bitboard());
        }

        let (diag, orth) = (board.diag_sliders(), board.orth_sliders());
        let (w_pinned, b_pinned) = (
            board.pinned() & board.colors(Color::White),
            board.pinned() & board.colors(Color::Black),
        );
        let (w_checks, b_checks) = (
            queen_rays(board.king(Color::White)),
            queen_rays(board.king(Color::Black))
        );
        let allowed_pieces = !(w_pinned | b_pinned)
            | (w_pinned & w_checks)
            | (b_pinned & b_checks);

        let mut attackers = board.attacks(to, occupied) & allowed_pieces;
        let mut color = !board.stm();

        'see: loop {
            let stm_attackers = attackers & board.colors(color);

            if stm_attackers.is_empty() {
                break 'see;
            }

            //find LVA
            for &piece in &Piece::ALL {
                next_victim = piece;
                if !(stm_attackers & board.pieces(next_victim)).is_empty() {
                    break;
                }
            }

            occupied ^= (stm_attackers & board.pieces(next_victim)).next_square();

            if matches!(next_victim, Piece::Pawn | Piece::Bishop | Piece::Queen) {
                attackers |= bishop_moves(to, occupied) & diag;
            }

            if matches!(next_victim, Piece::Rook | Piece::Queen) {
                attackers |= rook_moves(to, occupied) & orth;
            }

            attackers &= occupied;
            color = !color;

            balance = -balance - 1 - see_value(next_victim);
            if balance >= 0 {
                if next_victim == Piece::King && !(attackers & board.colors(color)).is_empty() {
                    color = !color;
                }

                break;
            }
        }

        board.stm() != color
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

#[cfg(test)]
mod tests {
    #[test]
    fn see() {
        use crate::*;
        let fens = &[
            "8/4k3/8/3n4/8/8/3R4/3K4 w - - 0 1",
            "8/4k3/1n6/3n4/8/8/3R4/3K4 w - - 0 1",
            "8/3r4/3q4/3r4/8/3Q3K/3R4/7k w - - 0 1",
            "8/8/b7/1q6/2b5/3Q3K/4B3/7k w - - 0 1",
            "8/1pp2k2/3p4/8/8/3Q1K2/8/8 w - - 0 1",
        ];
        let expected = &[
            see_value(Piece::Knight),
            see_value(Piece::Knight) - see_value(Piece::Rook),
            0,
            0,
            see_value(Piece::Pawn) - see_value(Piece::Queen),
        ];

        let moves = &[
            Move::new(Square::D2, Square::D5, MoveFlag::None),
            Move::new(Square::D2, Square::D5, MoveFlag::None),
            Move::new(Square::D3, Square::D5, MoveFlag::None),
            Move::new(Square::D3, Square::C4, MoveFlag::None),
            Move::new(Square::D3, Square::D6, MoveFlag::None),
        ];

        let weights = NetworkWeights::default();
        let mut pos = Position::new(Board::default(), &weights);

        for ((&fen, &expected), &mv) in fens.iter().zip(expected).zip(moves) {
            pos.set_board(Board::from_fen(fen, false).unwrap(), &weights);

            assert!(pos.cmp_see(mv, expected));
            assert!(!pos.cmp_see(mv, expected + 1));
        }
    }
}