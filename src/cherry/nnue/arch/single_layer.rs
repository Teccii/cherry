use std::num::NonZeroU16;

use arrayvec::ArrayVec;

use crate::*;

pub const INPUT: usize = 768;
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
    pub out_weights: [i16; HL * NUM_OUTPUT_BUCKETS * (2 - PAIRWISE_MUL as usize)],
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
pub struct Feature(NonZeroU16);

impl Feature {
    #[inline]
    pub fn new(piece: Piece, color: Color, sq: Square) -> Feature {
        let mut bits = 0;
        bits |= color as u16;
        bits |= (piece as u16) << 1;
        bits |= (sq as u16) << 4;

        Feature(NonZeroU16::new(bits).unwrap())
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

        color as usize * Square::COUNT * Piece::COUNT + piece as usize * Square::COUNT + sq as usize
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct FeatureUpdate {
    pub add: Option<Feature>,
    pub add2: Option<Feature>,
    pub sub: Option<Feature>,
    pub sub2: Option<Feature>,
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
        let (add, sub) = (prev.update.add.unwrap(), prev.update.sub.unwrap());

        match (prev.update.add2, prev.update.sub2) {
            (Some(add2), Some(sub2)) => acc_add2_sub2(
                prev.select(perspective),
                self.select_mut(perspective),
                add.to_index(king, perspective),
                add2.to_index(king, perspective),
                sub.to_index(king, perspective),
                sub2.to_index(king, perspective),
            ),
            (Some(_), None) => unreachable!(),
            (None, Some(sub2)) => acc_add_sub2(
                prev.select(perspective),
                self.select_mut(perspective),
                add.to_index(king, perspective),
                sub.to_index(king, perspective),
                sub2.to_index(king, perspective),
            ),
            (None, None) => acc_add_sub(
                prev.select(perspective),
                self.select_mut(perspective),
                add.to_index(king, perspective),
                sub.to_index(king, perspective),
            ),
        }

        self.dirty[perspective] = false;
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn reset(&mut self, board: &Board, perspective: Color) {
        let acc = self.select_mut(perspective);
        acc.copy_from_slice(&NETWORK.ft_bias);

        let king = board.king(perspective);
        let updates: ArrayVec<usize, 32> = board
            .occupied()
            .iter()
            .map(|sq| {
                Feature::new(board.piece_on(sq).unwrap(), board.color_on(sq).unwrap(), sq)
                    .to_index(king, perspective)
            })
            .collect();

        let ft_weights = &NETWORK.ft_weights;
        for i in 0..(HL / 32) {
            let offset = i * 32;

            unsafe {
                let mut value = i16x32::load(acc.as_ptr().add(offset));
                for &index in &updates {
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
fn acc_add_sub2(input: &[i16; HL], output: &mut [i16; HL], add: usize, sub1: usize, sub2: usize) {
    let ft_weights = &NETWORK.ft_weights;
    for i in 0..(HL / 32) {
        let offset = i * 32;

        unsafe {
            let mut value = i16x32::load(input.as_ptr().add(offset));
            value += i16x32::load(ft_weights.as_ptr().add(add * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub1 * HL + offset));
            value -= i16x32::load(ft_weights.as_ptr().add(sub2 * HL + offset));

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

/*----------------------------------------------------------------*/

impl Nnue {
    pub fn make_move(&mut self, old_board: &Board, new_board: &Board, mv: Move) {
        let mut update = FeatureUpdate::default();
        let (src, dest) = (mv.src(), mv.dest());
        let piece = old_board.piece_on(src).unwrap();
        let stm = old_board.stm();

        if mv.is_castling() {
            let our_backrank = Rank::First.relative_to(stm);
            let (king, rook) = if src.file() < dest.file() {
                (File::G, File::F)
            } else {
                (File::C, File::D)
            };

            update.sub = Some(Feature::new(Piece::King, stm, src));
            update.sub2 = Some(Feature::new(Piece::Rook, stm, dest));
            update.add = Some(Feature::new(
                Piece::King,
                stm,
                Square::new(king, our_backrank),
            ));
            update.add2 = Some(Feature::new(
                Piece::Rook,
                stm,
                Square::new(rook, our_backrank),
            ));
        } else if let Some(promotion) = mv.promotion() {
            update.sub = Some(Feature::new(piece, stm, src));
            update.add = Some(Feature::new(promotion, stm, dest));

            if mv.is_capture() {
                update.sub2 = Some(Feature::new(old_board.piece_on(dest).unwrap(), !stm, dest));
            }
        } else {
            update.sub = Some(Feature::new(piece, stm, src));
            update.add = Some(Feature::new(piece, stm, dest));

            if mv.is_en_passant() {
                let ep_square = Square::new(
                    old_board.en_passant().unwrap(),
                    Rank::Fifth.relative_to(stm),
                );

                update.sub2 = Some(Feature::new(Piece::Pawn, !stm, ep_square));
            } else if mv.is_capture() {
                update.sub2 = Some(Feature::new(old_board.piece_on(dest).unwrap(), !stm, dest));
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
    let out_weights = &NETWORK.out_weights;
    let (zero, qa) = (i16x32::splat(0), i16x32::splat(QA as i16));
    let mut sum = i32x16::splat(0);

    for i in 0..(HL / 32) {
        let offset = i * 32;

        unsafe {
            let stm = i16x32::load(stm.as_ptr().add(offset)).clamp(zero, qa);
            let ntm = i16x32::load(ntm.as_ptr().add(offset)).clamp(zero, qa);
            let stm_weight = i16x32::load(out_weights.as_ptr().add(bucket * HL * 2 + offset));
            let ntm_weight = i16x32::load(out_weights.as_ptr().add(bucket * HL * 2 + HL + offset));

            sum += (stm * stm_weight).madd(stm);
            sum += (ntm * ntm_weight).madd(ntm);
        }
    }

    *output = sum.reduce_sum();
}

#[inline]
fn feed_forward_pairwise(stm: &[i16; HL], ntm: &[i16; HL], bucket: usize, output: &mut i32) {
    let out_weights = &NETWORK.out_weights;
    let (zero, qa) = (i16x32::splat(0), i16x32::splat(QA as i16));
    let mut sum = i32x16::splat(0);

    for i in 0..(HL / 64) {
        let offset = i * 32;

        unsafe {
            let stm0 = i16x32::load(stm.as_ptr().add(offset)).clamp(zero, qa);
            let stm1 = i16x32::load(stm.as_ptr().add(offset + HL / 2)).clamp(zero, qa);
            let ntm0 = i16x32::load(ntm.as_ptr().add(offset)).clamp(zero, qa);
            let ntm1 = i16x32::load(ntm.as_ptr().add(offset + HL / 2)).clamp(zero, qa);

            let stm_weight = i16x32::load(out_weights.as_ptr().add(bucket * HL + offset));
            let ntm_weight = i16x32::load(out_weights.as_ptr().add(bucket * HL + HL / 2 + offset));

            sum += (stm0 * stm_weight).madd(stm1);
            sum += (ntm0 * ntm_weight).madd(ntm1);
        }
    }

    *output = sum.reduce_sum();
}
