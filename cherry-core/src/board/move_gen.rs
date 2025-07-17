use crate::*;

/*----------------------------------------------------------------*/

mod slider {
    use super::*;

    pub trait SlidingPiece {
        const PIECE: Piece;

        fn pseudo_legals(square: Square, blockers: Bitboard) -> Bitboard;
    }

    macro_rules! impl_sliding_piece {
        ($square:ident,$color:ident,$blockers:ident; $($type:ident => $impl:expr),*) => {
            $(pub struct $type;

            impl SlidingPiece for $type {
                const PIECE: Piece = Piece::$type;

                fn pseudo_legals($square: Square, $blockers: Bitboard) -> Bitboard {
                    $impl
                }
            })*
        };
    }

    impl_sliding_piece! {
        sq, color, blockers;
        Bishop => bishop_moves(sq, blockers),
        Rook => rook_moves(sq, blockers),
        Queen => queen_moves(sq, blockers)
    }
}

/*----------------------------------------------------------------*/

macro_rules! abort_if {
    ($($expr:expr),*) => {
        $(if $expr {
            return true;
        })*
    }
}

/*----------------------------------------------------------------*/


impl Board {
    // Squares we can land on. When we're in check, we have to block
    // or capture the checker. In any case, we can't land on our own
    // pieces. Assumed to only be called if there is only one checker.
    fn target_squares<const IN_CHECK: bool>(&self) -> Bitboard {
        let targets = if IN_CHECK {
            let checker = self.checkers().try_next_square().unwrap_or_else(|| {
                panic!("Board {}", self);
            });
            let our_king = self.king(self.stm);
            between(checker, our_king) | checker.bitboard()
        } else {
            Bitboard::FULL
        };
        targets & !self.colors(self.stm)
    }

    /*----------------------------------------------------------------*/

