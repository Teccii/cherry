use arrayvec::ArrayVec;
use cherry_types::*;
use crate::*;

impl Board {
    //TODO: Handle Promotions
    pub fn see(&self, mv: Move) -> i16 {
        let (from, to) = (mv.from(), mv.to());
        let mut blockers = self.occupied() ^ from.bitboard();

        /*
        En passant only has to be handled for the first capture, because pawn double pushes
        can never capture a piece so they don't matter at all in SEE.
        */
        let first_capture = if self.is_en_passant(mv) {
            blockers ^= Square::new(
                self.en_passant.unwrap(),
                Rank::Fifth.relative_to(self.stm)
            ).bitboard();

            Piece::Pawn
        }  else {
            self.piece_on(to).unwrap()
        };

        let mut attackers = self.attackers(to, blockers) & blockers;
        let mut target_piece = self.piece_on(from).unwrap();
        let mut stm = !self.stm;
        let mut gains: ArrayVec<i16, 32> = ArrayVec::new();
        gains.push(first_capture.see_value());

        'see: loop {
            for &piece in Piece::ALL.iter() {
                let stm_attackers = attackers & self.color_pieces(piece, stm);

                if let Some(sq) = stm_attackers.try_next_square() {
                    gains.push(target_piece.see_value());

                    if target_piece == Piece::King {
                        break;
                    }

                    let bb = sq.bitboard();

                    blockers ^= bb;
                    attackers ^= bb;
                    target_piece = piece;

                    if matches!(piece, Piece::Rook | Piece::Queen) {
                        attackers |= rook_moves(sq, blockers) & blockers & self.orth_sliders();
                    }

                    if matches!(piece, Piece::Pawn | Piece::Bishop | Piece::Queen) {
                        attackers |= bishop_moves(sq, blockers) & blockers & self.diag_sliders();
                    }

                    stm = !stm;
                    continue 'see;
                }
            }

            while gains.len() > 1 {
                let forced = gains.len() == 2;
                let their_gain = gains.pop().unwrap();
                let our_gain = gains.last_mut().unwrap();

                *our_gain -= their_gain;

                if !forced && *our_gain < 0 {
                    *our_gain = 0;
                }
            }

            return gains.pop().unwrap();
        }
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
        ];
        let expected = &[
            Piece::Knight.see_value(),
            Piece::Knight.see_value() - Piece::Rook.see_value(),
            0,
            0,
        ];

        let moves = &[
            Move::new(Square::D2, Square::D5, MoveFlag::None),
            Move::new(Square::D2, Square::D5, MoveFlag::None),
            Move::new(Square::D3, Square::D5, MoveFlag::None),
            Move::new(Square::D3, Square::C4, MoveFlag::None),
        ];

        for ((&fen, &expected), &mv) in fens.iter().zip(expected).zip(moves) {
            let board = Board::from_fen(fen, false).unwrap();

            assert!(board.see(mv) >= expected);
            assert!(board.see(mv) < (expected + 1));
        }
    }
}