use std::cell::SyncUnsafeCell;

use pyrrhic_rs::{EngineAdapter, TableBases, WdlProbeResult};
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct SyzygyAdapter;

impl EngineAdapter for SyzygyAdapter {
    fn pawn_attacks(color: pyrrhic_rs::Color, sq: u64) -> u64 {
        pawn_attacks(Square::index(sq as usize), match color {
            pyrrhic_rs::Color::White => Color::White,
            pyrrhic_rs::Color::Black => Color::Black,
        }).0
    }

    fn knight_attacks(sq: u64) -> u64 {
        knight_moves(Square::index(sq as usize)).0
    }

    fn king_attacks(sq: u64) -> u64 {
        king_moves(Square::index(sq as usize)).0
    }

    fn bishop_attacks(sq: u64, blockers: u64) -> u64 {
        bishop_moves(Square::index(sq as usize), Bitboard(blockers)).0
    }
    
    fn rook_attacks(sq: u64, blockers: u64) -> u64 {
        rook_moves(Square::index(sq as usize), Bitboard(blockers)).0
    }

    fn queen_attacks(sq: u64, blockers: u64) -> u64 {
        queen_moves(Square::index(sq as usize), Bitboard(blockers)).0
    }
}

/*----------------------------------------------------------------*/

pub static SYZYGY: SyncUnsafeCell<Option<TableBases<SyzygyAdapter>>> = SyncUnsafeCell::new(None);

pub fn set_syzygy_path(path: &str) {
    unsafe {
        let syzygy = &mut *SYZYGY.get();

        if let Some(old) = syzygy.take() {
            drop(old);
        }
        
        *syzygy = TableBases::<SyzygyAdapter>::new(path).ok();
    }
}

pub fn probe_wdl(board: &Board) -> Option<WdlProbeResult> {
    let tb = unsafe { &*SYZYGY.get() }.as_ref()?;

    if board.occupied().popcnt() as u32 > tb.max_pieces() {
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
        board.ep_square().map_or(0, |sq| sq as u32),
        board.stm() == Color::White,
    ).ok()
}