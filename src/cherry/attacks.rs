use crate::*;

/*----------------------------------------------------------------*/

include!(concat!(env!("OUT_DIR"), "/slider_moves.rs"));

#[inline]
pub fn bishop_moves(sq: Square, blockers: Bitboard) -> Bitboard {
    SLIDER_MOVES[bishop_magic_index(sq, blockers)]
}

#[inline]
pub fn rook_moves(sq: Square, blockers: Bitboard) -> Bitboard {
    SLIDER_MOVES[rook_magic_index(sq, blockers)]
}

#[inline]
pub fn queen_moves(sq: Square, blockers: Bitboard) -> Bitboard {
    Bitboard(bishop_moves(sq, blockers).0 | rook_moves(sq, blockers).0)
}
