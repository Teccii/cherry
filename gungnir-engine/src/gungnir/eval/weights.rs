use gungnir_core::*;
use std::ops::*;
use crate::gungnir::Score;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct T(pub i16, pub i16);

impl T {
    pub fn scale(self, phase: u16) -> Score {
        let phase = (phase * TAPER_SCALE + TOTAL_PHASE / 2) / TOTAL_PHASE;
        let score = (self.0 as i32 * (TAPER_SCALE - phase) as i32
            + self.1 as i32 * phase as i32) / TAPER_SCALE as i32;

        Score::new(score as i16)
    }
    
    pub const ZERO: T = T(0, 0);
}

macro_rules! impl_tapered_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for T {
            type Output = T;
            
            #[inline(always)]
            fn $fn(self, rhs: T) -> Self::Output {
                T(self.0.$fn(rhs.0), self.1.$fn(rhs.1))
            }
        }
    )*}
}

macro_rules! impl_tapered_i16_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<i16> for T {
            type Output = T;
            
            #[inline(always)]
            fn $fn(self, rhs: i16) -> Self::Output {
                T(self.0.$fn(rhs), self.1.$fn(rhs))
            }
        }
    )*}
}

macro_rules! impl_tapered_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for T {
            #[inline(always)]
            fn $fn(&mut self, rhs: T) {
                self.0.$fn(rhs.0);
                self.1.$fn(rhs.1);
            }
        }
    )*}
}

macro_rules! impl_tapered_i16_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<i16> for T {
            #[inline(always)]
            fn $fn(&mut self, rhs: i16) {
                self.0.$fn(rhs);
                self.1.$fn(rhs);
            }
        }
    )*}
}

impl_tapered_ops! {
    Add, add;
    Sub, sub;
}

impl_tapered_i16_ops! {
    Mul, mul;
    Div, div;
}

impl_tapered_assign_ops! {
    AddAssign, add_assign;
    SubAssign, sub_assign;
}

impl_tapered_i16_assign_ops! {
    MulAssign, mul_assign;
    DivAssign, div_assign;
}

pub type IndexTable<const N: usize> = [T; N];
pub type FileTable = IndexTable<{File::COUNT}>;
pub type RankTable = IndexTable<{Rank::COUNT}>;
pub type SquareTable = IndexTable<{Square::COUNT}>;

/*----------------------------------------------------------------*/

pub fn calc_phase(board: &Board) -> u16 {
    let mut phase = TOTAL_PHASE;
    phase -= board.pieces(Piece::Pawn).popcnt() as u16 * PAWN_PHASE;
    phase -= board.pieces(Piece::Knight).popcnt() as u16 * KNIGHT_PHASE;
    phase -= board.pieces(Piece::Bishop).popcnt() as u16 * BISHOP_PHASE;
    phase -= board.pieces(Piece::Rook).popcnt() as u16 * ROOK_PHASE;
    phase -= board.pieces(Piece::Queen).popcnt() as u16 * QUEEN_PHASE;
    phase
}

pub const PAWN_PHASE: u16 = 0;
pub const KNIGHT_PHASE: u16 = 1;
pub const BISHOP_PHASE: u16 = 1;
pub const ROOK_PHASE: u16 = 2;
pub const QUEEN_PHASE: u16 = 4;

pub const TOTAL_PHASE: u16 = 16 * PAWN_PHASE
    + 4 * KNIGHT_PHASE
    + 4 * BISHOP_PHASE
    + 4 * ROOK_PHASE
    + 2 * QUEEN_PHASE;

pub const TAPER_SCALE: u16 = 256;

/*----------------------------------------------------------------*/

macro_rules! weights {
    ($($elem:ident: $ty:ty => $default:expr,)*) => {
        #[derive(Debug, Clone)]
        pub struct EvalWeights {
            $(pub $elem: $ty,)*
        }
        
        impl Default for EvalWeights {
            fn default() -> Self {
                EvalWeights { $($elem: $default,)* }
            }
        }
    }
}

