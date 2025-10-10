use crate::*;

/*----------------------------------------------------------------*/

#[inline]
const fn expand_sq(sq: Square) -> u8 {
    sq as u8 + (sq as u8 & 0b111000)
}

#[inline]
#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
fn compress_coords_128(coords: Vec128) -> (Vec128, Vec128Mask8) {
    let valid = Vec128::testn8(coords, Vec128::splat8(0x88));
    let compressed = Vec128::sub8(coords, Vec128::shr16::<1>(coords & Vec128::splat8(0b0111_000)));

    (compressed, valid)
}

#[inline]
#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
fn compress_coords_512(coords: Vec512) -> (Vec512, Vec512Mask8) {
    let valid = Vec512::testn8(coords, Vec512::splat8(0x88));
    let compressed = Vec512::sub8(coords, Vec512::shr16::<1>(coords & Vec512::splat8(0b0111_000)));

    (compressed, valid)
}

#[inline]
#[cfg(target_feature = "avx512f")]
fn compress_coords_128(coords: Vec128) -> (Vec128, Vec128Mask8) {
    let valid = Vec128::testn8(coords, Vec128::splat8(0x88));
    let compressed = Vec128::gf2p8matmul8(coords, Vec128::splat64(0x0102041020400000));

    (compressed, valid)
}

#[inline]
#[cfg(target_feature = "avx512f")]
fn compress_coords_512(coords: Vec512) -> (Vec512, Vec512Mask8) {
    let valid = Vec512::testn8(coords, Vec512::splat8(0x88));
    let compressed = Vec512::gf2p8matmul8(coords, Vec512::splat64(0x0102041020400000));

    (compressed, valid)
}

/*----------------------------------------------------------------*/

#[inline]
pub(crate) fn superpiece_rays(sq: Square) -> (Vec512, Vec512Mask8) {
    let offsets = Vec512::from([
        0x1F, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, // N
        0x21, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // NE
        0x12, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // E
        0xF2, 0xF1, 0xE2, 0xD3, 0xC4, 0xB5, 0xA6, 0x97, // SE
        0xE1, 0xF0, 0xE0, 0xD0, 0xC0, 0xB0, 0xA0, 0x90, // S
        0xDF, 0xEF, 0xDE, 0xCD, 0xBC, 0xAB, 0x9A, 0x89, // SW
        0xEE, 0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, // W
        0x0E, 0x0F, 0x1E, 0x2D, 0x3C, 0x4B, 0x5A, 0x69, // NW
    ]);
    let uncompressed = Vec512::add8(Vec512::splat8(expand_sq(sq)), offsets);

    compress_coords_512(uncompressed)
}

#[inline]
pub(crate) fn superpiece_attacks(occ: u64, ray_valid: u64) -> u64 {
    let o = occ | 0x8181818181818181;
    let x = o ^ o.wrapping_sub(0x0303030303030303);

    x & ray_valid
}

/*----------------------------------------------------------------*/

#[inline]
pub(crate) fn adjacents(sq: Square) -> (Vec128, Vec128Mask8) {
    let offsets = Vec128::from([
        0x10, 0x11, 0x01, 0xF1, 0xF0, 0xEF, 0xFF, 0x0F,
        0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88
    ]);
    let uncompressed = Vec128::add8(Vec128::splat8(expand_sq(sq)), offsets);

    compress_coords_128(uncompressed)
}

/*----------------------------------------------------------------*/

pub(crate) const NON_HORSE_ATTACK_MASK: u64 = 0xFEFEFEFEFEFEFEFE;

#[inline]
fn slider_mask() -> Vec512 {
    const DIAG: u8 = 0b001 << 4;
    const ORTH: u8 = 0b010 << 4;

    Vec512::from([
        0, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        0, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        0, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        0, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        0, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        0, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        0, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        0, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
    ])
}

#[inline]
pub(crate) fn sliders_from_rays(rays: Vec512) -> u64 {
    (rays & Vec512::splat8(0b100 << 4)).nonzero8() & (rays & slider_mask()).nonzero8()
}

