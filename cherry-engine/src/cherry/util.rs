use std::sync::{Arc, atomic::*};
use cozy_chess::*;

/*----------------------------------------------------------------*/

pub const MAX_DEPTH: u8 = 128;
pub const MAX_PLY: u16 = 256;

/*----------------------------------------------------------------*/

#[derive(Debug)]
pub struct BatchedAtomicCounter {
    global: Arc<AtomicU64>,
    local: u64,
    buffer: u64,
}

impl BatchedAtomicCounter {
    #[inline(always)]
    pub fn new() -> BatchedAtomicCounter {
        BatchedAtomicCounter {
            global: Arc::new(AtomicU64::new(0)), 
            local: 0,
            buffer: 0,
        }
    }
    
    #[inline(always)]
    pub fn inc(&mut self) {
        self.buffer += 1;
        
        if self.buffer >= Self::BATCH_SIZE {
            self.flush();
        }
    }
    
    #[inline(always)]
    pub fn flush(&mut self) {
        self.global.fetch_add(self.buffer, Ordering::Relaxed);
        self.local += self.buffer;
        self.buffer = 0;
    }
    
    #[inline(always)]
    pub fn reset(&mut self) {
        self.global.store(0, Ordering::Relaxed);
        self.local = 0;
        self.buffer = 0;
    }

    #[inline(always)]
    pub fn global(&self) -> u64 {
        self.global.load(Ordering::Relaxed) + self.buffer
    }
    
    #[inline(always)]
    pub fn local(&self) -> u64 {
        self.local + self.buffer
    }
    
    #[inline(always)]
    pub fn buffer(&self) -> u64 {
        self.buffer
    }
    
    pub const BATCH_SIZE: u64 = 1024;
}