weights! {
    bishop_pair: T => BISHOP_PAIR,

    pawn_value: T => PAWN_VALUE,
    knight_value: T => KNIGHT_VALUE,
    bishop_value: T => BISHOP_VALUE,
    rook_value: T => ROOK_VALUE,
    queen_value: T => QUEEN_VALUE,

    pawn_psqt: SquareTable => PAWN_PSQT,
    knight_psqt: SquareTable => KNIGHT_PSQT,
    bishop_psqt: SquareTable => BISHOP_PSQT,
    rook_psqt: SquareTable => ROOK_PSQT,
    queen_psqt: SquareTable => QUEEN_PSQT,
    king_psqt: SquareTable => KING_PSQT,

    knight_mobility: IndexTable<9> => KNIGHT_MOBILITY,
    bishop_mobility: IndexTable<14> => BISHOP_MOBILITY,
    rook_mobility: IndexTable<15> => ROOK_MOBILITY,
    queen_mobility: IndexTable<28> => QUEEN_MOBILITY,

    rook_open_file: FileTable => ROOK_OPEN_FILE,
    rook_semiopen_file: FileTable => ROOK_SEMIOPEN_FILE,
    queen_open_file: FileTable => QUEEN_OPEN_FILE,
    queen_semiopen_file: FileTable => QUEEN_SEMIOPEN_FILE,

    pawn_minor_threat: T => PAWN_MINOR_THREAT,
    pawn_major_threat: T => PAWN_MAJOR_THREAT,
    minor_major_threat: T => MINOR_MAJOR_THREAT,

    center_control: T => CENTER_CONTROL,

    passed_pawn: RankTable => PASSED_PAWN,
    backwards_pawn: T => BACKWARDS_PAWN,
    isolated_pawn: T => ISOLATED_PAWN,
    doubled_pawn: T => DOUBLED_PAWN,
}

/*----------------------------------------------------------------*/

pub const KNIGHT_MOBILITY: IndexTable<9> = [
    T(-62, -81), T(-53, -56), T(-12, -31),
    T(-4, -16),  T(3, 5),     T(13, 11),
    T(22, 17),   T(28, 20),   T(33, 25),
];
pub const BISHOP_MOBILITY: IndexTable<14> = [
    T(-48,-59), T(-20,-23),
    T(16, -3),  T(26, 13),
    T(38, 24),  T(51, 42),
    T(55, 54),  T(63, 57),
    T(63, 65),  T(68, 73),
    T(81, 78),  T(81, 86),
    T(91, 88),  T(98, 97),
];

pub const ROOK_MOBILITY: IndexTable<15> = [
    T(-60,-78), T(-20,-17), T(2, 23),
    T(3, 39),   T(3, 70),   T(11, 99),
    T(22,103),  T(31,121),  T(40,134),
    T(40,139),  T(41,158),  T(48,164),
    T(57,168),  T(57,169),  T(62,172),
];

pub const QUEEN_MOBILITY: IndexTable<28> = [
    T(-30,-48), T(-12,-30), T(-8, -7),  T( -9, 19),
    T(20, 40),  T(23, 55),  T(23, 59),  T(35, 75),
    T(38, 78),  T(53, 96),  T(64, 96),  T(65,100),
    T(65,121),  T(66,127),  T(67,131),  T(67,133),
    T(72,136),  T(72,141),  T(77,147),  T(79,150),
    T(93,151),  T(108,168), T(108,168), T(108,171),
    T(110,182), T(114,182), T(114,192), T(116,219),
];

pub const ROOK_OPEN_FILE: FileTable = [
    T(56, 0), T(56, 0), T(56, 0), T(56, 0), T(56, 0), T(56, 0), T(56, 0), T(56, 0),
];

pub const ROOK_SEMIOPEN_FILE: FileTable = [
    T(36, 0), T(36, 0), T(36, 0), T(36, 0), T(36, 0), T(36, 0), T(36, 0), T(36, 0),
];
pub const QUEEN_OPEN_FILE: FileTable = [
    T(-6, 0), T(-6, 0), T(-6, 0), T(-6, 0), T(-6, 0), T(-6, 0), T(-6, 0), T(-6, 0),
];
pub const QUEEN_SEMIOPEN_FILE: FileTable = [
    T(11, 0), T(11, 0), T(11, 0), T(11, 0), T(11, 0), T(11, 0), T(11, 0), T(11, 0),
];

/*----------------------------------------------------------------*/

pub const PASSED_PAWN: RankTable = [
    T(0, 0),
    T(-10, 8),
    T(-13, 13),
    T(-7, 19),
    T(1, 27),
    T(17, 35),
    T(31, 57),
    T(0, 0),
];

pub const BACKWARDS_PAWN: T = T(-1, -2);
pub const ISOLATED_PAWN: T = T(-9, -1);
pub const DOUBLED_PAWN: T = T(-5, -3);

/*----------------------------------------------------------------*/

pub const PAWN_MINOR_THREAT: T = T(21, 37);
pub const PAWN_MAJOR_THREAT: T = T(22, 56);
pub const MINOR_MAJOR_THREAT: T = T(55, 86);

/*----------------------------------------------------------------*/

