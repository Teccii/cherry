use crate::*;

impl Board {
    #[inline]
    pub fn is_legal(&self, mv: Move) -> bool {
        let checkers = self.checkers();

        match checkers.popcnt() {
            0 => self.is_legal_no_check(mv),
            1 => self.is_legal_check(mv, checkers),
            2 => self.is_legal_evasion::<2>(mv, checkers),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn is_legal_no_check(&self, mv: Move) -> bool {
        let (src, dest, flag) = (mv.src(), mv.dest(), mv.flag());
        let src_place = self.inner.get(src);

        if src_place.color() != Some(self.stm) {
            return false;
        }
        let src_piece = src_place.piece().unwrap();

        match flag {
            MoveFlag::DoublePush
                if src_piece != Piece::Pawn || src.rank() != Rank::Second.relative_to(self.stm) =>
                return false,
            MoveFlag::EnPassant => return self.is_legal_ep(mv),
            _ if mv.is_castling() => return self.is_legal_castling(mv),
            _ if mv.is_promotion()
                && (src_piece != Piece::Pawn
                    || dest.rank() != Rank::Eighth.relative_to(self.stm)) =>
                return false,
            _ => {}
        }

        if self.colors(self.stm).has(dest) {
            return false;
        }

        let masked_attacks = Wordboard(self.attack_table(self.stm).0 & self.pinned_mask.0);
        let is_attacked = match src_piece {
            Piece::Pawn => {
                let pinned_pawns = self.pinned & !self.king(self.stm).file().bitboard();
                match flag {
                    MoveFlag::DoublePush =>
                        !pinned_pawns.has(src)
                            && pawn_quiets(src, self.stm, self.occupied()).has(dest),
                    _ if mv.is_capture() =>
                        masked_attacks.get(dest).has(src_place.index().unwrap())
                            && (dest.rank() == Rank::Eighth.relative_to(self.stm))
                                == mv.is_promotion(),
                    _ =>
                        !pinned_pawns.has(src)
                            && src.try_offset(0, self.stm.sign() as i8) == Some(dest)
                            && (dest.rank() == Rank::Eighth.relative_to(self.stm))
                                == mv.is_promotion(),
                }
            }
            Piece::King =>
                masked_attacks.get(dest).has(PieceIndex::KING)
                    && !self.attack_table(!self.stm).all().has(dest),
            _ => masked_attacks.get(dest).has(src_place.index().unwrap()),
        };

        is_attacked && self.colors(!self.stm).has(dest) == mv.is_capture()
    }

    #[inline]
    fn is_legal_check(&self, mv: Move, checkers: PieceMask) -> bool {
        let (src, dest, flag) = (mv.src(), mv.dest(), mv.flag());
        let src_place = self.inner.get(src);

        if src_place.color() != Some(self.stm) {
            return false;
        }

        let src_piece = match src_place.piece() {
            Some(Piece::King) => return self.is_legal_evasion::<1>(mv, checkers),
            Some(piece) => piece,
            None => unreachable!(),
        };

        match flag {
            MoveFlag::DoublePush
                if src_piece != Piece::Pawn || src.rank() != Rank::Second.relative_to(self.stm) =>
                return false,
            _ if mv.is_promotion()
                && (src_piece != Piece::Pawn
                    || dest.rank() != Rank::Eighth.relative_to(self.stm)) =>
                return false,
            _ => {}
        }

        if mv.is_promotion()
            && (src_piece != Piece::Pawn || dest.rank() != Rank::Eighth.relative_to(self.stm))
        {
            return false;
        }

        let king = self.king(self.stm);
        let checker = checkers.next().unwrap();
        let checker_piece = self.index_to_piece[!self.stm][checker].unwrap();
        let checker_sq = self.index_to_square[!self.stm][checker].unwrap();
        let valid = if checker_piece == Piece::Knight {
            checker_sq.bitboard()
        } else {
            between(king, checker_sq) | checker_sq
        };

        if !valid.has(dest) {
            return self
                .en_passant()
                .is_some_and(|f| checker_sq == Square::new(f, Rank::Fifth.relative_to(self.stm)))
                && self.is_legal_ep(mv);
        }

        if flag == MoveFlag::EnPassant {
            return false;
        }

        let masked_attacks = Wordboard(self.attack_table(self.stm).0 & self.pinned_mask.0);
        let is_attacked = match src_piece {
            Piece::Pawn => {
                let pinned_pawns = self.pinned & !self.king(self.stm).file().bitboard();
                match flag {
                    MoveFlag::DoublePush =>
                        !pinned_pawns.has(src)
                            && pawn_quiets(src, self.stm, self.occupied()).has(dest),
                    _ if mv.is_capture() =>
                        masked_attacks.get(dest).has(src_place.index().unwrap())
                            && (dest.rank() == Rank::Eighth.relative_to(self.stm))
                                == mv.is_promotion(),
                    _ =>
                        !pinned_pawns.has(src)
                            && src.try_offset(0, self.stm.sign() as i8) == Some(dest)
                            && (dest.rank() == Rank::Eighth.relative_to(self.stm))
                                == mv.is_promotion(),
                }
            }
            Piece::King =>
                masked_attacks.get(dest).has(PieceIndex::KING)
                    && !self.attack_table(!self.stm).all().has(dest),
            _ => masked_attacks.get(dest).has(src_place.index().unwrap()),
        };

        is_attacked && self.colors(!self.stm).has(dest) == mv.is_capture()
    }

    #[inline]
    fn is_legal_evasion<const CHECKERS: usize>(&self, mv: Move, checkers: PieceMask) -> bool {
        let (src, dest, flag) = (mv.src(), mv.dest(), mv.flag());
        let src_place = self.inner.get(src);

        if src_place.piece() != Some(Piece::King) || src_place.color() != Some(self.stm) {
            return false;
        }

        if ![MoveFlag::Normal, MoveFlag::Capture].contains(&flag) {
            return false;
        }

        let (our_attacks, their_attacks) =
            (self.attack_table(self.stm), self.attack_table(!self.stm));
        let (our_pieces, their_pieces) = (self.colors(self.stm), self.colors(!self.stm));

        let mut valid = our_attacks.for_mask(PieceMask::KING) & !their_attacks.all() & !our_pieces;
        for checker in checkers.into_iter().take(CHECKERS) {
            let checker_piece = self.index_to_piece[!self.stm][checker].unwrap();
            let checker_sq = self.index_to_square[!self.stm][checker].unwrap();

            if checker_piece.is_slider() {
                valid &= !line(checker_sq, src);
            }
        }

        valid.has(dest) && their_pieces.has(dest) == mv.is_capture()
    }

    #[inline]
    fn is_legal_castling(&self, mv: Move) -> bool {
        let (src, dest) = (mv.src(), mv.dest());
        let src_place = self.inner.get(src);

        if src_place.piece() != Some(Piece::King) || src_place.color() != Some(self.stm) {
            return false;
        }

        let our_backrank = Rank::First.relative_to(self.stm);
        let our_rights = self.castle_rights(self.stm);
        let (rook_src, king_dest, rook_dest) = if mv.flag() == MoveFlag::ShortCastling {
            (our_rights.short, File::G, File::F)
        } else {
            (our_rights.long, File::C, File::D)
        };

        let Some(rook_src) = rook_src
            .map(|f| Square::new(f, our_backrank))
            .filter(|&sq| sq == dest)
        else {
            return false;
        };

        let king_dest = Square::new(king_dest, our_backrank);
        let rook_dest = Square::new(rook_dest, our_backrank);
        let king_to_rook = between(src, rook_src);
        let king_to_dest = between(src, king_dest);
        let must_be_safe = king_to_dest | king_dest;
        let must_be_empty = must_be_safe | king_to_rook | rook_dest;
        let blockers = self.occupied() ^ src ^ rook_src;

        !self.pinned.has(rook_src)
            && blockers.is_disjoint(must_be_empty)
            && self.attack_table(!self.stm).all().is_disjoint(must_be_safe)
    }

    #[inline]
    fn is_legal_ep(&self, mv: Move) -> bool {
        if mv.flag() != MoveFlag::EnPassant {
            return false;
        }

        self.en_passant.is_some_and(|ep| {
            let ep_sq = Square::new(ep.file(), Rank::Sixth.relative_to(self.stm));
            let (src, dest) = (mv.src(), mv.dest());

            if dest != ep_sq {
                return false;
            }

            let left = src.file() < ep_sq.file();
            (left && ep.left()) || (!left && ep.right())
        })
    }
}
