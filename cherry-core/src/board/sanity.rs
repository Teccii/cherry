use crate::*;

macro_rules! soft_assert {
    ($e:expr) => {
        if !$e {
            return false;
        }
    }
}

impl Board {
    #[inline(always)]
    pub fn is_sane(&self) -> bool {
        soft_assert!(self.board_is_sane());
        soft_assert!(self.checkers_is_sane());
        soft_assert!(self.castle_rights_is_sane());
        soft_assert!(self.en_passant_is_sane());
        soft_assert!(self.halfmove_clock_is_sane());
        soft_assert!(self.fullmove_count_is_sane());

        true
    }

    pub(crate) fn board_is_sane(&self) -> bool {
        let mut occupied = Bitboard::EMPTY;

        for &piece in &Piece::ALL {
            let pieces = self.pieces(piece);

            soft_assert!(pieces.is_disjoint(occupied));
            occupied |= pieces;
        }

        soft_assert!(self.colors(Color::White).is_disjoint(self.colors(Color::Black)));
        soft_assert!(occupied == self.occupied());

        for &color in &Color::ALL {
            let colors = self.colors(color);
            let pawn_mask = Rank::First.bitboard() | Rank::Eighth.bitboard();

            soft_assert!(colors.popcnt() <= 16);
            soft_assert!((colors & self.pieces(Piece::King)).popcnt() == 1);
            soft_assert!((colors & self.pieces(Piece::Pawn)).popcnt() <= 8);
            soft_assert!((colors & self.pieces(Piece::Pawn)).is_disjoint(pawn_mask));
        }

        true
    }

    pub(crate) fn en_passant_is_sane(&self) -> bool {
        if let Some(ep_file) = self.en_passant {
            let from = Square::new(ep_file, Rank::Seventh.relative_to(self.stm));
            let to = Square::new(ep_file, Rank::Sixth.relative_to(self.stm));
            let pawn = Square::new(ep_file, Rank::Fifth.relative_to(self.stm));

            soft_assert!(self.color_pieces(Piece::Pawn, !self.stm).has(pawn));
            soft_assert!(!self.occupied().has(from));
            soft_assert!(!self.occupied().has(to));

            let king = self.king(self.stm);
            for checker in self.checkers {
                let ray_through = between(checker, king).has(from);
                soft_assert!(checker == pawn || ray_through)
            }
        }

        true
    }

    pub(crate) fn castle_rights_is_sane(&self) -> bool {
        for &color in &Color::ALL {
            let back_rank = Rank::First.relative_to(color);
            let rights = self.castle_rights(color);
            let rooks = self.color_pieces(Piece::Rook, color);

            if rights.short.is_some() || rights.long.is_some() {
                let king = self.king(color);

                soft_assert!(king.rank() == back_rank);

                if let Some(rook) = rights.short {
                    soft_assert!(rooks.has(Square::new(rook, back_rank)));
                    soft_assert!(king.file() < rook);
                }

                if let Some(rook) = rights.long {
                    soft_assert!(rooks.has(Square::new(rook, back_rank)));
                    soft_assert!(rook < king.file());
                }
            }
        }

        true
    }

    pub(crate) fn checkers_is_sane(&self) -> bool {
        let (checkers, _) = self.checks_and_pins(!self.stm);

        soft_assert!(checkers.is_empty()); //opponent can't be in check when it's our turn

        let (checkers, pinned) = self.checks_and_pins(self.stm);
        soft_assert!(checkers == self.checkers);
        soft_assert!(pinned == self.pinned);
        soft_assert!(self.checkers.popcnt() < 3);

        true
    }

    #[inline(always)]
    pub(crate) fn halfmove_clock_is_sane(&self) -> bool {
        self.halfmove_clock <= 100
    }

    #[inline(always)]
    pub(crate) fn fullmove_count_is_sane(&self) -> bool {
        self.fullmove_count > 0
    }

    pub fn checks_and_pins(&self, color: Color) -> (Bitboard, [Bitboard; Color::COUNT]) {
        let mut checkers = Bitboard::EMPTY;
        let mut pinned = [Bitboard::EMPTY; Color::COUNT];

        let our_king = self.king(color);
        let (diag, orth) = (self.diag_sliders(), self.orth_sliders());
        let their_attackers = self.colors(!color) & (
            (bishop_rays(our_king) & diag) | (rook_rays(our_king) & orth)
        );

        let occ = self.occupied();
        for sq in their_attackers {
            let between = between(sq, our_king) & occ;

            match between.popcnt() {
                0 => checkers |= sq.bitboard(),
                1 => pinned[color as usize] |= between,
                _ => ()
            }
        }

        let their_king = self.king(!color);
        let our_attackers = self.colors(color) & (
            (bishop_rays(their_king) & diag) | (rook_rays(their_king) & orth)
        );
        
        for sq in our_attackers {
            let between = between(sq, their_king) & occ;
            
            if between.popcnt() == 1 {
                pinned[!color as usize] |= between;
            }
        }

        checkers |= knight_moves(our_king) & self.color_pieces(Piece::Knight, !color);
        checkers |= pawn_attacks(our_king, color) & self.color_pieces(Piece::Pawn, !color);

        (checkers, pinned)
    }
}