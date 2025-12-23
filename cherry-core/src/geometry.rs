use crate::*;

/*----------------------------------------------------------------*/

const BETWEEN: [[Bitboard; Square::COUNT]; Square::COUNT] = {
    #[inline]
    const fn calc_between(src: Square, dest: Square) -> Bitboard {
        let dx = dest.file() as i8 - src.file() as i8;
        let dy = dest.rank() as i8 - src.rank() as i8;

        let diag = dx.abs() == dy.abs();
        let orth = dx == 0 || dy == 0;

        if !(diag ^ orth) {
            return Bitboard::EMPTY;
        }

        let (dx, dy) = (dx.signum(), dy.signum());

        let mut bb = Bitboard::EMPTY;
        let mut sq = src.offset(dx, dy);
        while sq as u8 != dest as u8 {
            bb.0 |= sq.bitboard().0;
            sq = sq.offset(dx, dy);
        }

        bb
    }

    let mut table = [[Bitboard::EMPTY; Square::COUNT]; Square::COUNT];
    let mut sq1 = 0;
    while sq1 < Square::COUNT {
        let mut sq2 = 0;
        while sq2 < Square::COUNT {
            table[sq1][sq2] = calc_between(Square::index(sq1), Square::index(sq2));
            sq2 += 1;
        }
        sq1 += 1;
    }

    table
};


const LINE: [[Bitboard; Square::COUNT]; Square::COUNT] = {
    #[inline]
    const fn calc_line(src: Square, dest: Square) -> Bitboard {
        let dx = dest.file() as i8 - src.file() as i8;
        let dy = dest.rank() as i8 - src.rank() as i8;

        let diag = dx.abs() == dy.abs();
        let orth = dx == 0 || dy == 0;

        if !(diag ^ orth) {
            return Bitboard::EMPTY;
        }

        let (dx, dy) = (dx.signum(), dy.signum());

        let mut bb = Bitboard::EMPTY;
        let mut next = src.try_offset(dx, dy);
        while let Some(sq) = next {
            bb.0 |= sq.bitboard().0;
            next = sq.try_offset(dx, dy);
        }

        bb
    }

    let mut table = [[Bitboard::EMPTY; Square::COUNT]; Square::COUNT];
    let mut sq1 = 0;
    while sq1 < Square::COUNT {
        let mut sq2 = 0;
        while sq2 < Square::COUNT {
            table[sq1][sq2] = calc_line(Square::index(sq1), Square::index(sq2));
            sq2 += 1;
        }
        sq1 += 1;
    }

    table
};


const RAY_PERMS: [[u8; Square::COUNT]; Square::COUNT] = {
    #[inline]
    const fn calc_perm(sq: Square) -> [u8; Square::COUNT] {
        const SLIDER_DIRS: &[(i8, i8); 8] = &[
            (North::DX, North::DY),
            (NorthEast::DX, NorthEast::DY),
            (East::DX, East::DY),
            (SouthEast::DX, SouthEast::DY),
            (South::DX, South::DY),
            (SouthWest::DX, SouthWest::DY),
            (West::DX, West::DY),
            (NorthWest::DX, NorthWest::DY),
        ];
        const KNIGHT_JUMPS: &[(i8, i8); 8] = &[
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ];

        let mut perm = [0x88; Square::COUNT];
        let mut dir = 0;
        while dir < 8 {
            let (dx, dy) = SLIDER_DIRS[dir];
            let mut i = 1;
            while i < 8 {
                perm[dir * 8 + i as usize] = if let Some(sq) = sq.try_offset(dx * i, dy * i) {
                    sq as u8
                } else {
                    0x88
                };

                i += 1;
            }

            let (dx, dy) = KNIGHT_JUMPS[dir];
            perm[dir * 8] = if let Some(sq) = sq.try_offset(dx, dy) {
                sq as u8
            } else {
                0x88
            };

            dir += 1;
        }

        perm
    }

    let mut table = [[0; Square::COUNT]; Square::COUNT];
    let mut sq = 0;
    while sq < Square::COUNT {
        table[sq] = calc_perm(Square::index(sq));
        sq += 1;
    }

    table
};

const RAY_VALID: [u64; Square::COUNT] = {
    #[inline]
    const fn calc_valid(sq: Square) -> u64 {
        let ray_perm = RAY_PERMS[sq as usize];
        let mut valid = 0;
        let mut sq = 0;
        while sq < Square::COUNT {
            if ray_perm[sq] != 0x88 {
                valid |= 1u64 << sq;
            }
            sq += 1;
        }

        valid
    }

    let mut table = [0; Square::COUNT];
    let mut sq = 0;
    while sq < Square::COUNT {
        table[sq] = calc_valid(Square::index(sq));
        sq += 1;
    }

    table
};

const INV_PERMS: [[u8; Square::COUNT]; Square::COUNT] = {
    #[inline]
    const fn calc_inv_perm(sq: Square) -> [u8; Square::COUNT] {
        let ray_perm = RAY_PERMS[sq as usize];
        let mut inv_perm = [0x88; Square::COUNT];
        let mut sq = 0;
        while sq < Square::COUNT {
            let perm_sq = ray_perm[sq];
            if perm_sq != 0x88 {
                inv_perm[perm_sq as usize] = sq as u8;
            }

            sq += 1;
        }

        inv_perm
    }

    let mut table = [[0; Square::COUNT]; Square::COUNT];
    let mut sq = 0;
    while sq < Square::COUNT {
        table[sq] = calc_inv_perm(Square::index(sq));
        sq += 1;
    }

    table
};