/*----------------------------------------------------------------*/

const ATTACK_MASK_TABLE: [u64; 16] = {
    #[inline]
    const fn white_piece(piece: Piece) -> u16 {
        1u16 << piece.bits()
    }

    #[inline]
    const fn black_piece(piece: Piece) -> u16 {
        0x100u16 << piece.bits()
    }

    #[inline]
    const fn both_pieces(piece: Piece) -> u16 {
        white_piece(piece) | black_piece(piece)
    }

    const KING: u16 = both_pieces(Piece::King);
    const WHITE_PAWN: u16 = white_piece(Piece::Pawn);
    const BLACK_PAWN: u16 = black_piece(Piece::Pawn);
    const KNIGHT: u16 = both_pieces(Piece::Knight);
    const BISHOP: u16 = both_pieces(Piece::Bishop);
    const ROOK: u16 = both_pieces(Piece::Rook);
    const QUEEN: u16 = both_pieces(Piece::Queen);

    const DIAG: u16 = BISHOP | QUEEN;
    const ORTH: u16 = ROOK | QUEEN;
    const OADJ: u16 = ROOK | QUEEN | KING;
    const WPDJ: u16 = BISHOP | QUEEN | KING | WHITE_PAWN;
    const BPDJ: u16 = BISHOP | QUEEN | KING | BLACK_PAWN;

    const BASE: [u16; 64] = [
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, WPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, BPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, BPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, WPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
    ];

    let mut table = [0u64; 16];
    let mut pt = 0;

    while pt < 16 {
        let pt_mask = 1u16 << pt;

        let mut sq = 0;
        while sq < Square::COUNT {
            if (BASE[sq] & pt_mask) != 0 {
                table[pt] |= 1u64 << sq;
            }

            sq += 1;
        }

        pt += 1;
    }

    table
};

#[inline]
pub(crate) const fn attack_mask(piece: Piece, color: Color) -> u64 {
    ATTACK_MASK_TABLE[((color as usize) << 3) | piece.bits() as usize]
}

/*----------------------------------------------------------------*/

#[inline]
pub(crate) fn attackers_from_rays(rays: Vec512) -> u64 {
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

    let piece_to_bits = Vec128::from([
        0, KING, BLACK_PAWN, KNIGHT, 0, BISHOP, ROOK, QUEEN,
        0, KING, WHITE_PAWN, KNIGHT, 0, BISHOP, ROOK, QUEEN,
    ]);
    let base = Vec512::from([
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, WPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, BPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, BPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
        KNIGHT, OADJ, ORTH, ORTH, ORTH, ORTH, ORTH, ORTH,
        KNIGHT, WPDJ, DIAG, DIAG, DIAG, DIAG, DIAG, DIAG,
    ]);

    let bit_rays = Vec512::permute8_128(Vec512::shr16::<4>(rays) & Vec512::splat8(0x0F), piece_to_bits);
    (bit_rays & base).nonzero8()
}

/*----------------------------------------------------------------*/

