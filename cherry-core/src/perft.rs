use crate::Board;

pub fn perft(board: &Board, depth: u8) -> u64 {
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
fn perft_test() {
    const EXPECTED_NODES: [u64; 14] = [
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
        1_981_066_775_000_396_239,
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