const INV_VALID: [u64; Square::COUNT] = {
    #[inline]
    const fn calc_valid(sq: Square) -> u64 {
        let ray_perm = INV_PERMS[sq as usize];
        let mut valid = 0;
        let mut sq = 0;
        while sq < Square::COUNT {
            if ray_perm[sq] != 0x88 {
                valid |= 1u64 << sq;
            }
            sq += 1;
        }

        valid
    }

    let mut table = [0; Square::COUNT];
    let mut sq = 0;
    while sq < Square::COUNT {
        table[sq] = calc_valid(Square::index(sq));
        sq += 1;
    }

    table
};

const ATTACK_MASKS: [[u64; Piece::COUNT]; Color::COUNT] = {
    const KING: u8 = 1 << 0;
    const WHITE_PAWN: u8 = 1 << 1;
    const BLACK_PAWN: u8 = 1 << 2;
    const KNIGHT: u8 = 1 << 3;
    const BISHOP: u8 = 1 << 4;
    const ROOK: u8 = 1 << 5;
    const QUEEN: u8 = 1 << 6;

    const DIAG: u8 = BISHOP | QUEEN;
    const ORTH: u8 = ROOK | QUEEN;
    const OADJ: u8 = ROOK | QUEEN | KING;
    const WPDJ: u8 = BISHOP | QUEEN | KING | WHITE_PAWN;
    const BPDJ: u8 = BISHOP | QUEEN | KING | BLACK_PAWN;

    #[rustfmt::skipo]
    const BASE: [u8; 64] = [
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, WPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, BPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, BPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, WPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
    ];

    let mut table = [[0; Piece::COUNT]; Color::COUNT];
    let mut color = 0;

    while color < Color::COUNT {
        let mut piece = 0;
        while piece < Piece::COUNT {
            let mut sq = 0;
            while sq < Square::COUNT {
                let mask = match piece {
                    0 =>
                        if color == 0 {
                            WHITE_PAWN
                        } else {
                            BLACK_PAWN
                        },
                    1 => KNIGHT,
                    2 => BISHOP,
                    3 => ROOK,
                    4 => QUEEN,
                    5 => KING,
                    _ => unreachable!(),
                };

                if (BASE[sq] & mask) != 0 {
                    table[color][piece] |= 1u64 << sq;
                }

                sq += 1;
            }

            piece += 1;
        }

        color += 1;
    }

    table
};

pub const NON_HORSE_ATTACK_MASK: u64 = 0xFEFEFEFEFEFEFEFE;

/*----------------------------------------------------------------*/

#[inline]
pub const fn between(from: Square, to: Square) -> Bitboard {
    BETWEEN[from as usize][to as usize]
}

#[inline]
pub const fn line(from: Square, to: Square) -> Bitboard {
    LINE[from as usize][to as usize]
}

#[inline]
pub fn ray_perm(sq: Square) -> (u8x64, u64) {
    (u8x64::from(RAY_PERMS[sq as usize]), RAY_VALID[sq as usize])
}

#[inline]
pub fn inv_perm(sq: Square) -> (u8x64, u64) {
    (u8x64::from(INV_PERMS[sq as usize]), INV_VALID[sq as usize])
}

#[inline]
pub const fn extend_bitrays(blockers: u64, valid: u64) -> u64 {
    let o = blockers | 0x8181818181818181;
    let x = o ^ o.wrapping_sub(0x0303030303030303);

    x & valid
}

#[inline]
pub const fn attack_mask(color: Color, piece: Piece) -> u64 {
    ATTACK_MASKS[color as usize][piece as usize]
}

#[inline]
pub fn ray_attackers(rays: u8x64) -> Mask64 {
    const KING: u8 = 1 << 0;
    const WHITE_PAWN: u8 = 1 << 1;
    const BLACK_PAWN: u8 = 1 << 2;
    const KNIGHT: u8 = 1 << 3;
    const BISHOP: u8 = 1 << 4;
    const ROOK: u8 = 1 << 5;
    const QUEEN: u8 = 1 << 6;

    const DIAG: u8 = BISHOP | QUEEN;
    const ORTH: u8 = ROOK | QUEEN;
    const OADJ: u8 = ROOK | QUEEN | KING;
    const WPDJ: u8 = BISHOP | QUEEN | KING | WHITE_PAWN;
    const BPDJ: u8 = BISHOP | QUEEN | KING | BLACK_PAWN;

    #[rustfmt::skip]
    let bits_to_piece = u8x16::from([
        0, KING, BLACK_PAWN, KNIGHT, 0, BISHOP, ROOK, QUEEN,
        0, KING, WHITE_PAWN, KNIGHT, 0, BISHOP, ROOK, QUEEN,
    ])
    .broadcast64();

    #[rustfmt::skip]
    let valid_attackers = u8x64::from([
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, WPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, BPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, BPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, WPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
    ]);

    let ray_pieces = (rays & u8x64::splat(Place::PIECE_MASK | Place::COLOR_MASK)).to_u16x32().shr::<4>().to_u8x64();
    let ray_pieces = bits_to_piece.shuffle(ray_pieces);

    (ray_pieces & valid_attackers).nonzero()
}

#[inline]
pub fn ray_sliders(rays: u8x64) -> Mask64 {
    const DIAG: u8 = 0b001 << 4;
    const ORTH: u8 = 0b010 << 4;

    #[rustfmt::skip]
    let slider_mask = u8x64::from([
        0, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        0, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        0, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        0, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        0, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        0, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        0, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        0, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
    ]);

    (rays & u8x64::splat(Place::SLIDER_BIT)).nonzero() & (rays & slider_mask).nonzero()
}
