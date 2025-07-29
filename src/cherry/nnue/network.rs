use arrayvec::ArrayVec;
use crate::*;

/*----------------------------------------------------------------*/

pub const INPUT: usize = 768;
pub const HL: usize = 256;
pub const L1: usize = HL * 2;

pub const EVAL_SCALE: i32 = 400;
pub const QA: i32 = 255;
pub const QB: i32 = 64;

/*----------------------------------------------------------------*/

const NETWORK_BYTES: &[u8] = &[];

#[derive(Debug, Clone)]
#[repr(C)]
pub struct NetworkWeights {
    pub ft_weights: Align64<[i16; INPUT * HL]>,
    pub ft_bias: Align64<[i16; HL]>,
    pub out_weights: Align64<[i16; L1]>,
    pub out_bias: i16
}

impl NetworkWeights {
    pub fn new(mut bytes: &[u8]) -> NetworkWeights {
        const I16_SIZE: usize = size_of::<i16>();

        debug_assert!(bytes.len() == I16_SIZE * (
            INPUT * HL + HL + L1 + 1
        ));

        let ft_weights = Self::aligned_from_bytes::<{INPUT * HL}>(bytes);
        bytes = &bytes[(INPUT * HL * I16_SIZE)..];
        let ft_bias = Self::aligned_from_bytes::<HL>(bytes);
        bytes = &bytes[(HL * I16_SIZE)..];
        let out_weights = Self::aligned_from_bytes::<L1>(bytes);
        bytes = &bytes[(L1 * I16_SIZE)..];
        let out_bias = i16::from_le_bytes([bytes[0], bytes[1]]);

        NetworkWeights {
            ft_weights,
            ft_bias,
            out_weights,
            out_bias,
        }
    }

    fn aligned_from_bytes<const N: usize>(bytes: &[u8]) -> Align64<[i16; N]> {
        let mut values = Align64([0; N]);

        for (chunk, value) in bytes.chunks_exact(2)
            .zip(&mut values.0)
            .take(N) {
            *value = i16::from_le_bytes([chunk[0], chunk[1]]);
        }

        values
    }
}

impl Default for NetworkWeights {
    #[inline]
    fn default() -> Self {
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

        nnue.reset(board, weights);
        nnue
    }

    pub fn reset(&mut self, board: &Board, weights: &NetworkWeights) {
        for acc in &mut *self.acc_stack {
            acc.white = weights.ft_bias.clone();
            acc.black = weights.ft_bias.clone();
        }

        let mut adds = ArrayVec::<_, 32>::new();
        for sq in board.occupied() {
            let piece = board.piece_on(sq).unwrap();
            let color = board.color_on(sq).unwrap();

            adds.push(FeatureUpdate { piece, color, sq });
        }

        let (w_add, b_add): (Vec<_>, Vec<_>) = adds.iter()
            .map(|f| (f.to_index(Color::White), f.to_index(Color::Black)))
            .unzip();

        self.acc_index = 0;
        let acc = self.acc_mut();
        vec_add(acc.select_mut(Color::White), weights, &w_add);
        vec_add(acc.select_mut(Color::Black), weights, &b_add);
        acc.dirty = [false; Color::COUNT];
    }

    pub fn make_move(&mut self, board: &Board, mv: Move) {
        let mut update = UpdateBuffer::default();
        let (from, to) = (mv.from(), mv.to());
        let piece = board.piece_on(from).unwrap();
        let color = board.stm();
        
        if board.is_castling(mv) {
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
            
            if board.is_capture(mv) {
                update.remove_piece(board.piece_on(to).unwrap(), !color, to); 
            }
        } else {
            update.move_piece(piece, color, from, to);
            
            if let Some(victim) = board.victim(mv) {
                let sq = if board.is_en_passant(mv) {
                    Square::new(
                        board.en_passant().unwrap(), 
                        Rank::Fifth.relative_to(board.stm())
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
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.acc_index -= 1;
    }
    
    pub fn eval(&self, weights: &NetworkWeights, stm: Color) -> i16 {
        let acc = self.acc();
        let (us, them) = (acc.select(stm), acc.select(!stm));

        let mut output = 0;
        feed_forward(them, &weights.out_weights[HL..].try_into().unwrap(), &mut output);
        feed_forward(us, &weights.out_weights[..HL].try_into().unwrap(), &mut output);

        ((output / QA + weights.out_bias as i32) * EVAL_SCALE / (QA * QB)) as i16
    }
    
    /*----------------------------------------------------------------*/

    pub fn apply_updates(&mut self, weights: &NetworkWeights) {
        for &color in &Color::ALL {
            if self.acc().dirty[color as usize] {
                self.lazy_update(weights, color);
            }
        }
    }

    fn lazy_update(&mut self, weights: &NetworkWeights, perspective: Color) {
        let mut index = self.acc_index;

        //Find the first non-dirty accumulator
        loop {
            index -= 1;

            if !self.acc_stack[index].dirty[perspective as usize] {
                break;
            }
        }

        //Recalculate all accumulators from thereon
        loop {
            index += 1;
            self.next_acc(weights, perspective, index);
            self.acc_stack[index].dirty[perspective as usize] = false;

            if index == self.acc_index {
                break;
            }
        }
    }
    
    fn next_acc(&mut self, weights: &NetworkWeights, perspective: Color, index: usize) {
        let (prev, next) = self.acc_stack.split_at_mut(index);
        let src = prev.last().unwrap();
        let target = next.first_mut().unwrap();
        
        match (src.update_buffer.adds(), src.update_buffer.subs()) {
            //quiet moves, including promotions
            (&[add], &[sub]) => {
                let add = add.to_index(perspective);
                let sub = sub.to_index(perspective);
                
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
                let add = add.to_index(perspective);
                let sub1 = sub1.to_index(perspective);
                let sub2 = sub2.to_index(perspective);
                
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
                let add1 = add1.to_index(perspective);
                let add2 = add2.to_index(perspective);
                let sub1 = sub1.to_index(perspective);
                let sub2 = sub2.to_index(perspective);

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
