use rand::{Rng, SeedableRng, rngs::StdRng};
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
    macro_rules! try_again {
        ($e:expr) => {
            if $e {
                return gen_fen(rng, moves, #[cfg(feature = "nnue")]weights);
            }
        }
    }

    let mut board = Board::default();
    for _ in 0..moves {
        try_again!(matches!(board.status(), BoardStatus::Checkmate | BoardStatus::Draw));

        let mut legals = Vec::new();
        board.gen_moves(|moves| {
            legals.extend(moves);
            false
        });

        board.make_move(legals[rng.random_range(0..legals.len())]);
    }

    try_again!(matches!(board.status(), BoardStatus::Checkmate | BoardStatus::Draw));
    try_again!(Position::new(board, #[cfg(feature = "nnue")]weights).eval(#[cfg(feature = "nnue")]weights).abs() > 1000);

    board.to_string()
}