use std::{
    mem::MaybeUninit,
    sync::Arc,
    ptr
};
use arrayvec::ArrayVec;
use crate::*;

/*----------------------------------------------------------------*/

pub const INPUT: usize = 768;
pub const HL: usize = 768;
pub const L1: usize = HL * 2;

pub const NUM_OUTPUT_BUCKETS: usize = 8;
pub const OUTPUT_BUCKETS: [usize; 33] = [
    0,                // 0
    0, 0, 0, 0, 0, 0, // 1,  2,  3,  4,  5,  6,
    0, 0, 0, 0,       // 7,  8,  9,  10
    1, 1, 1,          // 11, 12, 13
    2, 2, 2,          // 14, 15, 16
    3, 3, 3,          // 17, 18, 19
    4, 4, 4,          // 20, 21, 22
    5, 5, 5,          // 23, 24, 25
    6, 6, 6,          // 26, 27, 28
    7, 7, 7, 7,       // 29, 30, 31, 32
];

pub const EVAL_SCALE: i32 = 400;
pub const QA: i32 = 255;
pub const QB: i32 = 64;

/*----------------------------------------------------------------*/

const NETWORK_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/network.bin"));

#[derive(Debug, Clone)]
#[repr(C)]
pub struct NetworkWeights {
    pub ft_weights: Align64<[i16; INPUT * HL]>,
    pub ft_bias: Align64<[i16; HL]>,
    pub out_weights: Align64<[i16; L1 * NUM_OUTPUT_BUCKETS]>,
    pub out_bias: Align64<[i16; NUM_OUTPUT_BUCKETS]>,
}

impl NetworkWeights {
    pub fn new(mut bytes: &[u8]) -> Arc<NetworkWeights> {
        const I16_SIZE: usize = size_of::<i16>();

        assert_eq!(bytes.len(), (I16_SIZE * (INPUT * HL + HL + L1 * NUM_OUTPUT_BUCKETS + NUM_OUTPUT_BUCKETS)).next_multiple_of(64));
        
        /*
        Required for larger networks, otherwise we would overflow the stack
        even if we try to create it via `Arc::new(NetworkWeights::new(...))`
        because Rust is funky sometimes.
        */
        let mut weights: Arc<MaybeUninit<NetworkWeights>> = Arc::new_uninit();
        unsafe {
            let ptr = Arc::get_mut(&mut weights).unwrap().as_mut_ptr();

            ptr::copy(bytes.as_ptr(), ptr::addr_of_mut!((*ptr).ft_weights).cast(), INPUT * HL * I16_SIZE);
            bytes = &bytes[(INPUT * HL * I16_SIZE)..];
            ptr::copy(bytes.as_ptr(), ptr::addr_of_mut!((*ptr).ft_bias).cast(), HL * I16_SIZE);
            bytes = &bytes[(HL * I16_SIZE)..];
            ptr::copy(bytes.as_ptr(), ptr::addr_of_mut!((*ptr).out_weights).cast(), L1 * NUM_OUTPUT_BUCKETS * I16_SIZE);
            bytes = &bytes[(L1 * NUM_OUTPUT_BUCKETS * I16_SIZE)..];
            ptr::copy(bytes.as_ptr(), ptr::addr_of_mut!((*ptr).out_weights).cast(), NUM_OUTPUT_BUCKETS);

        };

        unsafe { weights.assume_init() }
    }

