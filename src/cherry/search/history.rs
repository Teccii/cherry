use cozy_chess::*;
use crate::BoardUtil;
use crate::cherry::MoveData;
/*----------------------------------------------------------------*/

pub const MAX_HISTORY: i16 = 8192;

/*----------------------------------------------------------------*/

const fn move_to<T: Copy>(default: T) -> MoveTo<T> {
    [[default; Square::NUM]; Square::NUM]
}

const fn piece_to<T: Copy>(default: T) -> PieceTo<T> {
    [[default; Square::NUM]; Piece::NUM]
}

pub type MoveTo<T> = [[T; Square::NUM]; Square::NUM];
pub type PieceTo<T> = [[T; Square::NUM]; Piece::NUM];
pub type ButterflyTable = [MoveTo<i16>; Color::NUM];
pub type PieceToTable = [PieceTo<i16>; Color::NUM];
pub type ContinuationTable = [PieceTo<PieceTo<i16>>; Color::NUM];

#[derive(Debug, Clone)]
pub struct History {
    quiets: Box<ButterflyTable>,
    captures: Box<PieceToTable>,
    counter_move: Box<ContinuationTable>,
    follow_up: Box<ContinuationTable>,
}

impl History {
    #[inline(always)]
    pub fn new() -> History {
        History {
            quiets: Box::new([move_to(0); Color::NUM]),
            captures: Box::new([piece_to(0); Color::NUM]),
            counter_move: Box::new([piece_to(piece_to(0)); Color::NUM]),
            follow_up: Box::new([piece_to(piece_to(0)); Color::NUM]),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn get_quiet(&self, board: &Board, mv: Move) -> i16 {
        self.quiets[board.side_to_move() as usize]
            [mv.from as usize]
            [mv.to as usize]
    }
    
    #[inline(always)]
    fn get_quiet_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        &mut self.quiets[board.side_to_move() as usize]
            [mv.to as usize]
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

    pub fn get_counter_move(
        &self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>,
    ) -> Option<i16> {
        let prev_mv = prev_mv?;

        Some(self.counter_move[board.side_to_move() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from).unwrap() as usize][mv.to as usize])
    }

    pub fn get_counter_move_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>,
    ) -> Option<&mut i16> {
        let prev_mv = prev_mv?;

        Some(&mut self.counter_move[board.side_to_move() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from).unwrap() as usize][mv.to as usize])
    }

    pub fn get_follow_up(
        &self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>,
    ) -> Option<i16> {
        let prev_mv = prev_mv?;

        Some(self.follow_up[board.side_to_move() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from).unwrap() as usize][mv.to as usize])
    }

    pub fn get_follow_up_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>,
    ) -> Option<&mut i16> {
        let prev_mv = prev_mv?;

        Some(&mut self.follow_up[board.side_to_move() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from).unwrap() as usize][mv.to as usize])
    }

    /*----------------------------------------------------------------*/
    
    #[inline(always)]
    pub fn get_move(
        &self,
        board: &Board,
        mv: Move,
        counter_move: Option<MoveData>,
        follow_up: Option<MoveData>
    ) -> i16 {
        if board.is_capture(mv) {
            self.get_capture(board, mv)
        } else {
            self.get_quiet(board, mv)
                + self.get_counter_move(board, mv, counter_move).unwrap_or_default()
                + self.get_follow_up(board, mv, follow_up).unwrap_or_default()
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn reset(&mut self) {
        self.quiets = Box::new([move_to(0); Color::NUM]);
        self.captures = Box::new([piece_to(0); Color::NUM]);
        self.counter_move = Box::new([piece_to(piece_to(0)); Color::NUM]);
        self.follow_up = Box::new([piece_to(piece_to(0)); Color::NUM]);
    }
    
    pub fn update(
        &mut self,
        board: &Board,
        best_move: Move,
        counter_move: Option<MoveData>,
        follow_up: Option<MoveData>,
        quiets: &[Move],
        captures: &[Move],
        depth: u8
    ) {
        let is_capture = board.is_capture(best_move);
        let amount = (14 * depth as i16).min(MAX_HISTORY);
        
        if is_capture {
            History::update_value(self.get_capture_mut(board, best_move), amount);
        } else {
            History::update_value(self.get_quiet_mut(board, best_move), amount);
            
            for &mv in quiets {
                History::update_value(self.get_quiet_mut(board, mv), -amount);
            }
            
            if let Some(value) = self.get_counter_move_mut(board, best_move, counter_move) {
                History::update_value(value, amount);
                
                for &mv in quiets {
                    History::update_value(
                        self.get_counter_move_mut(board, mv, counter_move).unwrap(),
                        -amount
                    );
                }
            }
            
            if let Some(value) = self.get_follow_up_mut(board, best_move, follow_up) {
                History::update_value(value, amount);

                for &mv in quiets {
                    History::update_value(
                        self.get_follow_up_mut(board, mv, follow_up).unwrap(),
                        -amount
                    );
                }
            }
        }

        for &mv in captures {
            History::update_value(self.get_capture_mut(board, mv), -amount);
        }
    }
    
    #[inline(always)]
    fn update_value(value: &mut i16, amount: i16) {
        let amount = amount.clamp(-MAX_HISTORY, MAX_HISTORY);
        let decay = (*value as i32 * amount.abs() as i32 / MAX_HISTORY as i32) as i16;
        
        *value += amount - decay;
    }
}