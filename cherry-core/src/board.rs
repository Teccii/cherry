use crate::*;

#[derive(Debug, Copy, Clone)]
pub struct Board {
    colors: [Bitboard; Color::COUNT],
    pieces: [Bitboard; Piece::COUNT],
    castling_rights: [CastlingRights; Color::COUNT],
    pinned: Bitboard,
    pinners: Bitboard,
    checkers: Bitboard,
    en_passant: Option<File>,
    turn: Color,
    hash: u64,
    pawn_hash: u64,
}