    pub fn default() -> Arc<NetworkWeights> {
        Self::new(NETWORK_BYTES)
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Nnue {
    pub acc_stack: Box<[Accumulator; MAX_PLY as usize + 1]>,
    pub acc_index: usize,
}

impl Nnue {
    pub fn new(board: &Board, weights: &NetworkWeights) -> Nnue {
        let mut nnue = Nnue {
            acc_stack: Box::new(std::array::from_fn(|_| Accumulator {
                white: Align64([0; HL]),
                black: Align64([0; HL]),
                update_buffer: UpdateBuffer::default(),
                dirty: [false; Color::COUNT],
            })),
            acc_index: 0,
        };

        nnue.full_reset(board, weights);
        nnue
    }

    pub fn full_reset(&mut self, board: &Board, weights: &NetworkWeights) {
        self.acc_index = 0;
        self.reset(board, weights, Color::White);
        self.reset(board, weights, Color::Black);
    }

    pub fn reset(&mut self, board: &Board, weights: &NetworkWeights, perspective: Color) {
        let acc = self.acc_mut();
        match perspective {
            Color::White => acc.white = weights.ft_bias.clone(),
            Color::Black => acc.black = weights.ft_bias.clone(),
        }

        let mut adds = ArrayVec::<_, 32>::new();
        for sq in board.occupied() {
            let piece = board.piece_on(sq).unwrap();
            let color = board.color_on(sq).unwrap();

            adds.push(FeatureUpdate { piece, color, sq });
        }

        let king = board.king(perspective);
        let adds: Vec<usize> = adds.iter().map(|f| f.to_index(king, perspective)).collect();

        vec_add(acc.select_mut(perspective), weights, &adds);
        acc.dirty[perspective as usize] = false;
    }

    pub fn make_move(
        &mut self,
        old_board: &Board,
        new_board: &Board,
        weights: &NetworkWeights,
        mv: Move
    ) {
        let mut update = UpdateBuffer::default();
        let (from, to) = (mv.from(), mv.to());
        let piece = old_board.piece_on(from).unwrap();
        let color = old_board.stm();

        if old_board.is_castling(mv) {
            let (king, rook) = if from.file() < to.file() {
                (File::G, File::F)
            } else {
                (File::C, File::D)
            };

            let back_rank = Rank::First.relative_to(color);

            update.move_piece(Piece::King, color, from, Square::new(king, back_rank));
            update.move_piece(Piece::Rook, color, to, Square::new(rook, back_rank));
        } else if let Some(promotion) = mv.promotion() {
            update.remove_piece(piece, color, from);
            update.add_piece(promotion, color, to);

            if old_board.is_capture(mv) {
                update.remove_piece(old_board.piece_on(to).unwrap(), !color, to);
            }
        } else {
            update.move_piece(piece, color, from, to);

            if let Some(victim) = old_board.victim(mv) {
                let sq = if old_board.is_en_passant(mv) {
                    Square::new(
                        old_board.en_passant().unwrap(),
                        Rank::Fifth.relative_to(old_board.stm())
                    )
                } else {
                    mv.to()
                };

                update.remove_piece(victim, !color, sq);
            }
        }

        self.acc_mut().update_buffer = update;
        self.acc_index += 1;
        self.acc_mut().dirty = [true; Color::COUNT];

        if piece == Piece::King && (from.file() > File::D) != (to.file() > File::D) {
            self.reset(new_board, weights, color);
        }
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.acc_index -= 1;
    }
    
    pub fn eval(&self, weights: &NetworkWeights, bucket: usize, stm: Color) -> i16 {
        let acc = self.acc();
        let (stm, ntm) = (acc.select(stm), acc.select(!stm));

        let mut output = 0;
        feed_forward(stm, ntm, bucket, &weights, &mut output);
        
        ((output / QA + i32::from(weights.out_bias[bucket])) * EVAL_SCALE / (QA * QB)) as i16
    }

    /*----------------------------------------------------------------*/

    pub fn apply_updates(&mut self, board: &Board, weights: &NetworkWeights) {
        for &color in &Color::ALL {
            if self.acc().dirty[color as usize] {
                self.lazy_update(board, weights, color);
            }
        }
    }

    fn lazy_update(&mut self, board: &Board, weights: &NetworkWeights, perspective: Color) {
        let mut index = self.acc_index;

        //Find the first non-dirty accumulator
        loop {
            index -= 1;
            if !self.acc_stack[index].dirty[perspective as usize] {
                break;
            }
        }

        let king = board.king(perspective);

        //Recalculate all accumulators from thereon
        loop {
            index += 1;
            self.next_acc(weights, king, perspective, index);
            self.acc_stack[index].dirty[perspective as usize] = false;

            if index == self.acc_index {
                break;
            }
        }
    }
    
    fn next_acc(&mut self, weights: &NetworkWeights, king: Square, perspective: Color, index: usize) {
        let (prev, next) = self.acc_stack.split_at_mut(index);
        let src = prev.last().unwrap();
        let target = next.first_mut().unwrap();
        
        match (src.update_buffer.adds(), src.update_buffer.subs()) {
            //quiet moves, including promotions
            (&[add], &[sub]) => {
                let add = add.to_index(king, perspective);
                let sub = sub.to_index(king, perspective);
                
                vec_add_sub(
                    src.select(perspective),
                    target.select_mut(perspective),
                    weights,
                    add,
                    sub
                );
            },
            //captures, including promotions and en passant
            (&[add], &[sub1, sub2]) => {
                let add = add.to_index(king, perspective);
                let sub1 = sub1.to_index(king, perspective);
                let sub2 = sub2.to_index(king, perspective);
                
                vec_add_sub2(
                    src.select(perspective),
                    target.select_mut(perspective),
                    weights,
                    add,
                    sub1,
                    sub2
                );
            },
            //castling
            (&[add1, add2], &[sub1, sub2]) => {
                let add1 = add1.to_index(king, perspective);
                let add2 = add2.to_index(king, perspective);
                let sub1 = sub1.to_index(king, perspective);
                let sub2 = sub2.to_index(king, perspective);

                vec_add2_sub2(
                    src.select(perspective),
                    target.select_mut(perspective),
                    weights,
                    add1,
                    add2,
                    sub1,
                    sub2
                );
            },
            _ => panic!("Invalid Update: {:?}", src.update_buffer)
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn acc(&self) -> &Accumulator {
        &self.acc_stack[self.acc_index]
    }

    #[inline]
    fn acc_mut(&mut self) -> &mut Accumulator {
        &mut self.acc_stack[self.acc_index]
    }
}