const SUPERPIECE_INV_RAYS: [[u8; 64]; Square::COUNT] = {
    const NONE: u8 = 0xFF;
    const BASE: [u8; 256] = [
        NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
        NONE, 0x2F, NONE, NONE, NONE, NONE, NONE, NONE, 0x27, NONE, NONE, NONE, NONE, NONE, NONE, 0x1F,
        NONE, NONE, 0x2E, NONE, NONE, NONE, NONE, NONE, 0x26, NONE, NONE, NONE, NONE, NONE, 0x1E, NONE,
        NONE, NONE, NONE, 0x2D, NONE, NONE, NONE, NONE, 0x25, NONE, NONE, NONE, NONE, 0x1D, NONE, NONE,
        NONE, NONE, NONE, NONE, 0x2C, NONE, NONE, NONE, 0x24, NONE, NONE, NONE, 0x1C, NONE, NONE, NONE,
        NONE, NONE, NONE, NONE, NONE, 0x2B, NONE, NONE, 0x23, NONE, NONE, 0x1B, NONE, NONE, NONE, NONE,
        NONE, NONE, NONE, NONE, NONE, NONE, 0x2A, 0x28, 0x22, 0x20, 0x1A, NONE, NONE, NONE, NONE, NONE,
        NONE, NONE, NONE, NONE, NONE, NONE, 0x30, 0x29, 0x21, 0x19, 0x18, NONE, NONE, NONE, NONE, NONE,
        NONE, 0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, NONE, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        NONE, NONE, NONE, NONE, NONE, NONE, 0x38, 0x39, 0x01, 0x09, 0x10, NONE, NONE, NONE, NONE, NONE,
        NONE, NONE, NONE, NONE, NONE, NONE, 0x3A, 0x00, 0x02, 0x08, 0x0A, NONE, NONE, NONE, NONE, NONE,
        NONE, NONE, NONE, NONE, NONE, 0x3B, NONE, NONE, 0x03, NONE, NONE, 0x0B, NONE, NONE, NONE, NONE,
        NONE, NONE, NONE, NONE, 0x3C, NONE, NONE, NONE, 0x04, NONE, NONE, NONE, 0x0C, NONE, NONE, NONE,
        NONE, NONE, NONE, 0x3D, NONE, NONE, NONE, NONE, 0x05, NONE, NONE, NONE, NONE, 0x0D, NONE, NONE,
        NONE, NONE, 0x3E, NONE, NONE, NONE, NONE, NONE, 0x06, NONE, NONE, NONE, NONE, NONE, 0x0E, NONE,
        NONE, 0x3F, NONE, NONE, NONE, NONE, NONE, NONE, 0x07, NONE, NONE, NONE, NONE, NONE, NONE, 0x0F,
    ];
    const OFFSETS: [u8; 64] = [
        0o210, 0o211, 0o212, 0o213, 0o214, 0o215, 0o216, 0o217,
        0o230, 0o231, 0o232, 0o233, 0o234, 0o235, 0o236, 0o237,
        0o250, 0o251, 0o252, 0o253, 0o254, 0o255, 0o256, 0o257,
        0o270, 0o271, 0o272, 0o273, 0o274, 0o275, 0o276, 0o277,
        0o310, 0o311, 0o312, 0o313, 0o314, 0o315, 0o316, 0o317,
        0o330, 0o331, 0o332, 0o333, 0o334, 0o335, 0o336, 0o337,
        0o350, 0o351, 0o352, 0o353, 0o354, 0o355, 0o356, 0o357,
        0o370, 0o371, 0o372, 0o373, 0o374, 0o375, 0o376, 0o377,
    ];

    let mut table = [[0u8; 64]; Square::COUNT];
    let mut sq = 0;
    while sq < Square::COUNT {
        let esq = expand_sq(Square::index(sq));

        let mut i = 0;
        while i < 64 {
            table[sq][i] = BASE[(OFFSETS[i] - esq) as usize];
            i += 1;
        }

        sq += 1;
    }

    table
};

const SUPERPIECE_INV_RAYS_SWAPPED: [[u8; 64]; Square::COUNT] = {
    const NONE: u8 = 0xFF;

    let mut table = SUPERPIECE_INV_RAYS;
    let mut sq = 0;
    while sq < Square::COUNT {
        let mut i = 0;
        while i < 64 {
            let j = table[sq][i];
            table[sq][i] = if j != NONE {
                (j + 32) % 64
            } else {
                NONE
            };

            i += 1;
        }

        sq += 1;
    }

    table
};

#[inline]
pub(crate) fn superpiece_inv_rays(sq: Square) -> Vec512 {
    Vec512::from(SUPERPIECE_INV_RAYS[sq])
}

#[inline]
pub(crate) fn superpiece_inv_rays_swapped(sq: Square) -> Vec512 {
    Vec512::from(SUPERPIECE_INV_RAYS_SWAPPED[sq])
}