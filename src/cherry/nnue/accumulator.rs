use arrayvec::ArrayVec;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Accumulator {
    pub white: Align64<[i16; HL]>,
    pub black: Align64<[i16; HL]>,

    pub mv: MoveData,
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

#[derive(Debug, Clone)]
pub struct AccumulatorCache {
    pub layouts: [PieceLayout; Color::COUNT],
    pub acc: Accumulator,
}

impl AccumulatorCache {
    pub fn load_accumulator(
        &mut self,
        acc: &mut Accumulator,
        board: &Board,
        weights: &NetworkWeights,
        perspective: Color,
    ) {
        let layout = board.layout();
        let king = board.king(perspective);
        let cache = self.acc.select_mut(perspective);

        let mut adds = ArrayVec::<_, 32>::new();
        let mut subs = ArrayVec::<_, 32>::new();
        self.layouts[perspective as usize].iter_diff(&layout, |sq, piece, color, add| if add {
            adds.push(FeatureUpdate { piece, color, sq }.to_index(king, perspective));
        } else {
            subs.push(FeatureUpdate { piece, color, sq }.to_index(king, perspective));
        });

        self.layouts[perspective as usize] = layout;

        vec_update(cache, weights, &adds, &subs);
        *acc.select_mut(perspective) = cache.clone();
        acc.dirty[perspective as usize] = false;
    }
}

impl Default for AccumulatorCache {
    #[inline]
    fn default() -> Self {
        AccumulatorCache {
            layouts: [PieceLayout::default(); Color::COUNT],
            acc: Accumulator {
                white: Align64([0; HL]),
                black: Align64([0; HL]),
                update_buffer: UpdateBuffer::default(),
                dirty: [false; Color::COUNT],
                mv: MoveData::default(),
            },
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
    for i in 0..(HL/CHUNK_SIZE) {
        let offset = i * CHUNK_SIZE;
        let mut value = I16Reg::from_slice(&acc[offset..]);

        for &index in adds {
            value += I16Reg::from_slice(&weights.ft_weights[(index * HL + offset)..]);
        }

        for &index in subs {
            value -= I16Reg::from_slice(&weights.ft_weights[(index * HL + offset)..]);
        }

        value.copy_to_slice(&mut acc[offset..]);
    }

    for i in (HL - HL % CHUNK_SIZE)..HL {
        for &index in adds {
            acc[i] += weights.ft_weights[index * HL + i];
        }

        for &index in subs {
            acc[i] -= weights.ft_weights[index * HL + i];
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