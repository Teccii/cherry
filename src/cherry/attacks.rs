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

/*----------------------------------------------------------------*/

#[inline]
pub const fn pawn_quiets(sq: Square, color: Color, blockers: Bitboard) -> Bitboard {
    let sq_bb = sq.bitboard();
    let mut moves = Bitboard(if let Color::White = color {
        sq_bb.0 << File::COUNT
    } else {
        sq_bb.0 >> File::COUNT
    });

    moves.0 &= !blockers.0;
    if !moves.is_empty() && Rank::Second.relative_to(color).bitboard().has(sq) {
        moves.0 |= if let Color::White = color {
            moves.0 << File::COUNT
        } else {
            moves.0 >> File::COUNT
        };
        moves.0 &= !blockers.0;
    }
    moves
}

#[inline]
pub const fn pawn_attacks(sq: Square, color: Color) -> Bitboard {
    #[inline]
    const fn calc_attacks(sq: Square, color: Color) -> Bitboard {
        let bb = sq.bitboard();

        match color {
            Color::White => Bitboard(bb.shift::<NorthEast>(1).0 | bb.shift::<NorthWest>(1).0),
            Color::Black => Bitboard(bb.shift::<SouthEast>(1).0 | bb.shift::<SouthWest>(1).0),
        }
    }

    const TABLE: [[Bitboard; Square::COUNT]; Color::COUNT] = {
        let mut table = [[Bitboard::EMPTY; Square::COUNT]; Color::COUNT];
        let mut i = 0;
        while i < Color::COUNT {
            let color = Color::index(i);
            let mut j = 0;
            while j < Square::COUNT {
                table[i][j] = calc_attacks(Square::index(j), color);
                j += 1;
            }

            i += 1;
        }

        table
    };

    TABLE[color as usize][sq as usize]
}

/*----------------------------------------------------------------*/

#[inline]
pub const fn knight_moves(sq: Square) -> Bitboard {
    #[inline]
    const fn calc_moves(sq: Square) -> Bitboard {
        const DELTAS: [(i8, i8); 8] = [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ];

        let mut bb = Bitboard::EMPTY;
        let mut i = 0;

        while i < DELTAS.len() {
            let (dx, dy) = DELTAS[i];

            if let Some(mv) = sq.try_offset(dx, dy) {
                bb.0 |= mv.bitboard().0;
            }

            i += 1;
        }

        bb
    }

    const TABLE: [Bitboard; Square::COUNT] = {
        let mut table = [Bitboard::EMPTY; Square::COUNT];
        let mut i = 0;
        while i < Square::COUNT {
            table[i] = calc_moves(Square::index(i));
            i += 1;
        }

        table
    };

    TABLE[sq as usize]
}

/*----------------------------------------------------------------*/

#[inline]
pub const fn king_moves(sq: Square) -> Bitboard {
    #[inline]
    const fn calc_moves(sq: Square) -> Bitboard {
        const DELTAS: [(i8, i8); 8] = [
            (0, 1),
            (1, 1),
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
        ];

        let mut bb = Bitboard::EMPTY;
        let mut i = 0;

        while i < DELTAS.len() {
            let (dx, dy) = DELTAS[i];

            if let Some(mv) = sq.try_offset(dx, dy) {
                bb.0 |= mv.bitboard().0;
            }

            i += 1;
        }

        bb
    }

    const TABLE: [Bitboard; Square::COUNT] = {
        let mut table = [Bitboard::EMPTY; Square::COUNT];
        let mut i = 0;
        while i < Square::COUNT {
            table[i] = calc_moves(Square::index(i));
            i += 1;
        }

        table
    };

    TABLE[sq as usize]
}

pub const fn king_zone(sq: Square, color: Color) -> Bitboard {
    const fn calc_zone(sq: Square, color: Color) -> Bitboard {
        let moves = Bitboard(king_moves(sq).0 | sq.bitboard().0);

        match color {
            Color::White => Bitboard(moves.0 | moves.shift::<North>(1).0),
            Color::Black => Bitboard(moves.0 | moves.shift::<South>(1).0),
        }
    }

    const TABLE: [[Bitboard; Square::COUNT]; Color::COUNT] = {
        let mut table = [[Bitboard::EMPTY; Square::COUNT]; Color::COUNT];
        let mut i = 0;

        while i < Color::COUNT {
            let mut j = 0;
            while j < Square::COUNT {
                table[i][j] = calc_zone(Square::index(j), Color::index(i));

                j += 1;
            }

            i += 1;
        }

        table
    };

    TABLE[color as usize][sq as usize]
}

/*----------------------------------------------------------------*/

#[inline]
pub const fn between(from: Square, to: Square) -> Bitboard {
    #[inline]
    const fn calc_between(from: Square, to: Square) -> Bitboard {
        let dx = to.file() as i8 - from.file() as i8;
        let dy = to.rank() as i8 - from.rank() as i8;

        let diag = dx.abs() == dy.abs();
        let orth = dx == 0 || dy == 0;

        if !(diag || orth) {
            return Bitboard::EMPTY;
        }

        let (dx, dy) = (dx.signum(), dy.signum());

        let mut bb = Bitboard::EMPTY;
        let mut sq = from.offset(dx, dy);

        while sq as u8 != to as u8 {
            bb.0 |= sq.bitboard().0;
            sq = sq.offset(dx, dy);
        }

        bb
    }

    const TABLE: [[Bitboard; Square::COUNT]; Square::COUNT] = {
        let mut table = [[Bitboard::EMPTY; Square::COUNT]; Square::COUNT];
        let mut i = 0;
        while i < Square::COUNT {
            let from = Square::index(i);
            let mut j = 0;
            while j < Square::COUNT {
                table[i][j] = calc_between(from, Square::index(j));
                j += 1;
            }

            i += 1;
        }

        table
    };

    TABLE[from as usize][to as usize]
}

#[inline]
pub const fn line(from: Square, to: Square) -> Bitboard {
    #[inline]
    const fn calc_line(from: Square, to: Square) -> Bitboard {
        let rays = bishop_rays(from);
        if rays.has(to) {
            return Bitboard((rays.0 | from.bitboard().0) & (bishop_rays(to).0 | to.bitboard().0));
        }

        let rays = rook_rays(from);
        if rays.has(to) {
            return Bitboard((rays.0 | from.bitboard().0) & (rook_rays(to).0 | to.bitboard().0));
        }

        Bitboard::EMPTY
    }

    const TABLE: [[Bitboard; Square::COUNT]; Square::COUNT] = {
        let mut table = [[Bitboard::EMPTY; Square::COUNT]; Square::COUNT];
        let mut i = 0;
        while i < Square::COUNT {
            let from = Square::index(i);
            let mut j = 0;
            while j < Square::COUNT {
                table[i][j] = calc_line(from, Square::index(j));
                j += 1;
            }

            i += 1;
        }

        table
    };

    TABLE[from as usize][to as usize]
}
