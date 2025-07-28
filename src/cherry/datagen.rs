use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use cherry_chess::{Board, BoardStatus};
use super::*;

pub fn datagen(count: usize, seed: u64, moves: usize) {
    let mut rng = StdRng::seed_from_u64(seed);
    #[cfg(feature = "nnue")] let weights = NetworkWeights::default();

    for _ in 0..count {
        let moves = moves + rng.random_bool(0.5) as usize;
        let fen = gen_fen(&mut rng, moves, #[cfg(feature = "nnue")]&weights);

        println!("info string genfens {}", fen);
    }
}

fn gen_fen(rng: &mut StdRng, moves: usize, #[cfg(feature = "nnue")]weights: &NetworkWeights) -> String {
    let mut board = Board::default();
    for _ in 0..moves {
        if matches!(board.status(), BoardStatus::Checkmate | BoardStatus::Draw) {
            return gen_fen(rng, moves, #[cfg(feature = "nnue")]weights);
        }

        let mut legals = Vec::new();
        board.gen_moves(|moves| {
            legals.extend(moves);
            false
        });

        board.make_move(legals[rng.random_range(0..legals.len())]);
    }

    if matches!(board.status(), BoardStatus::Checkmate | BoardStatus::Draw) {
        return gen_fen(rng, moves, #[cfg(feature = "nnue")]weights);
    }

    if Position::new(board, #[cfg(feature = "nnue")]weights).eval(#[cfg(feature = "nnue")]weights).abs() >= 1000 {
        return gen_fen(rng, moves, #[cfg(feature = "nnue")]weights);
    }

    board.to_string()
}