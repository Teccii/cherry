use pyrrhic_rs::{DtzProbeResult, EngineAdapter, TableBases, WdlProbeResult};
use cozy_chess::*;
use super::get_queen_moves;

#[derive(Clone)]
pub struct SyzygyAdapter;

impl EngineAdapter for SyzygyAdapter {
    fn pawn_attacks(color: pyrrhic_rs::Color, sq: u64) -> u64 {
        get_pawn_attacks(Square::index(sq as usize), match color {
            pyrrhic_rs::Color::White => Color::White,
            pyrrhic_rs::Color::Black => Color::Black,
        }).0
    }

    fn knight_attacks(sq: u64) -> u64 {
        get_knight_moves(Square::index(sq as usize)).0
    }

    fn king_attacks(sq: u64) -> u64 {
        get_king_moves(Square::index(sq as usize)).0
    }

    fn bishop_attacks(sq: u64, blockers: u64) -> u64 {
        get_bishop_moves(Square::index(sq as usize), BitBoard(blockers)).0
    }
    
    fn rook_attacks(sq: u64, blockers: u64) -> u64 {
        get_rook_moves(Square::index(sq as usize), BitBoard(blockers)).0
    }

    fn queen_attacks(sq: u64, blockers: u64) -> u64 {
        get_queen_moves(Square::index(sq as usize), BitBoard(blockers)).0
    }
}

pub fn probe_wdl(tb: &TableBases<SyzygyAdapter>, board: &Board) -> Option<WdlProbeResult> {
    if board.occupied().len() > tb.max_pieces() {
        return None;
    }
    
    tb.probe_wdl(
        board.colors(Color::White).0,
        board.colors(Color::Black).0,
        board.pieces(Piece::King).0,
        board.pieces(Piece::Queen).0,
        board.pieces(Piece::Rook).0,
        board.pieces(Piece::Bishop).0,
        board.pieces(Piece::Knight).0,
        board.pieces(Piece::Pawn).0,
        0,
        board.side_to_move() == Color::White,
    ).ok()
}

pub fn probe_dtz(tb: &TableBases<SyzygyAdapter>, board: &Board) -> Option<DtzProbeResult> {
    if board.occupied().len() > tb.max_pieces() {
        return None;
    }

    tb.probe_root(
        board.colors(Color::White).0,
        board.colors(Color::Black).0,
        board.pieces(Piece::King).0,
        board.pieces(Piece::Queen).0,
        board.pieces(Piece::Rook).0,
        board.pieces(Piece::Bishop).0,
        board.pieces(Piece::Knight).0,
        board.pieces(Piece::Pawn).0,
        board.halfmove_clock() as u32,
        0,
        board.side_to_move() == Color::White,
    ).ok()
}