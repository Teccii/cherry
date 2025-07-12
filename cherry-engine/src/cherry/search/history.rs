use cherry_core::*;
use crate::MoveData;

/*----------------------------------------------------------------*/

pub const MAX_HISTORY: i16 = 8192;

/*----------------------------------------------------------------*/

pub const fn move_to<T: Copy>(default: T) -> MoveTo<T> {
    [[default; Square::COUNT]; Square::COUNT]
}

pub const fn piece_to<T: Copy>(default: T) -> PieceTo<T> {
    [[default; Square::COUNT]; Piece::COUNT]
}

pub type MoveTo<T> = [[T; Square::COUNT]; Square::COUNT];
pub type PieceTo<T> = [[T; Square::COUNT]; Piece::COUNT];
pub type ButterflyTable = [MoveTo<i16>; Color::COUNT];
pub type PieceToTable = [PieceTo<i16>; Color::COUNT];
pub type ContinuationTable = [PieceTo<PieceTo<i16>>; Color::COUNT];

#[derive(Debug, Clone)]
pub struct History {
    quiets: Box<ButterflyTable>,
    captures: Box<PieceToTable>,
    counter_move: Box<ContinuationTable>,
    follow_up: Box<ContinuationTable>
}

impl History {
    #[inline(always)]
    pub fn new() -> History {
        History {
            quiets: Box::new([move_to(0); Color::COUNT]),
            captures: Box::new([piece_to(0); Color::COUNT]),
            counter_move: Box::new([piece_to(piece_to(0)); Color::COUNT]),
            follow_up: Box::new([piece_to(piece_to(0)); Color::COUNT]),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn get_quiet(&self, board: &Board, mv: Move) -> i16 {
        self.quiets[board.stm() as usize]
            [mv.from() as usize]
            [mv.to() as usize]
    }
    
    #[inline(always)]
    fn get_quiet_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        &mut self.quiets[board.stm() as usize]
            [mv.to() as usize]
            [mv.to() as usize]
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn get_capture(&self, board: &Board, mv: Move) -> i16 {
        self.captures[board.stm() as usize]
            [board.piece_on(mv.from()).unwrap() as usize]
            [mv.to() as usize]
    }

    #[inline(always)]
    fn get_capture_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        &mut self.captures[board.stm() as usize]
            [board.piece_on(mv.from()).unwrap() as usize]
            [mv.to() as usize]
    }

    /*----------------------------------------------------------------*/

    pub fn get_counter_move(
        &self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<i16> {
        let prev_mv = prev_mv?;

        Some(self.counter_move[board.stm() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from()).unwrap() as usize][mv.to() as usize])
    }

    fn get_counter_move_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<&mut i16> {
        let prev_mv = prev_mv?;

        Some(&mut self.counter_move[board.stm() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from()).unwrap() as usize][mv.to() as usize])
    }

    /*----------------------------------------------------------------*/

    pub fn get_follow_up(
        &self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<i16> {
        let prev_mv = prev_mv?;

        Some(self.follow_up[board.stm() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from()).unwrap() as usize][mv.to() as usize])
    }

    fn get_follow_up_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<MoveData>
    ) -> Option<&mut i16> {
        let prev_mv = prev_mv?;

        Some(&mut self.follow_up[board.stm() as usize]
            [prev_mv.piece as usize][prev_mv.to as usize]
            [board.piece_on(mv.from()).unwrap() as usize][mv.to() as usize])
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
        self.quiets = Box::new([move_to(0); Color::COUNT]);
        self.captures = Box::new([piece_to(0); Color::COUNT]);
        self.counter_move = Box::new([piece_to(piece_to(0)); Color::COUNT]);
        self.follow_up = Box::new([piece_to(piece_to(0)); Color::COUNT]);
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