    fn add_slider_legals<
        P: slider::SlidingPiece, F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: Bitboard, listener: &mut F) -> bool {
        let pieces = self.color_pieces(P::PIECE, self.stm) & mask;
        let target_squares = self.target_squares::<IN_CHECK>();
        let pinned = self.pinned(self.stm);
        let blockers = self.occupied();

        for piece in pieces & !pinned {
            let moves = P::pseudo_legals(piece, blockers) & target_squares;
            if !moves.is_empty() {
                abort_if!(listener(PieceMoves {
                    piece: P::PIECE,
                    from: piece,
                    to: moves,
                    flag: MoveFlag::None
                }));
            }
        }

        if !IN_CHECK {
            let our_king = self.king(self.stm);

            for piece in pieces & pinned {
                //If we're not in check, we can still slide along the pinned ray.
                let target_squares = target_squares & line(our_king, piece);
                let moves = P::pseudo_legals(piece, blockers) & target_squares;
                if !moves.is_empty() {
                    abort_if!(listener(PieceMoves {
                        piece: P::PIECE,
                        from: piece,
                        to: moves,
                        flag: MoveFlag::None
                    }));
                }
            }
        }

        false
    }

    /*----------------------------------------------------------------*/

    fn add_knight_legals<
        F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: Bitboard, listener: &mut F) -> bool {
        const PIECE: Piece = Piece::Knight;

        let pieces = self.color_pieces(PIECE, self.stm) & mask;
        let target_squares = self.target_squares::<IN_CHECK>();
        let pinned = self.pinned(self.stm);

        for piece in pieces & !pinned {
            let moves = knight_moves(piece) & target_squares;
            if !moves.is_empty() {
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: piece,
                    to: moves,
                    flag: MoveFlag::None
                }));
            }
        }

        false
    }

    /*----------------------------------------------------------------*/

    fn add_pawn_legals<
        F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: Bitboard, listener: &mut F) -> bool {
        const PIECE: Piece = Piece::Pawn;

        let our_king = self.king(self.stm);
        let target_squares = self.target_squares::<IN_CHECK>();
        let pieces = self.color_pieces(PIECE, self.stm) & mask;
        let their_pieces = self.colors(!self.stm);
        let pinned = self.pinned(self.stm);
        let blockers = self.occupied();

        for piece in pieces & !pinned {
            let moves = (
                pawn_quiets(piece, self.stm, blockers) | (pawn_attacks(piece, self.stm) & their_pieces)
            ) & target_squares;

            if !moves.is_empty() {
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: piece,
                    to: moves,
                    flag: MoveFlag::None
                }));
            }
        }

        if !IN_CHECK {
            for piece in pieces & pinned {
                //If we're not in check, we can still slide along the pinned ray.
                let target_squares = target_squares & line(our_king, piece);
                let moves = (
                    pawn_quiets(piece, self.stm, blockers) | (pawn_attacks(piece, self.stm) & their_pieces)
                ) & target_squares;

                if !moves.is_empty() {
                    abort_if!(listener(PieceMoves {
                        piece: PIECE,
                        from: piece,
                        to: moves,
                        flag: MoveFlag::None
                    }));
                }
            }
        }

        if let Some(en_passant) = self.en_passant() {
            let diag = their_pieces & self.diag_sliders();
            let orth = their_pieces & self.orth_sliders();

            let dest = Square::new(en_passant, Rank::Third.relative_to(!self.stm));
            let victim = Square::new(en_passant, Rank::Fourth.relative_to(!self.stm));

            for piece in pawn_attacks(dest, !self.stm) & pieces {
                //Simulate the capture and update the pieces accordingly.
                let blockers = blockers
                    ^ victim.bitboard()
                    ^ piece.bitboard()
                    | dest.bitboard();
                //First test a basic ray to prevent an expensive magic lookup
                let on_ray = !(bishop_rays(our_king) & diag).is_empty();
                if on_ray && !(bishop_moves(our_king, blockers) & diag).is_empty() {
                    continue;
                }
                let on_ray = !(rook_rays(our_king) & orth).is_empty();
                if on_ray && !(rook_moves(our_king, blockers) & orth).is_empty() {
                    continue;
                }
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: piece,
                    to: dest.bitboard(),
                    flag: MoveFlag::EnPassant
                }));
            }
        }
        false
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn king_safe_on(&self, square: Square) -> bool {
        macro_rules! short_circuit {
            ($($attackers:expr),*) => {
                $(if !$attackers.is_empty() {
                    return false;
                })*
                true
            }
        }

        let their_pieces = self.colors(!self.stm);
        let blockers = self.occupied()
            ^ self.color_pieces(Piece::King, self.stm)
            | square.bitboard();

        short_circuit! {
            bishop_moves(square, blockers) & their_pieces & (
                self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)
            ),
            rook_moves(square, blockers) & their_pieces & (
                self.pieces(Piece::Rook) | self.pieces(Piece::Queen)
            ),
            knight_moves(square) & their_pieces & self.pieces(Piece::Knight),
            king_moves(square) & their_pieces & self.pieces(Piece::King),
            pawn_attacks(square, self.stm) & their_pieces & self.pieces(Piece::Pawn)
        }
    }

    fn can_castle(&self, rook: File, king_dest: File, rook_dest: File) -> bool {
        let our_king = self.king(self.stm);
        let back_rank = Rank::First.relative_to(self.stm);
        let rook = Square::new(rook, back_rank);
        let blockers = self.occupied() ^ our_king.bitboard() ^ rook.bitboard();
        let king_dest = Square::new(king_dest, back_rank);
        let rook_dest = Square::new(rook_dest, back_rank);
        let king_to_rook = between(our_king, rook);
        let king_to_dest = between(our_king, king_dest);
        let must_be_safe = king_to_dest | king_dest.bitboard();
        let must_be_empty = must_be_safe | king_to_rook | rook_dest.bitboard();

        !self.pinned(self.stm).has(rook)
            && (blockers & must_be_empty).is_empty()
            && must_be_safe.iter().all(|square| self.king_safe_on(square))
    }

    fn add_king_legals<
        F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: Bitboard, listener: &mut F) -> bool {
        const PIECE: Piece = Piece::King;

        let our_pieces = self.colors(self.stm);
        let our_king = self.king(self.stm);
        if !mask.has(our_king) {
            return false;
        }
        let mut moves = Bitboard::EMPTY;
        for to in king_moves(our_king) & !our_pieces {
            if self.king_safe_on(to) {
                moves |= to.bitboard();
            }
        }

        if !moves.is_empty() {
            abort_if!(listener(PieceMoves {
                piece: PIECE,
                from: our_king,
                to: moves,
                flag: MoveFlag::None
            }));
        }

        if !IN_CHECK {
            let rights = self.castle_rights(self.stm);
            let back_rank = Rank::First.relative_to(self.stm);
            moves = Bitboard::EMPTY;

            if let Some(rook) = rights.short {
                if self.can_castle(rook, File::G, File::F) {
                    moves |= Square::new(rook, back_rank).bitboard();
                }
            }
            if let Some(rook) = rights.long {
                if self.can_castle(rook, File::C, File::D) {
                    moves |= Square::new(rook, back_rank).bitboard();
                }
            }

            if !moves.is_empty() {
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: our_king,
                    to: moves,
                    flag: MoveFlag::Castling
                }));
            }
        }

        false
    }

    /*----------------------------------------------------------------*/

    fn add_all_legals<
        F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: Bitboard, listener: &mut F) -> bool {
        abort_if! {
            self.add_pawn_legals::<_, IN_CHECK>(mask, listener),
            self.add_knight_legals::<_, IN_CHECK>(mask, listener),
            self.add_slider_legals::<slider::Bishop, _, IN_CHECK>(mask, listener),
            self.add_slider_legals::<slider::Rook, _, IN_CHECK>(mask, listener),
            self.add_slider_legals::<slider::Queen, _, IN_CHECK>(mask, listener),
            self.add_king_legals::<_, IN_CHECK>(mask, listener)
        }
        false
    }

    /*----------------------------------------------------------------*/

    pub fn gen_moves(&self, listener: impl FnMut(PieceMoves) -> bool) -> bool {
        self.gen_moves_for(Bitboard::FULL, listener)
    }

    pub fn gen_moves_for(
        &self, mask: Bitboard, mut listener: impl FnMut(PieceMoves) -> bool
    ) -> bool {
        match self.checkers().popcnt() {
            0 => self.add_all_legals::<_, false>(mask, &mut listener),
            1 => self.add_all_legals::<_, true>(mask, &mut listener),
            _ => self.add_king_legals::<_, true>(mask, &mut listener)
        }
    }

    /*----------------------------------------------------------------*/

    pub fn is_capture(&self, mv: Move) -> bool {
        self.colors(!self.stm).has(mv.to()) || self.is_en_passant(mv)
    }

    pub fn is_quiet(&self, mv: Move) -> bool {
        !self.is_capture(mv)
    }

    pub fn is_check(&self, mv: Move) -> bool {
        let mut board = self.clone();
        board.make_move(mv);

        board.in_check()
    }

    pub fn is_castling(&self, mv: Move) -> bool {
        mv.is_castling() || (self.king(self.stm) == mv.from() && self.colors(self.stm).has(mv.to()))
    }

    pub fn is_en_passant(&self, mv: Move) -> bool {
        mv.is_en_passant() || (
            Some(mv.to()) == self.ep_square() && self.piece_on(mv.from()).unwrap() == Piece::Pawn
        )
    }

    pub fn victim(&self, mv: Move) -> Option<Piece> {
        if self.is_en_passant(mv) {
            Some(Piece::Pawn)
        } else if self.is_capture(mv) {
            Some(self.piece_on(mv.to()).unwrap())
        } else {
            None
        }
    }

    /*----------------------------------------------------------------*/

    fn king_is_legal(&self, mv: Move) -> bool {
        let (from, to) = (mv.from(), mv.to());

        if self.checkers.is_empty() {
            let castles = self.castle_rights(self.stm);
            let back_rank = Rank::First.relative_to(self.stm);

            if let Some(rook) = castles.short {
                let rook_square = Square::new(rook, back_rank);
                if rook_square == to && self.can_castle(rook, File::G, File::F) {
                    return true;
                }
            }
            if let Some(rook) = castles.long {
                let rook_square = Square::new(rook, back_rank);
                if rook_square == to && self.can_castle(rook, File::C, File::D) {
                    return true;
                }
            }
        }
        if !(king_moves(from) & !self.colors(self.stm)).has(to) {
            return false;
        }

        if mv.is_promotion() {
            return false;
        }

        self.king_safe_on(to)
    }

    pub fn is_legal(&self, mv: Move) -> bool {
        let (from, to) = (mv.from(), mv.to());

        if !self.colors(self.stm).has(from) {
            return false;
        }

        let king_sq = self.king(self.stm);
        if from == king_sq {
            if mv.is_promotion() {
                return false;
            }

            return self.king_is_legal(mv);
        }

        if self.pinned(self.stm).has(from) && !line(king_sq, from).has(to) {
            return false;
        }

        let target_squares = match self.checkers().popcnt() {
            0 => self.target_squares::<false>(),
            1 => self.target_squares::<true>(),
            _ => return false,
        };

        let piece = self.piece_on(from);
        if piece != Some(Piece::Pawn) && mv.is_promotion() {
            return false;
        }

        match piece {
            None | Some(Piece::King) => false, // impossible
            Some(Piece::Pawn) => {
                let promo_rank = Rank::Eighth.relative_to(self.stm);
                match (to.rank() == promo_rank, mv.promotion()) {
                    (true, Some(Piece::Knight | Piece::Bishop | Piece::Rook | Piece::Queen)) => {}
                    (false, None) => {}
                    _ => return false,
                }
                let mut c = |moves: PieceMoves| moves.to.has(to);

                if self.checkers().is_empty() {
                    self.add_pawn_legals::<_, false>(from.bitboard(), &mut c)
                } else {
                    self.add_pawn_legals::<_, true>(from.bitboard(), &mut c)
                }
            }
            Some(Piece::Rook) => {
                (target_squares & rook_rays(from)).has(to) && (between(from, to) & self.occupied()).is_empty()
            }
            Some(Piece::Bishop) => {
                (target_squares & bishop_rays(from)).has(to) && (between(from, to) & self.occupied()).is_empty()
            }
            Some(Piece::Knight) => (target_squares & knight_moves(from)).has(to),
            Some(Piece::Queen) => {
                (target_squares & queen_rays(from)).has(to) && (between(from, to) & self.occupied()).is_empty()
            }
        }
    }
}

