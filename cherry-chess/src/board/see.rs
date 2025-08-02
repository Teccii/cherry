use cherry_types::*;
use crate::*;

impl Board {
    /*
    Adapted from Viridithas and Ethereal:
    https://github.com/cosmobobak/viridithas/blob/master/src/search.rs#L1734
    https://github.com/AndyGrant/Ethereal/blob/master/src/search.c#L929
    */
    pub fn cmp_see(&self, mv: Move, threshold: i16) -> bool {
        let (from, to, flag, promotion) = (mv.from(), mv.to(), mv.flag(), mv.promotion());

        let mut next_victim = promotion.unwrap_or_else(|| self.piece_on(from).unwrap());
        let mut balance = -threshold + match flag {
            MoveFlag::None => self.piece_on(to).map_or(0, |p| p.see_value()),
            MoveFlag::EnPassant => Piece::Pawn.see_value(),
            MoveFlag::Promotion => promotion.unwrap().see_value(),
            MoveFlag::Castling => 0,
        };

        //best case fail
        if balance < 0 {
            return false;
        }

        balance -= next_victim.see_value();
        //worst case pass
        if balance >= 0 {
            return true;
        }

        let mut occupied = self.occupied() ^ from | to;
        if flag == MoveFlag::EnPassant {
            occupied ^= self.ep_square().map_or(Bitboard::EMPTY, |sq| sq.bitboard());
        }

        let (diag, orth) = (self.diag_sliders(), self.orth_sliders());
        let (w_pinned, b_pinned) = (
            self.pinned() & self.colors(Color::White),
            self.pinned() & self.colors(Color::Black),
        );
        let (w_checks, b_checks) = (
            queen_rays(self.king(Color::White)),
            queen_rays(self.king(Color::Black))
        );
        let allowed_pieces = !(w_pinned | b_pinned)
            | (w_pinned & w_checks)
            | (b_pinned & b_checks);

        let mut attackers = self.attackers(to, occupied) & allowed_pieces;
        let mut color = !self.stm;

        'see: loop {
            let stm_attackers = attackers & self.colors(color);

            if stm_attackers.is_empty() {
                break 'see;
            }

            //find LVA
            for &piece in &Piece::ALL {
                next_victim = piece;
                if !(stm_attackers & self.pieces(next_victim)).is_empty() {
                    break;
                }
            }

            occupied ^= (stm_attackers & self.pieces(next_victim)).next_square();

            if matches!(next_victim, Piece::Pawn | Piece::Bishop | Piece::Queen) {
                attackers |= bishop_moves(to, occupied) & diag;
            }

            if matches!(next_victim, Piece::Rook | Piece::Queen) {
                attackers |= rook_moves(to, occupied) & orth;
            }

            attackers &= occupied;
            color = !color;

            balance = -balance - 1 - next_victim.see_value();
            if balance >= 0 {
                if next_victim == Piece::King && !(attackers & self.colors(color)).is_empty() {
                    color = !color;
                }

                break;
            }
        }

        self.stm != color
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
            Piece::Knight.see_value(),
            Piece::Knight.see_value() - Piece::Rook.see_value(),
            0,
            0,
            Piece::Pawn.see_value() - Piece::Queen.see_value(),
        ];

        let moves = &[
            Move::new(Square::D2, Square::D5, MoveFlag::None),
            Move::new(Square::D2, Square::D5, MoveFlag::None),
            Move::new(Square::D3, Square::D5, MoveFlag::None),
            Move::new(Square::D3, Square::C4, MoveFlag::None),
            Move::new(Square::D3, Square::D6, MoveFlag::None),
        ];

        for ((&fen, &expected), &mv) in fens.iter().zip(expected).zip(moves) {
            let board = Board::from_fen(fen, false).unwrap();

            assert!(board.cmp_see(mv, expected));
            assert!(!board.cmp_see(mv, expected + 1));
        }
    }
}