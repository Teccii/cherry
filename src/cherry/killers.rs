use cozy_chess::Move;

pub const KILLER_COUNT: usize = 3;

#[derive(Debug, Copy, Clone)]
pub struct Killers {
    moves: [Option<Move>; KILLER_COUNT],
}

impl Killers {
    #[inline(always)]
    pub fn new() -> Killers {
        Killers {
            moves: [None; KILLER_COUNT],
        }
    }
    
    pub fn contains(&self, mv: Move) -> bool {
        for &killer in self.moves.iter() {
            if killer == Some(mv) {
                return true;
            }
        }
        
        false
    }
    
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<Move> {
        if index >= KILLER_COUNT {
            return None;
        }
        
        self.moves[index]
    }
    
    #[inline(always)]
    pub fn push(&mut self, mv: Move){
        if self.moves[0] == Some(mv) {
            return;
        }
        
        self.moves[2] = self.moves[1];
        self.moves[1] = self.moves[0];
        self.moves[0] = Some(mv);
    }
    
    #[inline(always)]
    pub fn clear(&mut self) {
        self.moves = [None; KILLER_COUNT];
    }
}