impl Clone for BatchedAtomicCounter {
    #[inline(always)]
    fn clone(&self) -> Self {
        BatchedAtomicCounter {
            global: Arc::clone(&self.global),
            local: 0,
            buffer: 0,
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BitMove(pub u16);

impl BitMove {
    pub fn pack(mv: Option<Move>) -> BitMove {
        if let Some(mv) = mv {
            let mut bits = mv.from as u16;
            bits |= (mv.to as u16) << 6;
            bits |= mv.promotion.map_or(0b1111, |p| p as u16) << 12;
            
            return BitMove(bits);
        }
        
        BitMove(0)
    }
    
    pub fn unpack(self) -> Option<Move> {
        if self.0 == 0 {
            return None;
        }
        
        Some(Move {
            from: Square::index((self.0 & 0b111111) as usize),
            to: Square::index(((self.0 >> 6) & 0b111111) as usize),
            promotion: Piece::try_index(((self.0 >> 12) & 0b1111) as usize)
        })
    }
}

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

#[inline(always)]
pub fn get_queen_moves(sq: Square, blockers: BitBoard) -> BitBoard {
    get_bishop_moves(sq, blockers) | get_rook_moves(sq, blockers)
}

#[inline(always)]
pub fn get_queen_rays(sq: Square) -> BitBoard {
    get_bishop_rays(sq) | get_rook_rays(sq)
}

/*----------------------------------------------------------------*/


pub trait Direction {
    const MASK: BitBoard;
    const SHIFT: isize;
    const DX: i8;
    const DY: i8;
}

pub struct Up;
pub struct Down;
pub struct Right;
pub struct Left;

pub struct UpRight;
pub struct UpLeft;
pub struct DownRight;
pub struct DownLeft;

impl Direction for Up {
    const MASK: BitBoard = BitBoard::FULL;
    const SHIFT: isize = 8;
    const DX: i8 = 0;
    const DY: i8 = 1;
}

impl Direction for Down {
    const MASK: BitBoard = BitBoard::FULL;
    const SHIFT: isize = -8;
    const DX: i8 = 0;
    const DY: i8 = -1;
}

impl Direction for Right {
    const MASK: BitBoard = BitBoard(!File::H.bitboard().0);
    const SHIFT: isize = 1;
    const DX: i8 = 1;
    const DY: i8 = 0;
}

impl Direction for Left {
    const MASK: BitBoard = BitBoard(!File::A.bitboard().0);
    const SHIFT: isize = -1;
    const DX: i8 = -1;
    const DY: i8 = 0;
}

impl Direction for UpRight {
    const MASK: BitBoard = BitBoard(!File::H.bitboard().0);
    const SHIFT: isize = 9;
    const DX: i8 = 1;
    const DY: i8 = 1;
}

impl Direction for UpLeft {
    const MASK: BitBoard = BitBoard(!File::A.bitboard().0);
    const SHIFT: isize = 7;
    const DX: i8 = -1;
    const DY: i8 = 1;
}

impl Direction for DownRight {
    const MASK: BitBoard = BitBoard(!File::H.bitboard().0);
    const SHIFT: isize = -7;
    const DX: i8 = 1;
    const DY: i8 = -1;
}

impl Direction for DownLeft {
    const MASK: BitBoard = BitBoard(!File::A.bitboard().0);
    const SHIFT: isize = -9;
    const DX: i8 = -1;
    const DY: i8 = -1;
}

/*----------------------------------------------------------------*/

pub trait BitBoardUtil {
    fn next_square_back(self) -> Option<Square>;

    fn shift<D: Direction>(self, steps: usize) -> BitBoard;
    fn shift_rel<D: Direction>(self, steps: usize, color: Color) -> BitBoard;
    fn smear<D: Direction>(self) -> BitBoard;
    fn smear_rel<D: Direction>(self, color: Color) -> BitBoard;
    
    fn relative_to(self, color: Color) -> BitBoard;
}

impl BitBoardUtil for BitBoard {
    #[inline(always)]
    fn next_square_back(self) -> Option<Square> {
        Square::try_index(63 - self.0.leading_zeros() as usize)
    }

    #[inline(always)]
    fn shift<D: Direction>(self, steps: usize) -> BitBoard {
        /*
        For some reason, `shl` takes an `isize` as a parameter but then panics if you try to shift
        by a negative number. This makes no sense. It should just do `shr` if it's negative...
        */
        
        let mut result = self;

        for _ in 0..steps {
            result = if D::SHIFT > 0 {
                BitBoard((result & D::MASK).0 << D::SHIFT)
            } else {
                BitBoard((result & D::MASK).0 >> -D::SHIFT)
            };
        }
        
        result
    }

    fn shift_rel<D: Direction>(self, steps: usize, color: Color) -> BitBoard {
        /*
        For some reason, `shl` takes an `isize` as a parameter but then panics if you try to shift
        by a negative number. This makes no sense. It should just do `shr` if it's negative...
        */

        let mut result = self;

        for _ in 0..steps {
            result = match color {
                Color::White => if D::SHIFT > 0 {
                    BitBoard((result & D::MASK).0 << D::SHIFT)
                } else {
                    BitBoard((result & D::MASK).0 >> -D::SHIFT)
                },
                Color::Black => if D::SHIFT > 0 {
                    BitBoard((result & D::MASK).0 >> D::SHIFT)
                } else {
                    BitBoard((result & D::MASK).0 << -D::SHIFT)
                }
            };
        }

        result
    }

    #[inline(always)]
    fn smear<D: Direction>(self) -> BitBoard {
        let mut result = self;
        
        result |= result.shift::<D>(1);
        result |= result.shift::<D>(2);
        result |= result.shift::<D>(4);
        
        result
    }
    
    #[inline(always)]
    fn smear_rel<D: Direction>(self, color: Color) -> BitBoard {
        let mut result = self;
        
        result |= result.shift_rel::<D>(1, color);
        result |= result.shift_rel::<D>(2, color);
        result |= result.shift_rel::<D>(4, color);
        
        result
    }

    #[inline(always)]
    fn relative_to(self, color: Color) -> BitBoard {
        match color {
            Color::White => self,
            Color::Black => self.flip_ranks()
        }
    }
}

/*----------------------------------------------------------------*/

pub trait BoardUtil {
    fn minor_pieces(&self) -> BitBoard;
    fn major_pieces(&self) -> BitBoard;
    fn colored_minors(&self, color: Color) -> BitBoard;
    fn colored_majors(&self, color: Color) -> BitBoard;
    fn orth_sliders(&self) -> BitBoard;
    fn diag_sliders(&self) -> BitBoard;
    fn sliders(&self) -> BitBoard;
    fn colored_orth_sliders(&self, color: Color) -> BitBoard;
    fn colored_diag_sliders(&self, color: Color) -> BitBoard;
    fn colored_sliders(&self, color: Color) -> BitBoard;
    
    fn in_check(&self) -> bool;
    
    fn is_castles(&self, mv: Move) -> bool;
    fn is_check(&self, mv: Move) -> bool;
    fn is_en_passant(&self, mv: Move) -> bool;
    fn is_quiet_capture(&self, mv: Move) -> bool;
    fn is_capture(&self, mv: Move) -> bool;
    fn is_quiet(&self, mv: Move) -> bool;
    
    fn capture_piece(&self, mv: Move) -> Option<Piece>;
    fn capture_square(&self, mv: Move) -> Option<Square>;
    
    fn pawn_attacks(&self, color: Color) -> BitBoard;
}

impl BoardUtil for Board {
    #[inline(always)]
    fn minor_pieces(&self) -> BitBoard {
        self.pieces(Piece::Knight) | self.pieces(Piece::Bishop)
    }
    
    #[inline(always)]
    fn major_pieces(&self) -> BitBoard {
        self.pieces(Piece::Rook) | self.pieces(Piece::Queen)
    }

    #[inline(always)]
    fn colored_minors(&self, color: Color) -> BitBoard {
        self.colors(color) & self.minor_pieces()
    }
    
    #[inline(always)]
    fn colored_majors(&self, color: Color) -> BitBoard {
        self.colors(color) & self.major_pieces()
    }

    #[inline(always)]
    fn sliders(&self) -> BitBoard {
        self.pieces(Piece::Bishop) | self.pieces(Piece::Rook) | self.pieces(Piece::Queen)
    }
    
    #[inline(always)]
    fn orth_sliders(&self) -> BitBoard {
        self.pieces(Piece::Rook) | self.pieces(Piece::Queen)
    }
    
    #[inline(always)]
    fn diag_sliders(&self) -> BitBoard {
        self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)
    }

    #[inline(always)]
    fn colored_sliders(&self, color: Color) -> BitBoard {
        self.colors(color) & self.sliders()
    }

    #[inline(always)]
    fn colored_orth_sliders(&self, color: Color) -> BitBoard {
        self.colors(color) & self.orth_sliders()
    }
    
    #[inline(always)]
    fn colored_diag_sliders(&self, color: Color) -> BitBoard {
        self.colors(color) & self.diag_sliders()
    }

    #[inline(always)]
    fn in_check(&self) -> bool {
        !self.checkers().is_empty()
    }

    #[inline(always)]
    fn is_check(&self, mv: Move) -> bool {
        let mut board = self.clone();
        board.play_unchecked(mv);
        
        board.in_check()
    }
    
    #[inline(always)]
    fn is_castles(&self, mv: Move) -> bool {
        let stm = self.side_to_move();
        
        self.king(stm) == mv.from
        && self.colors(stm).has(mv.to)
    }
    
    #[inline(always)]
    fn is_en_passant(&self, mv: Move) -> bool {
        let stm = self.side_to_move();
        let ep_rank = Rank::Sixth.relative_to(stm);
        let capt_rank = Rank::Fifth.relative_to(stm);

        self.piece_on(mv.from).unwrap() == Piece::Pawn
            && Some(mv.to) == self.en_passant().map(|f| Square::new(f, ep_rank))
            && mv.to.shift_rel::<Down>(stm) == self.en_passant().map(|f| Square::new(f, capt_rank))
    }
    
    #[inline(always)]
    fn is_quiet_capture(&self, mv: Move) -> bool {
        self.is_capture(mv) && !self.is_check(mv)
    }
    
    #[inline(always)]
    fn is_capture(&self, mv: Move) -> bool {
        self.colors(!self.side_to_move()).has(mv.to) || self.is_en_passant(mv)
    }

    #[inline(always)]
    fn is_quiet(&self, mv: Move) -> bool {
        !self.is_capture(mv) && !self.is_check(mv)
    }

    #[inline(always)]
    fn capture_piece(&self, mv: Move) -> Option<Piece> {
        if !self.is_capture(mv) {
            return None;
        }
        
        self.is_en_passant(mv)
            .then_some(Piece::Pawn)
            .or_else(|| self.piece_on(mv.to))
    }
    
    #[inline(always)]
    fn capture_square(&self, mv: Move) -> Option<Square> {
        if !self.is_capture(mv) {
            return None;
        }

        self.is_en_passant(mv)
            .then(|| mv.to.shift_rel::<Down>(self.side_to_move()).unwrap())
            .or(Some(mv.to))
    }
    
    #[inline(always)]
    fn pawn_attacks(&self, color: Color) -> BitBoard {
        let pawns = self.colored_pieces(color, Piece::Pawn);
        
        match color {
            Color::White => !pawns & (pawns.shift::<UpLeft>(1) | pawns.shift::<UpRight>(1)),
            Color::Black => !pawns & (pawns.shift::<DownLeft>(1) | pawns.shift::<DownRight>(1)),
        }
    }
}

/*----------------------------------------------------------------*/

pub trait FileRankUtil {
    fn shift<D: Direction>(self) -> Option<Self> where Self: Sized;
    fn shift_rel<D: Direction>(self, color: Color) -> Option<Self> where Self: Sized;
    
    fn try_offset(self, delta: i8) -> Option<Self> where Self: Sized;
    fn offset(self, delta: i8) -> Self;
}

impl FileRankUtil for File {
    #[inline(always)]
    fn shift<D: Direction>(self) -> Option<Self> {
        self.try_offset(D::DX)
    }
    
    #[inline(always)]
    fn shift_rel<D: Direction>(self, color: Color) -> Option<Self> {
        self.try_offset(match color {
            Color::White => D::DX,
            Color::Black => -D::DX,
        })
    }
    
    fn try_offset(self, delta: i8) -> Option<Self> {
        let new_index = self as i8 + delta;
        
        if new_index < 0 || new_index >= File::NUM as i8 {
            return None;
        }
        
        Some(File::index(new_index as usize))
    }
    
    fn offset(self, delta: i8) -> Self {
        let new_index = self as i8 + delta;
        
        if new_index < 0 || new_index >= File::NUM as i8 {
            panic!("File::offset(): New index out of bounds");
        }
        
        File::index(new_index as usize)
    }
}

impl FileRankUtil for Rank {
    #[inline(always)]
    fn shift<D: Direction>(self) -> Option<Self> {
        self.try_offset(D::DY)
    }

    #[inline(always)]
    fn shift_rel<D: Direction>(self, color: Color) -> Option<Self> {
        self.try_offset(match color {
            Color::White => D::DY,
            Color::Black => -D::DY,
        })
    }
    
    fn try_offset(self, delta: i8) -> Option<Self> {
        let new_index = self as i8 + delta;

        if new_index < 0 || new_index >= Rank::NUM as i8 {
            return None;
        }

        Some(Rank::index(new_index as usize))
    }

    fn offset(self, delta: i8) -> Self {
        let new_index = self as i8 + delta;

        if new_index < 0 || new_index >= Rank::NUM as i8 {
            panic!("File::offset(): New index out of bounds");
        }

        Rank::index(new_index as usize)
    }
}

/*----------------------------------------------------------------*/

pub trait RankUtil {
    fn above(self) -> BitBoard;
    fn below(self) -> BitBoard;
}

impl RankUtil for Rank {
    #[inline(always)]
    fn above(self) -> BitBoard {
        BitBoard(0xFFFFFFFFFFFFFF00).shift::<Up>(self as usize)
    }
    
    #[inline(always)]
    fn below(self) -> BitBoard {
        BitBoard(0xFFFFFFFFFFFFFF).shift::<Down>(Rank::Eighth as usize - self as usize)
    }
}

/*----------------------------------------------------------------*/

pub trait SquareUtil {
    fn shift<D: Direction>(self) -> Option<Self> where Self: Sized;
    fn shift_rel<D: Direction>(self, color: Color) -> Option<Self> where Self: Sized;
    fn dist(self, other: Square) -> u8;
    fn center_dist(self) -> u8;
}

impl SquareUtil for Square {
    #[inline(always)]
    fn shift<D: Direction>(self) -> Option<Self> {
        self.try_offset(D::DX, D::DY)
    }

    #[inline(always)]
    fn shift_rel<D: Direction>(self, color: Color) -> Option<Self> {
        let (dx, dy) = match color {
            Color::White => (D::DX, D::DY),
            Color::Black => (-D::DX, -D::DY),
        };
        
        self.try_offset(dx, dy)
    }
    
    #[inline(always)]
    fn dist(self, other: Square) -> u8 {
        ((other.rank() as i8 - self.rank() as i8).abs() + (other.file() as i8 - self.file() as i8).abs()) as u8
    }

    #[inline(always)]
    fn center_dist(self) -> u8 {
        const TABLE: [u8; Square::NUM] = [
            6, 5, 4, 3, 3, 4, 5, 6,
            5, 4, 3, 2, 2, 3, 4, 5,
            4, 3, 2, 1, 1, 2, 3, 4,
            3, 2, 1, 0, 0, 1, 2, 3,
            3, 2, 1, 0, 0, 1, 2, 3,
            4, 3, 2, 1, 1, 2, 3, 4,
            5, 4, 3, 2, 2, 3, 4, 5,
            6, 5, 4, 3, 3, 4, 5, 6
        ];
        
        TABLE[self as usize]
    }
}

/*----------------------------------------------------------------*/

#[test]
fn test_bitboard_util() {
    let e5_bb = Square::E5.bitboard();
    
    assert_eq!(e5_bb.shift::<Up>(1), Square::E6.bitboard());
    assert_eq!(e5_bb.shift::<Down>(1), Square::E4.bitboard());
    assert_eq!(e5_bb.shift::<Right>(1), Square::F5.bitboard());
    assert_eq!(e5_bb.shift::<Left>(1), Square::D5.bitboard());
    assert_eq!(e5_bb.shift::<UpRight>(1), Square::F6.bitboard());
    assert_eq!(e5_bb.shift::<UpLeft>(1), Square::D6.bitboard());
    assert_eq!(e5_bb.shift::<DownRight>(1), Square::F4.bitboard());
    assert_eq!(e5_bb.shift::<DownLeft>(1), Square::D4.bitboard());
    
    assert_eq!(e5_bb.smear::<Up>(), BitBoard(0x1010101000000000));
    assert_eq!(e5_bb.smear::<Down>(), BitBoard(0x1010101010));
    assert_eq!(e5_bb.smear::<Right>(), BitBoard(0xF000000000));
    assert_eq!(e5_bb.smear::<Left>(), BitBoard(0x1F00000000));
    assert_eq!(e5_bb.smear::<UpRight>(), BitBoard(0x8040201000000000));
    assert_eq!(e5_bb.smear::<UpLeft>(), BitBoard(0x204081000000000));
    assert_eq!(e5_bb.smear::<DownRight>(), BitBoard(0x1020408000));
    assert_eq!(e5_bb.smear::<DownLeft>(), BitBoard(0x1008040201));
    
    assert_eq!(e5_bb.relative_to(Color::White), Square::E5.bitboard());
    assert_eq!(e5_bb.relative_to(Color::Black), Square::E4.bitboard());

    let mut f3_b6_bb = Square::F3.bitboard() | Square::B6.bitboard();
    assert_eq!(f3_b6_bb.next_square_back(), Some(Square::B6));
    f3_b6_bb ^= f3_b6_bb.next_square_back().unwrap().bitboard();
    assert_eq!(f3_b6_bb.next_square_back(), Some(Square::F3));
}

#[test]
fn test_board_util() {
    let board = Board::default();
    
    assert_eq!(board.minor_pieces(), BitBoard(0x6600000000000066));
    assert_eq!(board.major_pieces(), BitBoard(0x8900000000000089));
    
    //random ass position
    let board = "8/3p2k1/p4pp1/2P4p/1P2PP2/6P1/8/3K4 w - - 0 1".parse::<Board>().unwrap();
    
    assert_eq!(board.pawn_attacks(Color::White), BitBoard(0xA7980000000));
    assert_eq!(board.pawn_attacks(Color::Black), BitBoard(0x147240000000));

    let board = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2".parse::<Board>().unwrap();
    let e4d5 = Move {
        from: Square::E4,
        to: Square::D5,
        promotion: None
    };
     

    assert_eq!(board.is_capture(e4d5), true);
    assert_eq!(board.is_quiet_capture(e4d5), true);
    assert_eq!(board.is_check(e4d5), false);
    assert_eq!(board.is_en_passant(e4d5), false);
    assert_eq!(board.is_castles(e4d5), false);
    assert_eq!(board.is_quiet(e4d5), false);

    let board = "rnbqkbnr/ppp2ppp/8/3Pp3/8/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 3".parse::<Board>().unwrap();
    let d5e6 = Move {
        from: Square::D5,
        to: Square::E6,
        promotion: None
    };

    assert_eq!(board.is_capture(d5e6), true);
    assert_eq!(board.is_quiet_capture(d5e6), true);
    assert_eq!(board.is_check(d5e6), false);
    assert_eq!(board.is_en_passant(d5e6), true);
    assert_eq!(board.is_castles(d5e6), false);

    let board = "r1bqkbnr/pp1n1pp1/2p1P2p/8/8/2N2Q2/PPPP1PPP/R1B1KBNR w KQkq - 2 6".parse::<Board>().unwrap();
    let e6d7 = Move {
        from: Square::E6,
        to: Square::D7,
        promotion: None
    };

    assert_eq!(board.is_capture(e6d7), true);
    assert_eq!(board.is_quiet_capture(e6d7), false);
    assert_eq!(board.is_check(e6d7), true);
    assert_eq!(board.is_en_passant(e6d7), false);
    assert_eq!(board.is_castles(e6d7), false);

    let board = "r1q1kb1r/pp1bnpp1/2p4p/8/8/2NB1Q1N/PPPP1PPP/R1B1K2R w KQkq - 4 9".parse::<Board>().unwrap();
    let e1h1 = Move {
        from: Square::E1,
        to: Square::H1,
        promotion: None
    };

    assert_eq!(board.is_capture(e1h1), false);
    assert_eq!(board.is_quiet_capture(e1h1), false);
    assert_eq!(board.is_check(e1h1), false);
    assert_eq!(board.is_en_passant(e1h1), false);
    assert_eq!(board.is_castles(e1h1), true);

    let board = "r2k1bnr/ppq2b2/2p2ppp/8/2B5/1PN2QPN/PBP2P1P/R3K2R w KQ - 4 17".parse::<Board>().unwrap();
    let e1a1 = Move {
        from: Square::E1,
        to: Square::A1,
        promotion: None
    };

    assert_eq!(board.is_capture(e1a1), false);
    assert_eq!(board.is_quiet_capture(e1a1), false);
    assert_eq!(board.is_check(e1a1), true);
    assert_eq!(board.is_en_passant(e1a1), false);
    assert_eq!(board.is_castles(e1a1), true);

    let board = "r1bqkbnr/p4ppp/npp5/3pP3/8/2N5/PPPPQPPP/R1B1KBNR w KQkq d6 0 5".parse::<Board>().unwrap();
    let e5d6 = Move {
        from: Square::E5,
        to: Square::D6,
        promotion: None
    };
    
    assert_eq!(board.is_capture(e5d6), true);
    assert_eq!(board.is_quiet_capture(e5d6), false);
    assert_eq!(board.is_check(e5d6), true);
    assert_eq!(board.is_en_passant(e5d6), true);
    assert_eq!(board.is_castles(e5d6), false);
}

#[test]
fn test_file_rank_util() {
    assert_eq!(File::G.try_offset(1), Some(File::H));
    assert_eq!(File::G.try_offset(-1), Some(File::F));
    assert_eq!(File::A.try_offset(1), Some(File::B));
    assert_eq!(File::A.try_offset(-1), None);
    
    assert_eq!(Rank::Fifth.try_offset(1), Some(Rank::Sixth));
    assert_eq!(Rank::Fifth.try_offset(-1), Some(Rank::Fourth));
    assert_eq!(Rank::First.try_offset(1), Some(Rank::Second));
    assert_eq!(Rank::First.try_offset(-1), None);
}

#[test]
fn test_rank_util() {
    assert_eq!(Rank::Fifth.above(), BitBoard(0xFFFFFF0000000000));
    assert_eq!(Rank::Eighth.above(), BitBoard::EMPTY);
    assert_eq!(Rank::Fifth.below(), BitBoard(0xFFFFFFFF));
    assert_eq!(Rank::First.below(), BitBoard::EMPTY);
}