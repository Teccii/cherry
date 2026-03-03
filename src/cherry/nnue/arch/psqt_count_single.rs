use std::{
    iter::Iterator,
    num::{NonZeroU8, NonZeroU16},
    ops::Index,
};

use arrayvec::ArrayVec;

use crate::*;

pub const INPUT: usize = 798;
pub const HL: usize = 1024;
pub const HORIZONTAL_MIRRORING: bool = true;
pub const PAIRWISE_MUL: bool = false;

pub const NUM_OUTPUT_BUCKETS: usize = 8;
#[rustfmt::skip]
pub const OUTPUT_BUCKETS: [usize; 32] = [
    0, 0, 0, 0, // 1,  2,  3,  4,
    0, 0, 0,    // 5,  6,  7,
    0, 0, 0,    // 8,  9,  10,
    1, 1, 1,    // 11, 12, 13,
    2, 2, 2,    // 14, 15, 16,
    3, 3, 3,    // 17, 18, 19,
    4, 4, 4,    // 20, 21, 22,
    5, 5, 5,    // 23, 24, 25,
    6, 6, 6,    // 26, 27, 28,
    7, 7, 7, 7, // 29, 30, 31, 32
];

pub const QA: i32 = 255;
pub const QB: i32 = 64;

#[derive(Debug, Clone)]
#[repr(C, align(64))]
pub struct NetworkWeights {
    pub ft_weights: [i16; INPUT * HL],
    pub ft_bias: [i16; HL],
    pub out_weights: [[i16; HL * (2 - PAIRWISE_MUL as usize)]; NUM_OUTPUT_BUCKETS],
    pub out_bias: [i16; NUM_OUTPUT_BUCKETS],
}

/*----------------------------------------------------------------*/

/*
Bit Layout:
- Bit    0: Color
- Bits 1-3: Piece
- Bits 4-9: Square
*/
#[derive(Debug, Copy, Clone)]
pub struct PieceFeature(NonZeroU16);

