use cozy_chess::*;
use super::*;
use crate::*;

/*----------------------------------------------------------------*/

pub const INPUT: usize = 768;
pub const L1: usize = 1024;
pub const L2: usize = 32;
pub const L3: usize = 32;

pub const QA: i16 = 255;
pub const QB: i16 = 64;

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
#[repr(C)]
pub struct NetworkWeights {
    pub feature_weights: Align64<[i16; INPUT * L1]>,
    pub feature_bias: Align64<[i16; L1]>,
    pub l1_weights: Align64<[i16; L1 * L2]>,
    pub l1_bias: Align64<[i16; L2]>,
    pub l2_weights: Align64<[i16; L2 * L3]>,
    pub l2_bias: Align64<[i16; L3]>,
    pub l3_weights: Align64<[i16; L3]>,
    pub l3_bias: Align64<[i16; L3]>,
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Nnue {
    pub acc_stack: [Accumulator; MAX_PLY as usize + 1],
    pub acc_index: usize,
}

impl Nnue {
    pub fn make_move(&mut self, board: &Board, mv: Move) {
        let mut update = UpdateBuffer::default();
        let piece = board.piece_on(mv.from).unwrap();
        let color = board.side_to_move();
        
        if board.is_castles(mv) {
            let (king, rook) = if mv.from.file() < mv.to.file() {
                (File::G, File::F)
            } else {
                (File::C, File::D)
            };
            
            let back_rank = Rank::First.relative_to(color);
            
            update.move_piece(Piece::King, color, mv.from, Square::new(king, back_rank));
            update.move_piece(Piece::Rook, color, mv.to, Square::new(rook, back_rank));
        } else if mv.promotion.is_some() {
            update.remove_piece(piece, color, mv.from);
            update.add_piece(mv.promotion.unwrap(), color, mv.to);
            
            if board.is_capture(mv) {
                update.remove_piece(board.piece_on(mv.to).unwrap(), !color, mv.to); 
            }
        } else {
            update.move_piece(piece, color, mv.from, mv.to);
            
            //handles en passant
            if let Some(sq) = board.capture_square(mv) {
                update.remove_piece(board.piece_on(sq).unwrap(), !color, sq);
            }
        }
        
        self.acc_mut().update = update;
        
        self.acc_index += 1;
        self.acc_mut().dirty = [false; Color::NUM];
    }
    
    pub fn unmake_move(&mut self) {
        self.acc_index -= 1;
    }
    
    pub fn eval(&self, weights: &NetworkWeights, stm: Color) -> i16 {
        let acc = self.acc();
        let (stm, nstm) = match stm {
            Color::White => (&acc.white, &acc.black),
            Color::Black => (&acc.black, &acc.white),
        };
        
        let mut l1_outputs = Align64([0; L2]);
        let mut l2_outputs = Align64([0; L3]);
        let mut l3_output = 0;
        
        l3_output
    }
    
    /*----------------------------------------------------------------*/
    
    pub fn apply_updates(&mut self, weights: &NetworkWeights, perspective: Color) {
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
            self.acc_mut().dirty[perspective as usize] = false;
            
            if index == self.acc_index {
                break;
            }
        }
    }
    
    pub fn force_updates(&mut self, weights: &NetworkWeights) {
        for &color in Color::ALL.iter() {
            if self.acc().dirty[color as usize] {
                self.apply_updates(weights, color);
            }
        }
    }
    
    fn next_acc(&mut self, weights: &NetworkWeights, perspective: Color, index: usize) {
        let (prev, next) = self.acc_stack.split_at_mut(index);
        let src = prev.last().unwrap();
        let target = next.first_mut().unwrap();
        
        match (src.update.adds(), src.update.subs()) {
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
            _ => panic!("Invalid Update: {:?}", src.update)
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    fn acc(&self) -> &Accumulator {
        &self.acc_stack[self.acc_index]
    }

    #[inline(always)]
    fn acc_mut(&mut self) -> &mut Accumulator {
        &mut self.acc_stack[self.acc_index]
    }
}
