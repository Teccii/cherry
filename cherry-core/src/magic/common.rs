use crate::*;

pub const fn bishop_relevant_blockers(sq: Square) -> Bitboard {
    let mut rays = Bitboard::EMPTY;
    let mut i = 0;
    while i < Square::COUNT {
        let target = Square::index(i);
        let dx = (sq.file() as i8 - target.file() as i8).abs();
        let dy = (sq.rank() as i8 - target.rank() as i8).abs();
        if dy == dx && dy != 0 {
            rays.0 |= target.bitboard().0;
        }

        i += 1;
    }

    Bitboard(rays.0 & !Bitboard::EDGES.0)
}

pub const fn rook_relevant_blockers(sq: Square) -> Bitboard {
    let rank_moves = sq.rank().bitboard().0 & !(File::A.bitboard().0 | File::H.bitboard().0);
    let file_moves =
        sq.file().bitboard().0 & !(Rank::First.bitboard().0 | Rank::Eighth.bitboard().0);

    Bitboard((rank_moves | file_moves) & !sq.bitboard().0)
}

/*----------------------------------------------------------------*/

#[inline]
const fn slider_moves_slow(sq: Square, mut blockers: Bitboard, deltas: &[(i8, i8); 4]) -> Bitboard {
    blockers.0 &= !sq.bitboard().0;

    let mut moves = Bitboard::EMPTY;
    let mut i = 0;

    while i < deltas.len() {
        let (dx, dy) = deltas[i];
        let mut sq = sq;

        while !blockers.has(sq) {
            if let Some(next) = sq.try_offset(dx, dy) {
                sq = next;
                moves.0 |= sq.bitboard().0;
            } else {
                break;
            }
        }

        i += 1;
    }

    moves
}

pub const fn bishop_moves_slow(sq: Square, blockers: Bitboard) -> Bitboard {
    slider_moves_slow(sq, blockers, &[(1, 1), (1, -1), (-1, -1), (-1, 1)])
}

pub const fn rook_moves_slow(sq: Square, blockers: Bitboard) -> Bitboard {
    slider_moves_slow(sq, blockers, &[(1, 0), (0, -1), (-1, 0), (0, 1)])
}

/*----------------------------------------------------------------*/

static BISHOP_RAYS: [Bitboard; Square::COUNT] = {
    const fn calc_rays(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let sq = sq.bitboard();

        bb.0 |= sq.smear::<NorthEast>().0;
        bb.0 |= sq.smear::<NorthWest>().0;
        bb.0 |= sq.smear::<SouthEast>().0;
        bb.0 |= sq.smear::<SouthWest>().0;
        bb.0 &= !sq.0;

        bb
    }

    let mut table = [Bitboard::EMPTY; Square::COUNT];
    let mut i = 0;
    while i < Square::COUNT {
        table[i] = calc_rays(Square::index(i));
        i += 1;
    }

    table
};

static ROOK_RAYS: [Bitboard; Square::COUNT] = {
    const fn calc_rays(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let sq = sq.bitboard();

        bb.0 |= sq.smear::<North>().0;
        bb.0 |= sq.smear::<South>().0;
        bb.0 |= sq.smear::<East>().0;
        bb.0 |= sq.smear::<West>().0;
        bb.0 &= !sq.0;

        bb
    }

    let mut table = [Bitboard::EMPTY; Square::COUNT];
    let mut i = 0;
    while i < Square::COUNT {
        table[i] = calc_rays(Square::index(i));
        i += 1;
    }

    table
};

static QUEEN_RAYS: [Bitboard; Square::COUNT] = {
    const fn calc_rays(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let sq = sq.bitboard();

        bb.0 |= sq.smear::<North>().0;
        bb.0 |= sq.smear::<NorthEast>().0;
        bb.0 |= sq.smear::<NorthWest>().0;
        bb.0 |= sq.smear::<South>().0;
        bb.0 |= sq.smear::<SouthEast>().0;
        bb.0 |= sq.smear::<SouthWest>().0;
        bb.0 |= sq.smear::<East>().0;
        bb.0 |= sq.smear::<West>().0;
        bb.0 &= !sq.0;

        bb
    }

    let mut table = [Bitboard::EMPTY; Square::COUNT];
    let mut i = 0;
    while i < Square::COUNT {
        table[i] = calc_rays(Square::index(i));
        i += 1;
    }

    table
};

#[inline]
pub const fn bishop_rays(sq: Square) -> Bitboard {
    BISHOP_RAYS[sq as usize]
}

#[inline]
pub const fn rook_rays(sq: Square) -> Bitboard {
    ROOK_RAYS[sq as usize]
}

#[inline]
pub const fn queen_rays(sq: Square) -> Bitboard {
    QUEEN_RAYS[sq as usize]
}
