use std::arch::x86_64::_pext_u64;
use crate::*;

struct PextEntry {
    offset: u32,
    mask: Bitboard,
}

impl PextEntry {
    pub const EMPTY: PextEntry = PextEntry {
        offset: 0,
        mask: Bitboard::EMPTY,
    };
}

struct PextMagics {
    rook_magics: [PextEntry; Square::COUNT],
    bishop_magics: [PextEntry; Square::COUNT],
    table_size: usize,
}

const MAGICS: &PextMagics = {
    let mut offset = 0;
    let mut rook_magics = [PextEntry::EMPTY; Square::COUNT];
    let mut i = 0;
    while i < Square::COUNT {
        let sq = Square::index(i);
        let mask = rook_relevant_blockers(sq);

        rook_magics[i] = PextEntry { offset, mask};
        offset += 1 << mask.popcnt();
        i += 1;
    }

    let mut bishop_magics = [PextEntry::EMPTY; Square::COUNT];
    let mut i = 0;
    while i < Square::COUNT {
        let sq = Square::index(i);
        let mask = bishop_relevant_blockers(sq);

        bishop_magics[i] = PextEntry { offset, mask};
        offset += 1 << mask.popcnt();
        i += 1;
    }

    &PextMagics {
        rook_magics,
        bishop_magics,
        table_size: offset as usize
    }
};

fn pext_index(magics: &[PextEntry; Square::COUNT], sq: Square, blockers: Bitboard) -> usize {
    let magic = &magics[sq as usize];
    let index = unsafe { _pext_u64(blockers.0, magic.mask.0) };

    magic.offset as usize + index as usize
}

#[inline(always)]
pub fn rook_magic_index(sq: Square, blockers: Bitboard) -> usize {
    pext_index(&MAGICS.rook_magics, sq, blockers)
}

#[inline(always)]
pub fn bishop_magic_index(sq: Square, blockers: Bitboard) -> usize {
    pext_index(&MAGICS.bishop_magics, sq, blockers)
}

pub const SLIDER_TABLE_SIZE: usize = MAGICS.table_size;