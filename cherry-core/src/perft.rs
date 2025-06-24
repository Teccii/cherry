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