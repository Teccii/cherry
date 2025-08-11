use cherry_types::*;
use crate::*;

impl Board {
    #[inline]
    pub fn attacks(&self, sq: Square, blockers: Bitboard) -> Bitboard {
        (knight_moves(sq) & self.pieces(Piece::Knight))
            | (king_moves(sq) & self.pieces(Piece::King))
            | (bishop_moves(sq, blockers) & self.diag_sliders())
            | (rook_moves(sq, blockers) & self.orth_sliders())
            | (pawn_attacks(sq, Color::White) & self.color_pieces(Piece::Pawn, Color::Black))
            | (pawn_attacks(sq, Color::Black) & self.color_pieces(Piece::Pawn, Color::White))
    }

    #[inline]
    pub fn pawn_attacks(&self, color: Color) -> Bitboard {
        let pawns = self.color_pieces(Piece::Pawn, color);

        match color {
            Color::White => pawns.shift::<UpLeft>(1) | pawns.shift::<UpRight>(1),
            Color::Black => pawns.shift::<DownLeft>(1) | pawns.shift::<DownRight>(1),
        }
    }

    /*----------------------------------------------------------------*/

    //It's a surprise tool that will help us later
    #[allow(dead_code)]
    pub fn threats(&self, color: Color) -> Bitboard {
        let mut threats = Bitboard::EMPTY;
        let their_pieces = self.colors(!color);
        let occ = self.occupied();

        threats |= self.pawn_attacks(color) & their_pieces;
        threats |= king_moves(self.king(color)) & their_pieces;

        for knight in self.color_pieces(Piece::Knight, color) {
            threats |= knight_moves(knight) & their_pieces;
        }

        for slider in self.color_diag_sliders(color) {
            let any_threats = !(bishop_rays(slider) & their_pieces).is_empty();
            if any_threats {
                threats |= bishop_moves(slider, occ) & their_pieces;
            }
        }

        for slider in self.color_orth_sliders(color) {
            let any_threats = !(rook_rays(slider) & their_pieces).is_empty();
            if any_threats {
                threats |= rook_moves(slider, occ) & their_pieces;
            }
        }

        threats
    }
}