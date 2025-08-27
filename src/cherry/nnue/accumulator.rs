use arrayvec::ArrayVec;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Accumulator {
    pub white: Align64<[i16; HL]>,
    pub black: Align64<[i16; HL]>,
    
    pub update_buffer: UpdateBuffer,
    pub dirty: [bool; Color::COUNT],
}

impl Accumulator {
    #[inline]
    pub fn select(&self, color: Color) -> &Align64<[i16; HL]> {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    #[inline]
    pub fn select_mut(&mut self, color: Color) -> &mut Align64<[i16; HL]> {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
}

/*----------------------------------------------------------------*/

pub fn vec_add(
    acc: &mut Align64<[i16; HL]>,
    weights: &NetworkWeights,
    adds: &[usize],
) {
    for i in 0..(HL / I16_CHUNK) {
        let offset = i * I16_CHUNK;

        unsafe {
            let mut value = load_i16(acc.as_ptr().add(offset));

            for &index in adds {
                value = add_i16(value, load_i16(weights.ft_weights.as_ptr().add(index * HL + offset)));
            }

            store_i16(acc.as_mut_ptr().add(offset), value);
        }
    }

    for i in (HL - HL % I16_CHUNK)..HL {
        for &index in adds {
            acc[i] += weights.ft_weights[index * HL + i];
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
    for i in 0..(HL / I16_CHUNK) {
        let offset = i * I16_CHUNK;

        unsafe {
            let mut value = load_i16(input.as_ptr().add(offset));
            value = add_i16(value, load_i16(weights.ft_weights.as_ptr().add(add * HL + offset)));
            value = sub_i16(value, load_i16(weights.ft_weights.as_ptr().add(sub * HL + offset)));

            store_i16(output.as_mut_ptr().add(offset), value);
        }
    }

    for i in (HL - HL % I16_CHUNK)..HL {
        output[i] = input[i] + weights.ft_weights[add * HL + i]
            - weights.ft_weights[sub * HL + i];
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
    for i in 0..(HL / I16_CHUNK) {
        let offset = i * I16_CHUNK;

        unsafe {
            let mut value = load_i16(input.as_ptr().add(offset));
            value = add_i16(value, load_i16(weights.ft_weights.as_ptr().add(add * HL + offset)));
            value = sub_i16(value, load_i16(weights.ft_weights.as_ptr().add(sub1 * HL + offset)));
            value = sub_i16(value, load_i16(weights.ft_weights.as_ptr().add(sub2 * HL + offset)));

            store_i16(output.as_mut_ptr().add(offset), value);
        }
    }

    for i in (HL - HL % I16_CHUNK)..HL {
        output[i] = input[i] + weights.ft_weights[add * HL + i]
            - weights.ft_weights[sub1 * HL + i]
            - weights.ft_weights[sub2 * HL + i];
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
    for i in 0..(HL / I16_CHUNK) {
        let offset = i * I16_CHUNK;

        unsafe {
            let mut value = load_i16(input.as_ptr().add(offset));
            value = add_i16(value, load_i16(weights.ft_weights.as_ptr().add(add1 * HL + offset)));
            value = add_i16(value, load_i16(weights.ft_weights.as_ptr().add(add2 * HL + offset)));
            value = sub_i16(value, load_i16(weights.ft_weights.as_ptr().add(sub1 * HL + offset)));
            value = sub_i16(value, load_i16(weights.ft_weights.as_ptr().add(sub2 * HL + offset)));

            store_i16(output.as_mut_ptr().add(offset), value);
        }
    }

    for i in (HL - HL % I16_CHUNK)..HL {
        output[i] = input[i] + weights.ft_weights[add1 * HL + i]
            + weights.ft_weights[add2 * HL + i]
            - weights.ft_weights[sub1 * HL + i]
            - weights.ft_weights[sub2 * HL + i];
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone, Default)]
pub struct UpdateBuffer {
    pub add: ArrayVec<FeatureUpdate, 2>,
    pub sub: ArrayVec<FeatureUpdate, 2>,
}

impl UpdateBuffer {
    #[inline]
    pub fn move_piece(&mut self, piece: Piece, color: Color, from: Square, to: Square) {
        self.add_piece(piece, color, to);
        self.remove_piece(piece, color, from);
    }
    
    #[inline]
    pub fn add_piece(&mut self, piece: Piece, color: Color, sq: Square) {
        self.add.push(FeatureUpdate { piece, color, sq });
    }
    
    #[inline]
    pub fn remove_piece(&mut self, piece: Piece, color: Color, sq: Square) {
        self.sub.push(FeatureUpdate { piece, color, sq });
    }

    #[inline]
    pub fn adds(&self) -> &[FeatureUpdate] {
        &self.add
    }

    #[inline]
    pub fn subs(&self) -> &[FeatureUpdate] {
        &self.sub
    }
}