pub const CENTER_CONTROL: T = T(3, 0);

/*----------------------------------------------------------------*/

pub const KNIGHT_ATTACK: u8 = 2;
pub const BISHOP_ATTACK: u8 = 2;
pub const ROOK_ATTACK: u8 = 3;
pub const QUEEN_ATTACK: u8 = 5;

pub const KING_DANGER: [i16; 100] = [
    0,   0,   1,   2,   3,   5,   7,   9,   12,  15,
    18,  22,  26,  30,  35,  39,  44,  50,  56,  62,
    68,  75,  82,  85,  89,  97,  105, 113, 122, 131,
    140, 150, 169, 180, 191, 202, 213, 225, 237, 248,
    260, 272, 283, 295, 307, 319, 330, 342, 354, 366,
    377, 389, 401, 412, 424, 436, 448, 459, 471, 483,
    494, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500
];

/*----------------------------------------------------------------*/

pub const BISHOP_PAIR: T = T(29, 130);

pub const PAWN_VALUE: T = T(100, 182);
pub const KNIGHT_VALUE: T = T(275, 371);
pub const BISHOP_VALUE: T = T(300, 443);
pub const ROOK_VALUE: T = T(550, 643);
pub const QUEEN_VALUE: T =  T(1850, 1743);

pub const PAWN_PSQT: SquareTable = [
    T(0, 0),   T(0, 0),   T(0, 0),   T(0, 0),   T(0, 0),   T (0, 0),  T(0, 0),   T(0, 0),
    T(25, 51), T(14, 56), T(17, 52), T(0, 54),  T(13, 54), T(28, 50), T(30, 45), T(22, 43),
    T(17, 46), T(5, 49),  T(2, 45),  T(-5, 46), T(1, 46),  T(15, 44), T(12, 42), T(14, 41),
    T(24, 48), T(16, 47), T(18, 40), T(11, 36), T(16, 41), T(20, 43), T(24, 41), T(21, 39),
    T(35, 50), T(21, 50), T(19, 44), T(28, 31), T(30, 39), T(35, 38), T(31, 43), T(30, 41),
    T(41, 63), T(33, 54), T(37, 49), T(37, 44), T(41, 49), T(46, 40), T(59, 48), T(40, 45),
    T(91, 63), T(50, 64), T(41, 64), T(57, 50), T(15, 59), T(41, 53), T(31, 61), T(13, 55),
    T(0, 0),   T(0, 0),   T(0, 0),   T(0, 0),   T(0, 0),   T(0, 0),   T(0, 0),   T(0, 0),
];

pub const KNIGHT_PSQT: SquareTable = [
    T(-23, -18), T(-15, -12), T(-2, 27),   T(-11, -23), T(-11, -23), T(-2, -27),  T(-15, -12), T(-23, -18),
    T(-2, -20),  T(-4, -22),  T(11, 18),   T(16, 27),   T(16, 27),   T(11, 18),   T(-4, -22),  T(-2, -20),
    T(-9, -29),  T(14, 22),   T(10, 35),   T(20, 39),   T(20, 39),   T(10, 35),   T(14, 22),   T(-9, -29),
    T(-12, -33), T(35, 28),   T(19, 52),   T(22, 41),   T(22, 41),   T(19, 52),   T(35, 28),   T(-12, -33),
    T(-31, -37), T(13, 35),   T(22, 47),   T(26, 49),   T(26, 49),   T(22, 47),   T(13, 35),   T(-31, -37),
    T(-20, -45), T(18, 40),   T(24, 59),   T(33, 51),   T(33, 51),   T(24, 59),   T(18, 40),   T(-20, -45),
    T(-15, -45), T(-20, -48), T(34, 34),   T(24, 40),   T(24, 40),   T(34, 34),   T(-20, -48), T(-15, -45),
    T(-23, -13), T(-5, -31),  T(-11, -37), T(-4, -44),  T(-4, -44),  T(-11, -37), T(-5, 31),   T(-23, -13),
];


