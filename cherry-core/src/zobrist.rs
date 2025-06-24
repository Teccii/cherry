use crate::{Color, File, Piece, Square};

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct Xorshift64 {
    pub state: u64
}

impl Xorshift64 {
    #[inline(always)]
    pub const fn new(state: u64) -> Xorshift64 {
        Xorshift64 { state }
    }

    #[inline(always)]
    pub const fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;

        self.state
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Zobrist {
    pub pieces: [[[u64; Square::COUNT]; Piece::COUNT]; Color::COUNT],
    pub castle_rights: [[u64; File::COUNT]; Color::COUNT],
    pub en_passant: [u64; File::COUNT],
    pub stm: u64,
}

impl Zobrist {
    #[inline(always)]
    const fn empty() -> Zobrist {
        Zobrist {
            pieces: [[[0; Square::COUNT]; Piece::COUNT]; Color::COUNT],
            castle_rights: [[0; File::COUNT]; Color::COUNT],
            en_passant: [0; File::COUNT],
            stm: 0,
        }
    }
    
    #[inline(always)]
    pub const fn piece(&self, sq: Square, piece: Piece, color: Color) -> u64 {
        self.pieces[color as usize][piece as usize][sq as usize]
    }

    #[inline(always)]
    pub const fn castle_rights(&self, file: File, color: Color) -> u64 {
        self.castle_rights[color as usize][file as usize]
    }
    
    #[inline(always)]
    pub const fn en_passant(&self, file: File) -> u64 {
        self.en_passant[file as usize]
    }
    
    #[inline(always)]
    pub const fn stm(&self) -> u64 {
        self.stm
    }
}

/*----------------------------------------------------------------*/

pub const ZOBRIST: Zobrist = {
    let mut zobrist = Zobrist::empty();
    let mut rng = Xorshift64::new(0x1234567890ABCDEFu64);
    
    let mut i = 0;
    while i < Color::COUNT {
        let mut j = 0;
        while j < Piece::COUNT {
            let mut k = 0;
            while k < Square::COUNT {
                zobrist.pieces[i][j][k] = rng.next();
                
                k += 1;
            }
            
            j += 1;
        }
        
        j = 0;
        while j < File::COUNT {
            zobrist.castle_rights[i][j] = rng.next();
            
            j += 1;
        }
        
        i += 1;
    }
    
    i = 0;
    while i < File::COUNT {
        zobrist.en_passant[i] = rng.next();
        i += 1;
    }
    
    zobrist.stm = rng.next();
    
    zobrist
};