use arrayvec::ArrayVec;
use cozy_chess::*;
use super::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Accumulator {
    pub white: Align64<[i16; HL]>,
    pub black: Align64<[i16; HL]>,
    
    pub update: UpdateBuffer,
    pub dirty: [bool; Color::NUM],
}

impl Accumulator {
    pub fn select(&self, color: Color) -> &Align64<[i16; HL]> {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    pub fn select_mut(&mut self, color: Color) -> &mut Align64<[i16; HL]> {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
}

/*----------------------------------------------------------------*/

pub fn vec_update(
    acc: &mut Align64<[i16; HL]>,
    weights: &NetworkWeights,
    adds: &[usize],
    subs: &[usize],
) {
    for i in 0..(HL/simd::I16_CHUNK) {
        let offset = i * simd::I16_CHUNK;

        unsafe {
            let mut add_chunks = ArrayVec::<_, 64>::new();
            let mut sub_chunks = ArrayVec::<_, 64>::new();
            for &index in adds {
                add_chunks.push(simd::load_i16(weights.ft_weights.get_unchecked(index * HL + offset)));
            }
            for &index in subs {
                sub_chunks.push(simd::load_i16(weights.ft_weights.get_unchecked(index * HL + offset)));
            }

            let mut value = simd::load_i16(acc.get_unchecked(offset));
            for chunk in add_chunks {
                value = simd::add_i16(value, chunk);
            }
            for chunk in sub_chunks {
                value = simd::sub_i16(value, chunk);
            }

            simd::store_i16(acc.get_unchecked_mut(offset), value);
        }
    }
}

pub fn vec_add_inplace(
    acc: &mut Align64<[i16; HL]>,
    weights: &NetworkWeights,
    adds: &[usize],
) {
    for i in 0..(HL/simd::I16_CHUNK) {
        let offset = i * simd::I16_CHUNK;

        unsafe {
            let mut add_chunks = ArrayVec::<_, 64>::new();
            for &index in adds {
                add_chunks.push(simd::load_i16(weights.ft_weights.get_unchecked(index * HL + offset)));
            }

            let mut value = simd::load_i16(acc.get_unchecked(offset));

            for chunk in add_chunks {
                value = simd::add_i16(value, chunk);
            }

            simd::store_i16(acc.get_unchecked_mut(offset), value);
        }
    }
}

pub fn vec_sub(
    acc: &mut Align64<[i16; HL]>,
    weights: &NetworkWeights,
    subs: &[usize],
) {
    for i in 0..(HL/simd::I16_CHUNK) {
        let offset = i * simd::I16_CHUNK;

        unsafe {
            let mut sub_chunks = ArrayVec::<_, 64>::new();
            for &index in subs {
                sub_chunks.push(simd::load_i16(weights.ft_weights.get_unchecked(index * HL + offset)));
            }

            let mut value = simd::load_i16(acc.get_unchecked(offset));

            for chunk in sub_chunks {
                value = simd::sub_i16(value, chunk);
            }

            simd::store_i16(acc.get_unchecked_mut(offset), value);
        }
    }
}

pub fn vec_add_sub(
    input: &Align64<[i16; HL]>,
    output: &mut Align64<[i16; HL]>,
    weights: &NetworkWeights,
    add: usize,
    sub: usize
) {
    for i in 0..(HL /simd::I16_CHUNK) {
        let offset = i * simd::I16_CHUNK;

        unsafe {
            let add_chunk = simd::load_i16(weights.ft_weights.get_unchecked(add * HL + offset));
            let sub_chunk = simd::load_i16(weights.ft_weights.get_unchecked(sub * HL + offset));

            let mut value = simd::load_i16(input.get_unchecked(offset));
            value = simd::add_i16(value, add_chunk);
            value = simd::sub_i16(value, sub_chunk);

            simd::store_i16(output.get_unchecked_mut(offset), value);
        }
    }
}

pub fn vec_add_sub2(
    input: &Align64<[i16; HL]>,
    output: &mut Align64<[i16; HL]>,
    weights: &NetworkWeights,
    add: usize,
    sub1: usize,
    sub2: usize
) {
    for i in 0..(HL /simd::I16_CHUNK) {
        let offset = i * simd::I16_CHUNK;

        unsafe {
            let add_chunk = simd::load_i16(weights.ft_weights.get_unchecked(add * HL + offset));
            let sub1_chunk = simd::load_i16(weights.ft_weights.get_unchecked(sub1 * HL + offset));
            let sub2_chunk = simd::load_i16(weights.ft_weights.get_unchecked(sub2 * HL + offset));

            let mut value = simd::load_i16(input.get_unchecked(offset));
            value = simd::add_i16(value, add_chunk);
            value = simd::sub_i16(value, sub1_chunk);
            value = simd::sub_i16(value, sub2_chunk);

            simd::store_i16(output.get_unchecked_mut(offset), value);
        }
    }
}

pub fn vec_add2_sub2(
    input: &Align64<[i16; HL]>,
    output: &mut Align64<[i16; HL]>,
    weights: &NetworkWeights,
    add1: usize,
    add2: usize,
    sub1: usize,
    sub2: usize
) {
    for i in 0..(HL /simd::I16_CHUNK) {
        let offset = i * simd::I16_CHUNK;

        unsafe {
            let add1_chunk = simd::load_i16(weights.ft_weights.get_unchecked(add1 * HL + offset));
            let add2_chunk = simd::load_i16(weights.ft_weights.get_unchecked(add2 * HL + offset));
            let sub1_chunk = simd::load_i16(weights.ft_weights.get_unchecked(sub1 * HL + offset));
            let sub2_chunk = simd::load_i16(weights.ft_weights.get_unchecked(sub2 * HL + offset));

            let mut value = simd::load_i16(input.get_unchecked(offset));
            value = simd::add_i16(value, add1_chunk);
            value = simd::add_i16(value, add2_chunk);
            value = simd::sub_i16(value, sub1_chunk);
            value = simd::sub_i16(value, sub2_chunk);

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