pub const BISHOP_PSQT: SquareTable = [
    T(-28, -36), T(43, 33),  T(23, 35), T(23, 28), T(23, 29), T(32, 35),  T(45, 22),  T(-22, -25),
    T(32, 35),   T(23, 32),  T(33, 36), T(17, 41), T(26, 36), T(24, 33),  T(36, 26),  T(34, 34),
    T(20, 36),   T(29, 32),  T(23, 43), T(28, 42), T(21, 41), T(23, 39),  T(12, 32),  T(33, 35),
    T(19, 39),   T(21, 37),  T(31, 46), T(24, 43), T(21, 48), T(15, 43),  T(28, 40),  T(27, 31),
    T(30, 38),   T(30, 43),  T(20, 42), T(31, 45), T(26, 53), T(28, 43),  T(18, 46),  T(13, 47),
    T(25, 44),   T(29, 47),  T(23, 51), T(25, 42), T(32, 44), T(22, 52),  T(31, 44),  T(31, 40),
    T(13, 45),   T(16, 53),  T(30, 38), T(13, 45), T(8, 48),  T(35, 43),  T(-20, 54), T(3, 57),
    T(-6, -47),  T(-15, 52), T(-6, 51), T(-4, 48), T(-3, 52), T(-10, 45), T(9, 50),   T(0, -45),
];

pub const ROOK_PSQT: SquareTable = [
    T(4, 35),  T(11, 34), T(6, 41),  T(13, 34), T(13, 30), T(14, 33), T(19, 29), T(6, 37),
    T(3, 40),  T(0, 40),  T(5, 41),  T(14, 28), T(12, 26), T(12, 39), T(19, 34), T(16, 34),
    T(0, 48),  T(-1, 51), T(8, 47),  T(10, 41), T(17, 32), T(18, 42), T(17, 42), T(13, 42),
    T(-1, 59), T(1, 59),  T(4, 60),  T(12, 47), T(21, 48), T(10, 58), T(27, 53), T(9, 54),
    T(14, 61), T(15, 64), T(30, 58), T(34, 50), T(22, 55), T(45, 55), T(27, 65), T(24, 62),
    T(20, 64), T(21, 68), T(39, 60), T(30, 56), T(52, 50), T(46, 60), T(41, 64), T(41, 59),
    T(32, 62), T(22, 66), T(29, 70), T(32, 57), T(45, 53), T(57, 58), T(19, 70), T(12, 70),
    T(22, 59), T(17, 67), T(2, 69),  T(11, 58), T(18, 53), T(30, 60), T(28, 65), T(15, 67),
];

pub const QUEEN_PSQT: SquareTable = [
    T(24, 35),  T(31, 30),  T(29, 30), T(33, 49),  T(33, 44), T(16, 34), T(18, 33),  T(31, 26),
    T(22, 61),  T(35, 41),  T(30, 53), T(31, 47),  T(31, 59), T(32, 47), T(43, 18),  T(50, 30),
    T(20, 71),  T(29, 66),  T(28, 69), T(26, 72),  T(29, 63), T(34, 81), T(38, 71),  T(50, 47),
    T(23, 97),  T(16, 85),  T(22, 90), T(21, 91),  T(22, 84), T(27, 85), T(31, 85),  T(43, 75),
    T(21, 98),  T(25, 101), T(31, 87), T(12, 100), T(23, 97), T(33, 91), T(32, 91),  T(46, 66),
    T(26, 102), T(20, 103), T(35, 94), T(25, 90),  T(33, 98), T(35, 94), T(31, 107), T(41, 88),
    T(26, 94),  T(24, 102), T(31, 97), T(34, 96),  T(39, 96), T(45, 94), T(31, 104), T(45, 97),
    T(39, 75),  T(40, 78),  T(29, 86), T(16, 80),  T(43, 83), T(56, 78), T(68, 87),  T(57, 82),
];

pub const KING_PSQT: SquareTable = [
    T(271, 1),   T(327, 45),  T(271, 85),  T(198, 76),  T(198, 76),  T(271, 85),  T(327, 45),  T(271, 1),
    T(278, 53),  T(303, 100), T(234, 133), T(179, 135), T(179, 135), T(234, 133), T(303, 100), T(278, 53),
    T(195, 88),  T(258, 130), T(169, 169), T(120, 175), T(120, 175), T(169, 169), T(258, 130), T(195, 88),
    T(164, 103), T(190, 156), T(138, 172), T(98, 172),  T(98, 172),  T(138, 172), T(190, 156), T(164, 103),
    T(154, 96),  T(179, 166), T(105, 199), T(70, 199),  T(70, 199),  T(105, 199), T(179, 166), T(154, 96),
    T(123, 92),  T(145, 172), T(81, 184),  T(31, 191),  T(31, 191),  T(81, 184),  T(145, 172), T(123, 92),
    T(88, 47),   T(120, 121), T(65, 116),  T(33, 131),  T(33, 131),  T(65, 116),  T(120, 121), T(88, 47),
    T(59, 11),   T(89, 59),   T(45, 73),   T(-1, 78),   T(-1, 78),   T(45, 73),   T(89, 59),   T(59, 11),
];