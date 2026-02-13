use crate::*;

pub static NETWORK: NetworkWeights =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/network.nnue"))) };

#[derive(Debug, Clone)]
pub struct Nnue {
    pub acc_stack: Box<[Accumulator; MAX_PLY as usize + 1]>,
    pub acc_index: usize,
}

impl Nnue {
    pub fn new(board: &Board) -> Nnue {
        let mut nnue = Nnue {
            acc_stack: vec![Accumulator::default(); MAX_PLY as usize + 1]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            acc_index: 0,
        };

        nnue.full_reset(board);
        nnue
    }

    #[inline]
    pub fn full_reset(&mut self, board: &Board) {
        self.acc_index = 0;
        self.reset(board, Color::White);
        self.reset(board, Color::Black);
    }

    #[inline]
    pub fn reset(&mut self, board: &Board, perspective: Color) {
        self.acc_stack[self.acc_index].reset(board, perspective);
    }

    #[inline]
    pub fn apply_updates(&mut self, board: &Board) {
        for &color in &Color::ALL {
            if self.acc_stack[self.acc_index].dirty[color as usize] {
                self.lazy_update(board, color);
            }
        }
    }

    #[inline]
    fn lazy_update(&mut self, board: &Board, perspective: Color) {
        //Find the first clean accumulator
        let clean_index = (0..self.acc_index)
            .rev()
            .find(|&i| !self.acc_stack[i].dirty[perspective])
            .unwrap();

        let king = board.king(perspective);

        //Extrapolate all accumulators from thereon
        for index in clean_index..self.acc_index {
            let [clean, dirty] = self.acc_stack.get_disjoint_mut([index, index + 1]).unwrap();
            dirty.extrapolate(clean, king, perspective);
        }
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.acc_index -= 1;
    }
}
