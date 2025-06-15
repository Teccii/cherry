use cozy_chess::*;
use super::BoardUtil;

/*----------------------------------------------------------------*/

pub const MAX_HISTORY: i16 = 1024;

/*----------------------------------------------------------------*/

pub type HistoryTable = [[[i16; Square::NUM]; Piece::NUM]; Color::NUM];

#[derive(Debug, Clone)]
pub struct History {
    quiets: HistoryTable,
    captures: HistoryTable,
}

impl History {
    #[inline(always)]
    pub fn new() -> History {
        History {
            quiets: [[[0; Square::NUM]; Piece::NUM]; Color::NUM],
            captures: [[[0; Square::NUM]; Piece::NUM]; Color::NUM],
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn get_quiet(&self, board: &Board, mv: Move) -> i16 {
        self.quiets[board.side_to_move() as usize]
            [board.piece_on(mv.from).unwrap() as usize]
            [mv.to as usize]
    }
    
    #[inline(always)]
    fn get_quiet_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        &mut self.quiets[board.side_to_move() as usize]
            [board.piece_on(mv.from).unwrap() as usize]
            [mv.to as usize]
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn get_capture(&self, board: &Board, mv: Move) -> i16 {
        self.captures[board.side_to_move() as usize]
            [board.piece_on(mv.from).unwrap() as usize]
            [mv.to as usize]
    }

    #[inline(always)]
    fn get_capture_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        &mut self.captures[board.side_to_move() as usize]
            [board.piece_on(mv.from).unwrap() as usize]
            [mv.to as usize]
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn reset(&mut self) {
        self.quiets = [[[0; Square::NUM]; Piece::NUM]; Color::NUM];
        self.captures = [[[0; Square::NUM]; Piece::NUM]; Color::NUM];
    }
    
    pub fn update(
        &mut self,
        board: &Board,
        best_move: Move,
        quiets: &[Move],
        captures: &[Move],
        depth: u8
    ) {
        let is_capture = board.is_capture(best_move);
        let amount = (14 * depth as i16).min(MAX_HISTORY);
        
        if is_capture {
            History::update_value(self.get_capture_mut(board, best_move), amount);
            
            for &mv in captures {
                History::update_value(self.get_capture_mut(board, mv), -amount);
            }
        } else {
            History::update_value(self.get_quiet_mut(board, best_move), amount);
            
            for &mv in quiets {
                History::update_value(self.get_quiet_mut(board, mv), -amount);
            }
        }
    }
    
    #[inline(always)]
    fn update_value(value: &mut i16, amount: i16) {
        let amount = amount.clamp(-MAX_HISTORY, MAX_HISTORY);
        let decay = (*value as i32 * amount.abs() as i32 / MAX_HISTORY as i32) as i16;
        
        *value += amount - decay;
    }
}