/*----------------------------------------------------------------*/

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::*;

    /*----------------------------------------------------------------*/

    fn test_legality(board: Board) {
        let mut legals = HashSet::new();
        board.gen_moves(|moves| {
            legals.extend(moves);
            false
        });

        const PROMOTIONS: [Piece; 4] = [
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen
        ];

        const FLAGS: [MoveFlag; 4] = [
            MoveFlag::None,
            MoveFlag::EnPassant,
            MoveFlag::Castling,
            MoveFlag::Promotion
        ];

        for &from in &Square::ALL {
            for &to in &Square::ALL {
                if from == to {
                    continue;
                }

                for &promotion in &PROMOTIONS {
                    let mv = Move::new_promotion(from, to, promotion);

                    assert_eq!(legals.contains(&mv),board.is_legal(mv));
                }

                let mv = Move::new(from, to, MoveFlag::None);
                assert_eq!(legals.contains(&mv), board.is_legal(mv));
            }
        }
    }

    /*----------------------------------------------------------------*/

    #[test]
    fn simple_legals() {
        test_legality(Board::default());
        test_legality(
            "rk2r3/pn1p1p1p/1p4NB/2pP1K2/4p2N/1P3BP1/P1P3PP/1R3q2 w - - 2 32"
                .parse()
                .unwrap()
        );
    }

    #[test]
    fn castle_legals() {
        test_legality(
            "rnbqk2r/ppppbp1p/5np1/4p3/4P3/3P1N2/PPP1BPPP/RNBQK2R w KQkq - 0 5"
                .parse()
                .unwrap(),
        );
        test_legality(
            "rnbqk2r/ppppbp1p/5npB/4p3/4P3/3P1N2/PPP1BPPP/RN1QK2R b KQkq - 1 5"
                .parse()
                .unwrap(),
        );
        test_legality(
            "r1bqk2r/ppppbp1p/2n2npB/4p3/4P3/2NP1N2/PPPQBPPP/R3K2R w KQq - 6 8"
                .parse()
                .unwrap(),
        );
        test_legality(
            "r1bqk2r/ppppbp1p/2n2npB/4p3/4P3/2NP1N2/PPPQBPPP/R2K3R b q - 7 8"
                .parse()
                .unwrap(),
        );
        test_legality(
            "rnbqkbn1/pppprppp/8/8/8/8/PPPP1PPP/RNBQK2R w KQq - 0 1"
                .parse()
                .unwrap(),
        );
    }

    #[test]
    fn castle_960_legals() {
        test_legality(
            Board::from_fen(
                "rq1kr3/p1ppbp1p/bpn3pB/3Np3/3P4/1P1Q1Nn1/P1P1BPPP/R2KR3 w AEae - 3 15",
                true,
            ).unwrap(),
        );
        test_legality(
            Board::from_fen(
                "rq1kr3/p1ppbp1p/bpn3pB/3Np3/3P4/1P1Q1Nn1/P1P1BPPP/R2KR3 b AEae - 3 15",
                true,
            ).unwrap(),
        );
        test_legality(
            Board::from_fen(
                "rk2r3/pqppbp1p/bpn3pB/3Npn2/3P4/1P1Q1N2/P1P2PPP/RKRB4 w ACa - 3 15",
                true,
            ).unwrap(),
        );
    }

    #[test]
    fn en_passant_legals() {
        test_legality(
            "rk2r3/pn1p1p1p/1p4NB/q1pP1K2/4p2b/1P3NP1/P1P3PP/R1RB4 w - - 0 29"
                .parse()
                .unwrap(),
        );
        test_legality(
            "rk2r3/pn1p1p1p/1p4NB/2pP1K2/4p2N/qP4P1/P1P3PP/R1RB4 w - c6 0 30"
                .parse()
                .unwrap(),
        );
    }
}