impl PieceFeature {
    #[inline]
    pub fn new(piece: Piece, color: Color, sq: Square) -> Option<PieceFeature> {
        let mut bits = 0;
        bits |= color as u16;
        bits |= (piece as u16) << 1;
        bits |= (sq as u16) << 4;

        Some(PieceFeature(unsafe { NonZeroU16::new_unchecked(bits) }))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn piece(self) -> Piece {
        Piece::index(((self.0.get() >> 1) & 7) as usize)
    }

    #[inline]
    pub fn color(self) -> Color {
        Color::index((self.0.get() & 1) as usize)
    }

    #[inline]
    pub fn square(self) -> Square {
        Square::index(((self.0.get() >> 4) & 63) as usize)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn to_index(self, king: Square, perspective: Color) -> usize {
        let (piece, color, sq) = (self.piece(), self.color(), self.square());
        let (mut sq, color) = match perspective {
            Color::White => (sq, color),
            Color::Black => (sq.flip_rank(), !color),
        };

        if HORIZONTAL_MIRRORING && king.file() > File::D {
            sq = sq.flip_file();
        }

        Self::COLOR_OFFSET * color as usize + Self::PIECE_OFFSET * piece as usize + sq as usize
    }

    pub const COLOR_OFFSET: usize = Square::COUNT * Piece::COUNT;
    pub const PIECE_OFFSET: usize = Square::COUNT;
    pub const TOTAL: usize = 768;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CountPiece {
    Pawn,
    Knight,
    LightBishop,
    DarkBishop,
    Rook,
    Queen,
}

impl CountPiece {
    #[inline]
    pub fn new(piece: Piece, sq: Square) -> Option<CountPiece> {
        use CountPiece::*;
        match piece {
            Piece::Pawn => Some(Pawn),
            Piece::Knight => Some(Knight),
            Piece::Bishop =>
                if Bitboard::LIGHT_SQUARES.has(sq) {
                    Some(LightBishop)
                } else {
                    Some(DarkBishop)
                },
            Piece::Rook => Some(Rook),
            Piece::Queen => Some(Queen),
            Piece::King => None,
        }
    }

    #[inline]
    pub const fn index(i: usize) -> CountPiece {
        if i < CountPiece::COUNT {
            return unsafe { core::mem::transmute::<u8, CountPiece>(i as u8) };
        }

        panic!("CountPiece::index(): Index out of bounds");
    }

    pub const ALL: [CountPiece; Self::COUNT] = [
        CountPiece::Pawn,
        CountPiece::Knight,
        CountPiece::LightBishop,
        CountPiece::DarkBishop,
        CountPiece::Rook,
        CountPiece::Queen,
    ];
    pub const COUNT: usize = 6;
}

impl<T> Index<CountPiece> for [T; CountPiece::COUNT] {
    type Output = T;

    #[inline]
    fn index(&self, index: CountPiece) -> &Self::Output {
        self.index(index as usize)
    }
}

/*
Bit Layout:
- Bit    0: Always On
- Bit    1: Color
- Bits 2-4: Piece (Light and Dark Bishops are separate)
- Bits 5-7: Offset
*/
#[derive(Debug, Copy, Clone)]
pub struct CountFeature(NonZeroU8);

impl CountFeature {
    #[inline]
    pub fn new(board: &Board, piece: Piece, color: Color, sq: Square) -> Option<CountFeature> {
        const LIGHT: Bitboard = Bitboard::LIGHT_SQUARES;
        const DARK: Bitboard = Bitboard::DARK_SQUARES;
        use CountPiece::*;

        let count_piece = CountPiece::new(piece, sq)?;

        let pieces = board.color_pieces(color, piece);
        let offset = match count_piece {
            Pawn | Knight | Rook | Queen => pieces.popcnt() - 1,
            LightBishop => (pieces & LIGHT).popcnt() - 1,
            DarkBishop => (pieces & DARK).popcnt() - 1,
        };

        if offset >= Self::MAX_COUNT[count_piece] {
            return None;
        }

        Some(CountFeature::from_parts(count_piece, color, offset))
    }

    #[inline]
    pub fn from_parts(piece: CountPiece, color: Color, offset: usize) -> CountFeature {
        let mut bits = 1;
        bits |= (color as u8) << 1;
        bits |= (piece as u8) << 2;
        bits |= (offset as u8) << 5;

        CountFeature(unsafe { NonZeroU8::new_unchecked(bits) })
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn color(self) -> Color {
        Color::index(((self.0.get() >> 1) & 1) as usize)
    }

    #[inline]
    pub fn piece(self) -> CountPiece {
        CountPiece::index(((self.0.get() >> 2) & 7) as usize)
    }

    #[inline]
    pub fn offset(self) -> usize {
        ((self.0.get() >> 5) & 7) as usize
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn to_index(self, perspective: Color) -> usize {
        let (piece, color, offset) = (self.piece(), self.color(), self.offset());
        let color = match perspective {
            Color::White => color,
            Color::Black => !color,
        };

        Self::PIECE_OFFSET[piece] + Self::COLOR_OFFSET * color as usize + offset
    }

    /*----------------------------------------------------------------*/

    const MAX_COUNT: [usize; CountPiece::COUNT] = [8, 2, 1, 1, 2, 1];
    const PIECE_OFFSET: [usize; CountPiece::COUNT] = {
        let mut table = [0; CountPiece::COUNT];
        let mut current = PieceFeature::TOTAL;
        let mut i = 0;

        while i < CountPiece::COUNT {
            table[i] = current;
            current += Self::MAX_COUNT[i];
            i += 1;
        }

        table
    };
    const COLOR_OFFSET: usize = {
        let mut count = 0;
        let mut i = 0;
        while i < CountPiece::COUNT {
            count += Self::MAX_COUNT[i];
            i += 1;
        }

        count
    };

    pub const TOTAL: usize = Self::COLOR_OFFSET * 2;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, Default)]
pub struct FeatureUpdate {
    pub piece_add: Option<PieceFeature>,
    pub piece_add2: Option<PieceFeature>,
    pub piece_sub: Option<PieceFeature>,
    pub piece_sub2: Option<PieceFeature>,
    pub count_add: Option<CountFeature>,
    pub count_sub: Option<CountFeature>,
    pub count_sub2: Option<CountFeature>,
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Accumulator {
    pub white: [i16; HL],
    pub black: [i16; HL],
    pub dirty: [bool; Color::COUNT],
    pub update: FeatureUpdate,
}

impl Accumulator {
    #[inline]
    pub fn select(&self, color: Color) -> &[i16; HL] {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    #[inline]
    pub fn select_mut(&mut self, color: Color) -> &mut [i16; HL] {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn extrapolate(&mut self, prev: &Accumulator, king: Square, perspective: Color) {
        let adds = [
            prev.update.piece_add.map(|f| f.to_index(king, perspective)),
            prev.update
                .piece_add2
                .map(|f| f.to_index(king, perspective)),
            prev.update.count_add.map(|f| f.to_index(perspective)),
        ];
        let subs = [
            prev.update.piece_sub.map(|f| f.to_index(king, perspective)),
            prev.update
                .piece_sub2
                .map(|f| f.to_index(king, perspective)),
            prev.update.count_sub.map(|f| f.to_index(perspective)),
            prev.update.count_sub2.map(|f| f.to_index(perspective)),
        ];

        match (adds, subs) {
            ([Some(add), Some(add2), None], [Some(sub), Some(sub2), None, None]) => acc_add2_sub2(
                prev.select(perspective),
                self.select_mut(perspective),
                add,
                add2,
                sub,
                sub2,
            ),
            ([Some(add), _, Some(add2)], [Some(sub), _, Some(sub2), None]) => acc_add2_sub2(
                prev.select(perspective),
                self.select_mut(perspective),
                add,
                add2,
                sub,
                sub2,
            ),
            ([Some(add), _, Some(add2)], [Some(sub), Some(sub2), Some(sub3), Some(sub4)]) =>
                acc_add2_sub4(
                    prev.select(perspective),
                    self.select_mut(perspective),
                    add,
                    add2,
                    sub,
                    sub2,
                    sub3,
                    sub4,
                ),
            ([Some(add), _, _], [Some(sub), Some(sub2), Some(sub3), None]) => acc_add_sub3(
                prev.select(perspective),
                self.select_mut(perspective),
                add,
                sub,
                sub2,
                sub3,
            ),
            ([Some(add), _, _], [Some(sub), _, _, _]) => acc_add_sub(
                prev.select(perspective),
                self.select_mut(perspective),
                add,
                sub,
            ),
            _ => unreachable!(),
        }

        self.dirty[perspective] = false;
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn reset(&mut self, board: &Board, perspective: Color) {
        let acc = self.select_mut(perspective);
        acc.copy_from_slice(&NETWORK.ft_bias);

        let king = board.king(perspective);
        let piece_updates: ArrayVec<usize, 32> = board
            .occupied()
            .iter()
            .map(|sq| {
                PieceFeature::new(board.piece_on(sq).unwrap(), board.color_on(sq).unwrap(), sq)
                    .unwrap()
                    .to_index(king, perspective)
            })
            .collect();
        let count_updates: ArrayVec<usize, { CountFeature::TOTAL }> =
            CountPiece::ALL
                .iter()
                .fold(ArrayVec::new(), |mut vec, &count_piece| {
                    const LIGHT: Bitboard = Bitboard::LIGHT_SQUARES;
                    const DARK: Bitboard = Bitboard::DARK_SQUARES;
                    use CountPiece::*;

                    let piece = match count_piece {
                        Pawn => Piece::Pawn,
                        Knight => Piece::Knight,
                        LightBishop => Piece::Bishop,
                        DarkBishop => Piece::Bishop,
                        Rook => Piece::Rook,
                        Queen => Piece::Queen,
                    };

                    for &color in &Color::ALL {
                        let pieces = board.color_pieces(color, piece);
                        let count = match count_piece {
                            Pawn | Knight | Rook | Queen => pieces.popcnt(),
                            LightBishop => (pieces & LIGHT).popcnt(),
                            DarkBishop => (pieces & DARK).popcnt(),
                        };

                        for i in (0..count).take(CountFeature::MAX_COUNT[count_piece]) {
                            vec.push(
                                CountFeature::from_parts(count_piece, color, i)
                                    .to_index(perspective),
                            );
                        }
                    }

                    vec
                });

        let ft_weights = &NETWORK.ft_weights;
        for i in 0..(HL / 32) {
            let offset = i * 32;

            unsafe {
                let mut value = i16x32::load(acc.as_ptr().add(offset));
                for &index in piece_updates.iter().chain(count_updates.iter()) {
                    value += i16x32::load(ft_weights.as_ptr().add(index * HL + offset));
                }

                value.store(acc.as_mut_ptr().add(offset));
            }
        }

        self.dirty[perspective] = false;
    }
}

impl Default for Accumulator {
    #[inline]
    fn default() -> Self {
        Accumulator {
            white: [0; HL],
            black: [0; HL],
            dirty: [false; Color::COUNT],
            update: FeatureUpdate::default(),
        }
    }
}

#[inline]
fn acc_add_sub(input: &[i16; HL], output: &mut [i16; HL], add: usize, sub: usize) {
    let ft_weights = &NETWORK.ft_weights;
    for i in 0..(HL / 32) {
        let offset = i * 32;

        unsafe {
            let mut value = i16x32::load(input.as_ptr().add(offset));
            value += i16x32::load(ft_weights.as_ptr().add(add * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub * HL + offset));

            value.store(output.as_mut_ptr().add(offset));
        }
    }
}

#[inline]
fn acc_add_sub3(
    input: &[i16; HL],
    output: &mut [i16; HL],
    add: usize,
    sub1: usize,
    sub2: usize,
    sub3: usize,
) {
    let ft_weights = &NETWORK.ft_weights;
    for i in 0..(HL / 32) {
        let offset = i * 32;

        unsafe {
            let mut value = i16x32::load(input.as_ptr().add(offset));
            value += i16x32::load(ft_weights.as_ptr().add(add * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub1 * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub2 * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub3 * HL + offset));

            value.store(output.as_mut_ptr().add(offset));
        }
    }
}

#[inline]
fn acc_add2_sub2(
    input: &[i16; HL],
    output: &mut [i16; HL],
    add1: usize,
    add2: usize,
    sub1: usize,
    sub2: usize,
) {
    let ft_weights = NETWORK.ft_weights;
    for i in 0..(HL / 32) {
        let offset = i * 32;

        unsafe {
            let mut value = i16x32::load(input.as_ptr().add(offset));
            value += i16x32::load(ft_weights.as_ptr().add(add1 * HL + offset));
            value += i16x32::load(ft_weights.as_ptr().add(add2 * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub1 * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub2 * HL + offset));

            value.store(output.as_mut_ptr().add(offset));
        }
    }
}

#[inline]
fn acc_add2_sub4(
    input: &[i16; HL],
    output: &mut [i16; HL],
    add1: usize,
    add2: usize,
    sub1: usize,
    sub2: usize,
    sub3: usize,
    sub4: usize,
) {
    let ft_weights = NETWORK.ft_weights;
    for i in 0..(HL / 32) {
        let offset = i * 32;

        unsafe {
            let mut value = i16x32::load(input.as_ptr().add(offset));
            value += i16x32::load(ft_weights.as_ptr().add(add1 * HL + offset));
            value += i16x32::load(ft_weights.as_ptr().add(add2 * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub1 * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub2 * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub3 * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub4 * HL + offset));

            value.store(output.as_mut_ptr().add(offset));
        }
    }
}

/*----------------------------------------------------------------*/

impl Nnue {
    pub fn make_move(&mut self, old_board: &Board, new_board: &Board, mv: Move) {
        let mut update = FeatureUpdate::default();
        let (src, mut dest) = (mv.src(), mv.dest());
        let piece = old_board.piece_on(src).unwrap();
        let stm = old_board.stm();

        if mv.is_castling() {
            let our_backrank = Rank::First.relative_to(stm);
            let (king, rook) = if src.file() < dest.file() {
                (File::G, File::F)
            } else {
                (File::C, File::D)
            };
            let king_dest = Square::new(king, our_backrank);
            let rook_dest = Square::new(rook, our_backrank);

            update.piece_sub = PieceFeature::new(Piece::King, stm, src);
            update.piece_sub2 = PieceFeature::new(Piece::Rook, stm, dest);
            update.piece_add = PieceFeature::new(Piece::King, stm, king_dest);
            update.piece_add2 = PieceFeature::new(Piece::Rook, stm, rook_dest);

            dest = king_dest;
        } else if let Some(promotion) = mv.promotion() {
            update.piece_sub = PieceFeature::new(piece, stm, src);
            update.piece_add = PieceFeature::new(promotion, stm, dest);
            update.count_sub = CountFeature::new(old_board, piece, stm, src);
            update.count_add = CountFeature::new(new_board, promotion, stm, dest);

            if mv.is_capture() {
                let victim = old_board.piece_on(dest).unwrap();
                update.piece_sub2 = PieceFeature::new(victim, !stm, dest);
                update.count_sub2 = CountFeature::new(old_board, victim, !stm, dest);
            }
        } else {
            update.piece_sub = PieceFeature::new(piece, stm, src);
            update.piece_add = PieceFeature::new(piece, stm, dest);

            if mv.is_en_passant() {
                let ep_square = Square::new(
                    old_board.en_passant().unwrap(),
                    Rank::Fifth.relative_to(stm),
                );

                update.piece_sub2 = PieceFeature::new(Piece::Pawn, !stm, ep_square);
                update.count_sub = CountFeature::new(old_board, Piece::Pawn, !stm, ep_square);
            } else if mv.is_capture() {
                let victim = old_board.piece_on(dest).unwrap();
                update.piece_sub2 = PieceFeature::new(victim, !stm, dest);
                update.count_sub = CountFeature::new(old_board, victim, !stm, dest);
            }
        }

        self.acc_stack[self.acc_index].update = update;
        self.acc_stack[self.acc_index + 1].dirty = [true; Color::COUNT];
        self.acc_index += 1;

        if HORIZONTAL_MIRRORING
            && piece == Piece::King
            && (src.file() > File::D) != (dest.file() > File::D)
        {
            self.reset(new_board, stm);
        }
    }

    #[inline]
    pub fn eval(&self, board: &Board) -> i32 {
        let bucket = OUTPUT_BUCKETS[board.occupied().popcnt() - 1];
        let (stm, ntm) = (
            self.acc_stack[self.acc_index].select(board.stm()),
            self.acc_stack[self.acc_index].select(!board.stm()),
        );

        let mut output = 0;

        if PAIRWISE_MUL {
            feed_forward_pairwise(stm, ntm, bucket, &mut output);
        } else {
            feed_forward(stm, ntm, bucket, &mut output);
        }

        (output / QA + i32::from(NETWORK.out_bias[bucket])) * W::eval_scale() / (QA * QB)
    }
}

/*----------------------------------------------------------------*/

#[inline]
fn feed_forward(stm: &[i16; HL], ntm: &[i16; HL], bucket: usize, output: &mut i32) {
    let out_weights = &NETWORK.out_weights[bucket];
    let (zero, qa) = (i16x32::splat(0), i16x32::splat(QA as i16));
    let mut sum = i32x16::splat(0);

    for i in 0..(HL / 32) {
        let offset = i * 32;

        unsafe {
            let stm = i16x32::load(stm.as_ptr().add(offset)).clamp(zero, qa);
            let ntm = i16x32::load(ntm.as_ptr().add(offset)).clamp(zero, qa);
            let stm_weight = i16x32::load(out_weights.as_ptr().add(offset));
            let ntm_weight = i16x32::load(out_weights.as_ptr().add(HL + offset));

            sum += (stm * stm_weight).madd(stm);
            sum += (ntm * ntm_weight).madd(ntm);
        }
    }

    *output = sum.reduce_sum();
}

#[inline]
fn feed_forward_pairwise(stm: &[i16; HL], ntm: &[i16; HL], bucket: usize, output: &mut i32) {
    let out_weights = &NETWORK.out_weights[bucket];
    let (zero, qa) = (i16x32::splat(0), i16x32::splat(QA as i16));
    let mut sum = i32x16::splat(0);

    for i in 0..(HL / 64) {
        let offset = i * 32;

        unsafe {
            let stm0 = i16x32::load(stm.as_ptr().add(offset)).clamp(zero, qa);
            let stm1 = i16x32::load(stm.as_ptr().add(offset + HL / 2)).clamp(zero, qa);
            let ntm0 = i16x32::load(ntm.as_ptr().add(offset)).clamp(zero, qa);
            let ntm1 = i16x32::load(ntm.as_ptr().add(offset + HL / 2)).clamp(zero, qa);

            let stm_weight = i16x32::load(out_weights.as_ptr().add(offset));
            let ntm_weight = i16x32::load(out_weights.as_ptr().add(HL / 2 + offset));

            sum += (stm0 * stm_weight).madd(stm1);
            sum += (ntm0 * ntm_weight).madd(ntm1);
        }
    }

    *output = sum.reduce_sum();
}
