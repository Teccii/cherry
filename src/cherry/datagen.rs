use rand::{RngExt, SeedableRng, random_bool, rngs::SmallRng};

use crate::*;

#[inline]
fn gen_opening(rng: &mut SmallRng, dfrc: bool, moves: usize) -> Option<Board> {
    let mut board = if dfrc {
        Board::dfrc_startpos(rng.random_range(0..960), rng.random_range(0..960))
    } else {
        Board::startpos()
    };

    let moves = moves + random_bool(0.5) as usize;
    for _ in 0..moves {
        let legal_moves = board.gen_moves();
        if legal_moves.is_empty() {
            return None;
        }

        let mv = legal_moves[rng.random_range(0..legal_moves.len())];
        board.make_move(mv);
    }

    if board.status() != BoardStatus::Ongoing {
        return None;
    }

    Some(board)
}

impl Engine {
    #[inline]
    pub fn gen_fens(&mut self, num: usize, seed: u64, dfrc: bool, moves: usize) {
        let mut rng = SmallRng::seed_from_u64(seed);
        self.options.soft_target = true;
        self.options.frc = dfrc;

        for _ in 0..num {
            let mut opening = None;
            while opening.is_none() {
                opening = gen_opening(&mut rng, dfrc, moves).filter(|board| {
                    self.pos.set_board(board.clone());
                    self.searcher.search(
                        self.pos.clone(),
                        vec![SearchLimit::MaxNodes(1000)],
                        self.options,
                        SearchInfo::None,
                    );
                    self.searcher.wait();

                    self.searcher.shared.best_score().abs() < 1000
                });
            }

            println!("info string genfens {}", opening.unwrap().to_fen(dfrc));
        }
    }
}
