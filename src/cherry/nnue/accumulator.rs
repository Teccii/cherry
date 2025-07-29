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
    for i in 0..(HL/CHUNK_SIZE) {
        let offset = i * CHUNK_SIZE;
        let mut value = I16Reg::from_slice(&acc[offset..]);
        
        for &index in adds {
            value += I16Reg::from_slice(&weights.ft_weights[(index * HL + offset)..]);
        }

        value.copy_to_slice(&mut acc[offset..]);
    }

    for i in (HL - HL % CHUNK_SIZE)..HL {
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
    for i in 0..(HL/CHUNK_SIZE) {
        let offset = i * CHUNK_SIZE;
        let add_chunk = I16Reg::from_slice(&weights.ft_weights[(add * HL + offset)..]);
        let sub_chunk = I16Reg::from_slice(&weights.ft_weights[(sub * HL + offset)..]);
        let value = I16Reg::from_slice(&input[offset..]);
        let value = value + add_chunk - sub_chunk;
        
        value.copy_to_slice(&mut output[offset..]);
    }

    for i in (HL - HL % CHUNK_SIZE)..HL {
        output[i] += input[i] + weights.ft_weights[add * HL + i]
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
    for i in 0..(HL/CHUNK_SIZE) {
        let offset = i * CHUNK_SIZE;
        let add_chunk = I16Reg::from_slice(&weights.ft_weights[(add * HL + offset)..]);
        let sub1_chunk = I16Reg::from_slice(&weights.ft_weights[(sub1 * HL + offset)..]);
        let sub2_chunk = I16Reg::from_slice(&weights.ft_weights[(sub2 * HL + offset)..]);
        let value = I16Reg::from_slice(&input[offset..]);
        let value = value + add_chunk - sub1_chunk - sub2_chunk;

        value.copy_to_slice(&mut output[offset..]);
    }

    for i in (HL - HL % CHUNK_SIZE)..HL {
        output[i] += input[i] + weights.ft_weights[add * HL + i]
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
    for i in 0..(HL/CHUNK_SIZE) {
        let offset = i * CHUNK_SIZE;
        let add1_chunk = I16Reg::from_slice(&weights.ft_weights[(add1 * HL + offset)..]);
        let add2_chunk = I16Reg::from_slice(&weights.ft_weights[(add2 * HL + offset)..]);
        let sub1_chunk = I16Reg::from_slice(&weights.ft_weights[(sub1 * HL + offset)..]);
        let sub2_chunk = I16Reg::from_slice(&weights.ft_weights[(sub2 * HL + offset)..]);
        let value = I16Reg::from_slice(&input[offset..]);
        let value = value + add1_chunk + add2_chunk - sub1_chunk - sub2_chunk;

        value.copy_to_slice(&mut output[offset..]);
    }

    for i in (HL - HL % CHUNK_SIZE)..HL {
        output[i] += input[i] + weights.ft_weights[add1 * HL + i]
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