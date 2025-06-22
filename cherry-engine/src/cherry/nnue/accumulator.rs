use arrayvec::ArrayVec;
use cozy_chess::*;
use super::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Accumulator {
    pub white: Align64<[i16; L1]>,
    pub black: Align64<[i16; L1]>,
    
    pub update: UpdateBuffer,
    pub dirty: [bool; Color::NUM],
}

impl Accumulator {
    pub fn select(&self, color: Color) -> &Align64<[i16; L1]> {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    pub fn select_mut(&mut self, color: Color) -> &mut Align64<[i16; L1]> {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
}

/*----------------------------------------------------------------*/

pub fn vec_add_sub(
    input: &Align64<[i16; L1]>,
    output: &mut Align64<[i16; L1]>,
    weights: &NetworkWeights,
    add: usize,
    sub: usize
) {
    for i in 0..(L1/simd::I16_CHUNK) {
        let offset = i * simd::I16_CHUNK;

        unsafe {
            let add_chunk = simd::load_i16(weights.feature_weights.get_unchecked(add * L1 + offset));
            let sub_chunk = simd::load_i16(weights.feature_weights.get_unchecked(sub * L1 + offset));

            let value = simd::load_i16(input.get_unchecked(offset));
            let value = simd::add_i16(value, add_chunk);
            let value = simd::sub_i16(value, sub_chunk);

            simd::store_i16(output.get_unchecked_mut(offset), value);
        }
    }
}

pub fn vec_add_sub2(
    input: &Align64<[i16; L1]>,
    output: &mut Align64<[i16; L1]>,
    weights: &NetworkWeights,
    add: usize,
    sub1: usize,
    sub2: usize
) {
    for i in 0..(L1/simd::I16_CHUNK) {
        let offset = i * simd::I16_CHUNK;

        unsafe {
            let add_chunk = simd::load_i16(weights.feature_weights.get_unchecked(add * L1 + offset));
            let sub1_chunk = simd::load_i16(weights.feature_weights.get_unchecked(sub1 * L1 + offset));
            let sub2_chunk = simd::load_i16(weights.feature_weights.get_unchecked(sub2 * L1 + offset));

            let value = simd::load_i16(input.get_unchecked(offset));
            let value = simd::add_i16(value, add_chunk);
            let value = simd::sub_i16(value, sub1_chunk);
            let value = simd::sub_i16(value, sub2_chunk);

            simd::store_i16(output.get_unchecked_mut(offset), value);
        }
    }
}

pub fn vec_add2_sub2(
    input: &Align64<[i16; L1]>,
    output: &mut Align64<[i16; L1]>,
    weights: &NetworkWeights,
    add1: usize,
    add2: usize,
    sub1: usize,
    sub2: usize
) {
    for i in 0..(L1/simd::I16_CHUNK) {
        let offset = i * simd::I16_CHUNK;

        unsafe {
            let add1_chunk = simd::load_i16(weights.feature_weights.get_unchecked(add1 * L1 + offset));
            let add2_chunk = simd::load_i16(weights.feature_weights.get_unchecked(add2 * L1 + offset));
            let sub1_chunk = simd::load_i16(weights.feature_weights.get_unchecked(sub1 * L1 + offset));
            let sub2_chunk = simd::load_i16(weights.feature_weights.get_unchecked(sub2 * L1 + offset));

            let value = simd::load_i16(input.get_unchecked(offset));
            let value = simd::add_i16(value, add1_chunk);
            let value = simd::add_i16(value, add2_chunk);
            let value = simd::sub_i16(value, sub1_chunk);
            let value = simd::sub_i16(value, sub2_chunk);

            simd::store_i16(output.get_unchecked_mut(offset), value);
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone, Default)]
pub struct UpdateBuffer {
    pub add: ArrayVec<FeatureUpdate, 2>,
    pub sub: ArrayVec<FeatureUpdate, 2>,
}

impl UpdateBuffer {
    #[inline(always)]
    pub fn move_piece(&mut self, piece: Piece, color: Color, from: Square, to: Square) {
        self.add_piece(piece, color, to);
        self.remove_piece(piece, color, from);
    }
    
    #[inline(always)]
    pub fn add_piece(&mut self, piece: Piece, color: Color, sq: Square) {
        self.add.push(FeatureUpdate { piece, color, sq });
    }
    
    #[inline(always)]
    pub fn remove_piece(&mut self, piece: Piece, color: Color, sq: Square) {
        self.sub.push(FeatureUpdate { piece, color, sq });
    }

    #[inline(always)]
    pub fn adds(&self) -> &[FeatureUpdate] {
        &self.add
    }

    #[inline(always)]
    pub fn subs(&self) -> &[FeatureUpdate] {
        &self.sub
    }
}