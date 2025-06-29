/*----------------------------------------------------------------
MIT License | Copyright (c) 2021 analog-hors

Most of cherry-core directly copies, or at the very least
uses a substantial portion of the source code of the wonderful
cozy-chess library written and distributed by analog-hors.

The original source code can be found at https://github.com/analog-hors/cozy-chess.
----------------------------------------------------------------*/

mod board;
mod moves;
mod zobrist;

pub use board::*;
pub use moves::*;
pub use zobrist::*;

pub use cherry_types::*;

fn perft(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    board.gen_moves(|moves| {
        for mv in moves {
            let mut board = board.clone();
            board.make_move(mv);

            nodes += perft(&board, depth - 1);
        }

        false
    });

    nodes
}

#[test]
fn test_perft() {
    const EXPECTED_NODES: &[u64; 14] = &[
        1,
        20,
        400,
        8902,
        197_281,
        4_865_609,
        119_060_324,
        3_195_901_860,
        84_998_978_956,
        2_439_530_234_167,
        69_352_859_712_417,
        2_097_651_003_696_806,
        62_854_969_236_701_747,
        1_981_066_775_000_396_239
    ];

    let board = Board::default();
    for depth in 0..(EXPECTED_NODES.len() as u8) {
        println!(
            "Depth {} Nodes {} Expected {}",
            depth,
            perft(&board, depth),
            EXPECTED_NODES[depth as usize]
        );
    }
}