use crate::*;

/*----------------------------------------------------------------*/

#[inline(always)]
pub const fn pawn_quiets(sq: Square, color: Color, blockers: Bitboard) -> Bitboard {
    todo!()
}

#[inline(always)]
pub const fn pawn_attacks(sq: Square, color: Color) -> Bitboard {
    #[inline(always)]
    const fn calc_attacks(sq: Square, color: Color) -> Bitboard {
        if matches!(sq.rank(), Rank::First | Rank::Eighth) {
            return Bitboard::EMPTY;
        }
        
        let bb = sq.bitboard();
        match color {
            Color::White => Bitboard(bb.shift::<UpLeft>(1).0 | bb.shift::<UpRight>(1).0),
            Color::Black => Bitboard(bb.shift::<DownLeft>(1).0 | bb.shift::<DownRight>(1).0),
        }
    }
    
    const TABLE: [[Bitboard; Square::COUNT]; Color::COUNT] = {
        let mut table = [[Bitboard::EMPTY; Square::COUNT]; Color::COUNT];
        let mut i = 0;
        while i < Color::COUNT {
            let color = Color::index(i);
            let mut j = 0;
            while j < Square::COUNT {
                table[i][j] = calc_attacks(Square::index(i), color);
                j += 1;
            }
            
            i += 1;
        }
        
        table
    };
    
    TABLE[color as usize][sq as usize]
}

/*----------------------------------------------------------------*/

#[inline(always)]
pub const fn knight_moves(sq: Square) -> Bitboard {
    #[inline(always)]
    const fn calc_moves(sq: Square) -> Bitboard {
        const DELTAS: [(i8, i8); 8] = [
            (1, 2),   (2, 1),
            (2, -1),  (1, -2),
            (-1, -2), (-2, -1),
            (-2, 1),  (-1, 2)
        ];
        
        let mut bb = Bitboard::EMPTY;
        let mut i = 0;
        
        while i < DELTAS.len() {
            let (dx, dy) = DELTAS[i];
            
            if let Some(mv) = sq.try_offset(dx, dy) {
                bb.0 ^= mv.bitboard().0;
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

#[inline(always)]
pub const fn king_moves(sq: Square) -> Bitboard {
    #[inline(always)]
    const fn calc_moves(sq: Square) -> Bitboard {
        const DELTAS: [(i8, i8); 8] = [
            (0, 1),  (1, 1),
            (1, 0),  (1, -1),
            (0, -1), (-1, -1),
            (-1, 0), (-1, 1),
        ];

        let mut bb = Bitboard::EMPTY;
        let mut i = 0;

        while i < DELTAS.len() {
            let (dx, dy) = DELTAS[i];

            if let Some(mv) = sq.try_offset(dx, dy) {
                bb.0 ^= mv.bitboard().0;
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

#[inline(always)]
pub const fn bishop_moves(sq: Square, blockers: Bitboard) -> Bitboard {
    todo!()
}

#[inline(always)]
pub const fn bishop_rays(sq: Square) -> Bitboard {
    #[inline(always)]
    const fn calc_moves(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let sq = sq.bitboard();
        
        bb.0 |= sq.smear::<UpLeft>().0;
        bb.0 |= sq.smear::<UpRight>().0;
        bb.0 |= sq.smear::<DownLeft>().0;
        bb.0 |= sq.smear::<DownRight>().0;
        bb.0 &= !sq.0;
        
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

#[inline(always)]
pub const fn rook_moves(sq: Square, blockers: Bitboard) -> Bitboard {
    todo!()
}

#[inline(always)]
pub const fn rook_rays(sq: Square) -> Bitboard {
    #[inline(always)]
    const fn calc_moves(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let sq = sq.bitboard();

        bb.0 |= sq.smear::<Up>().0;
        bb.0 |= sq.smear::<Down>().0;
        bb.0 |= sq.smear::<Right>().0;
        bb.0 |= sq.smear::<Left>().0;
        bb.0 &= !sq.0;

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

#[inline(always)]
pub const fn queen_moves(sq: Square, blockers: Bitboard) -> Bitboard {
    Bitboard(bishop_moves(sq, blockers).0 | rook_moves(sq, blockers).0)
}

#[inline(always)]
pub const fn queen_rays(sq: Square) -> Bitboard {
    #[inline(always)]
    const fn calc_moves(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let sq = sq.bitboard();

        bb.0 |= sq.smear::<Up>().0;
        bb.0 |= sq.smear::<UpRight>().0;
        bb.0 |= sq.smear::<UpLeft>().0;
        bb.0 |= sq.smear::<Down>().0;
        bb.0 |= sq.smear::<DownRight>().0;
        bb.0 |= sq.smear::<DownLeft>().0;
        bb.0 |= sq.smear::<Right>().0;
        bb.0 |= sq.smear::<Left>().0;
        bb.0 &= !sq.0;
        
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

#[inline(always)]
pub const fn between(from: Square, to: Square) -> Bitboard {
    #[inline(always)]
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

#[inline(always)]
pub const fn line(from: Square, to: Square) -> Bitboard {
    #[inline(always)]
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