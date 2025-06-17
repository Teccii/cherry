use std::{fmt, ops::*};
use cozy_chess::*;
use super::*;

/*----------------------------------------------------------------*/

#[macro_export]
macro_rules! table {
    ($(($mg:expr, $eg:expr),)*) => {
        IndexTable::new([$(T($mg, $eg),)*])
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(Default)]
pub struct T(pub i16, pub i16);

impl T {
    #[inline(always)]
    pub fn new_mg(score: i16) -> T {
        T(score, 0)
    }

    #[inline(always)]
    pub fn new_eg(score: i16) -> T {
        T(0, score)
    }

    #[inline(always)]
    pub fn scale(self, phase: u16) -> Score {
        let phase = (phase * TAPER_SCALE + TOTAL_PHASE / 2) / TOTAL_PHASE;
        let score = (self.0 as i32 * (TAPER_SCALE - phase) as i32 + self.1 as i32 * phase as i32) / TAPER_SCALE as i32;

        Score(score as i16)
    }

    #[inline(always)]
    pub const fn mg(self) -> T {
        T(self.0, 0)
    }

    #[inline(always)]
    pub const fn eg(self) -> T {
        T(0, self.1)
    }

    pub const ZERO: T = T(0, 0);
}

impl fmt::Display for T {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T({}, {})", self.0, self.1)
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_tapered_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<T> for T {
            type Output = T;

            #[inline(always)]
            fn $fn(self, rhs: T) -> Self::Output {
                T(self.0.$fn(rhs.0), self.1.$fn(rhs.1))
            }
        }
    )*};
}

macro_rules! impl_tapered_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<T> for T {
            #[inline(always)]
            fn $fn(&mut self, rhs: T) {
                self.0.$fn(rhs.0);
                self.1.$fn(rhs.1);
            }
        }
    )*};
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

        impl $trait<T> for i16 {
            type Output = T;

            #[inline(always)]
            fn $fn(self, rhs: T) -> Self::Output {
                T(self.$fn(rhs.0), self.$fn(rhs.1))
            }
        }
    )*};
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
    )*};
}

/*----------------------------------------------------------------*/

impl_tapered_ops! {
    Add, add;
    Sub, sub;
}

impl_tapered_assign_ops! {
    AddAssign, add_assign;
    SubAssign, sub_assign;
}

impl_tapered_i16_ops! {
    Mul, mul;
    Div, div;
}

impl_tapered_i16_assign_ops! {
    MulAssign, mul_assign;
    DivAssign, div_assign;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IndexTable<const COUNT: usize>(pub [T; COUNT]);

pub type FileTable = IndexTable<{File::NUM}>;
pub type RankTable = IndexTable<{Rank::NUM}>;
pub type SquareTable = IndexTable<{Square::NUM}>;

impl<const COUNT: usize> IndexTable<COUNT> {
    #[inline(always)]
    pub const fn new(table: [T; COUNT]) -> Self {
        Self(table)
    }
}

impl<const COUNT:usize> fmt::Display for IndexTable<COUNT> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        
        for i in 0..COUNT {
            write!(f, "{}", self[i])?;
        }
        
        write!(f, "]")
    }
}

impl<const COUNT: usize> Index<usize> for IndexTable<COUNT> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const COUNT: usize> IndexMut<usize> for IndexTable<COUNT> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_table_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait for IndexTable<COUNT> {
            type Output = Self;

            fn $fn(self, rhs: Self) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < COUNT {
                    result.0[i] = T(self.0[i].0.$fn(rhs.0[i].0), self.0[i].1.$fn(rhs.0[i].1));
                    i += 1;
                }

                result
            }
        }
    )*};
}

macro_rules! impl_table_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait for IndexTable<COUNT> {
            fn $fn(&mut self, rhs: Self) {
                let mut i = 0;

                while i < COUNT {
                    self.0[i] = T(self.0[i].0.$op(rhs.0[i].0), self.0[i].1.$op(rhs.0[i].1));
                    i += 1;
                }
            }
        }
    )*};
}

macro_rules! impl_table_tapered_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait<T> for IndexTable<COUNT> {
            type Output = Self;

            fn $fn(self, rhs: T) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < COUNT {
                    result[i] = T(self.0[i].0.$fn(rhs.0), self.0[i].1.$fn(rhs.1));
                    i += 1;
                }

                result
            }
        }
    )*};
}

macro_rules! impl_table_tapered_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait<T> for IndexTable<COUNT> {
            fn $fn(&mut self, rhs: T) {
                let mut i = 0;

                while i < COUNT {
                    self.0[i] = T(self.0[i].0.$op(rhs.0), self.0[i].1.$op(rhs.1));
                    i += 1;
                }
            }
        }
    )*};
}


macro_rules! impl_table_i16_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl<const COUNT: usize> $trait<i16> for IndexTable<COUNT> {
            type Output = Self;

            fn $fn(self, rhs: i16) -> Self::Output {
                let mut result = self;
                let mut i = 0;

                while i < COUNT {
                    result.0[i] = T(self.0[i].0.$fn(rhs), self.0[i].1.$fn(rhs));
                    i += 1;
                }

                result
            }
        }
    )*};
}

macro_rules! impl_table_i16_assign_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {$(
        impl<const COUNT: usize> $trait<i16> for IndexTable<COUNT> {
            fn $fn(&mut self, rhs: i16) {
                let mut i = 0;

                while i < COUNT {
                    self.0[i] = T(self.0[i].0.$op(rhs), self.0[i].1.$op(rhs));
                    i += 1;
                }
            }
        }
    )*};
}

/*----------------------------------------------------------------*/

impl_table_ops! {
    Add, add;
    Sub, sub;
}

impl_table_assign_ops! {
    AddAssign, add_assign, add;
    SubAssign, sub_assign, sub;
}

impl_table_tapered_ops! {
    Add, add;
    Sub, sub;
    Mul, mul;
    Div, div;
}

impl_table_tapered_assign_ops! {
    AddAssign, add_assign, add;
    SubAssign, sub_assign, sub;
}

impl_table_i16_ops! {
    Mul, mul;
    Div, div;
}

impl_table_i16_assign_ops! {
    MulAssign, mul_assign, mul;
    DivAssign, div_assign, div;
}


/*----------------------------------------------------------------*/

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
    ($($elem:ident : $ty:ty = $default:expr,)*) => {
        #[derive(Debug, Copy, Clone)]
        pub struct EvalWeights {
            $(pub $elem: $ty),*
        }
        
        impl Default for EvalWeights {
            fn default() -> Self {
                EvalWeights { $($elem: $default),* }
            }
        }
        
        impl fmt::Display for EvalWeights {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                $(write!(f, "{}: {}", stringify!($elem), self.$elem)?;)*
                
                Ok(())
            }
        }
    }
}

weights! {
    bishop_pair: T = BISHOP_PAIR,

    pawn_value: T = PAWN_VALUE,
    knight_value: T = KNIGHT_VALUE,
    bishop_value: T = BISHOP_VALUE,
    rook_value: T = ROOK_VALUE,
    queen_value: T = QUEEN_VALUE,

    pawn_psqt: SquareTable = PAWN_PSQT,
    knight_psqt: SquareTable = KNIGHT_PSQT,
    bishop_psqt: SquareTable = BISHOP_PSQT,
    rook_psqt: SquareTable = ROOK_PSQT,
    queen_psqt: SquareTable = QUEEN_PSQT,
    king_psqt: SquareTable = KING_PSQT,

    knight_mobility: IndexTable<9> = KNIGHT_MOBILITY,
    bishop_mobility: IndexTable<14> = BISHOP_MOBILITY,
    rook_mobility: IndexTable<15> = ROOK_MOBILITY,
    queen_mobility: IndexTable<28> = QUEEN_MOBILITY,

    rook_open_file: FileTable = ROOK_OPEN_FILE,
    rook_semiopen_file: FileTable = ROOK_SEMIOPEN_FILE,
    queen_open_file: FileTable = QUEEN_OPEN_FILE,
    queen_semiopen_file: FileTable = QUEEN_SEMIOPEN_FILE,

    knight_attack: T = KNIGHT_ATTACK,
    bishop_attack: T = BISHOP_ATTACK,
    rook_attack: T = ROOK_ATTACK,
    queen_attack: T = QUEEN_ATTACK,

    pawn_minor_threat: T = PAWN_MINOR_THREAT,
    pawn_major_threat: T = PAWN_MAJOR_THREAT,
    minor_major_threat: T = MINOR_MAJOR_THREAT,

    passed_pawn: RankTable = PASSED_PAWN,
    phalanx: RankTable = PHALANX,
    backwards_pawn: T = BACKWARDS_PAWN,
    isolated_pawn: T = ISOLATED_PAWN,
    doubled_pawn: T = DOUBLED_PAWN,
    support: T = SUPPORT,

    center_control: T = CENTER_CONTROL,
}

/*----------------------------------------------------------------*/

pub const BISHOP_PAIR: T = T(21, 71);

pub const KNIGHT_MOBILITY: IndexTable<9> = table![
    (-62, -81),
    (-53, -56),
    (-12, -31),
    (-4, -16),
    (3, 5),
    (13, 11),
    (22, 17),
    (28, 20),
    (33, 25),
];
pub const BISHOP_MOBILITY: IndexTable<14> = table![
    (-48, -59),
    (-20, -23),
    (16, -3),
    (26, 13),
    (38, 24),
    (51, 42),
    (55, 54),
    (63, 57),
    (63, 65),
    (68, 73),
    (81, 78),
    (81, 86),
    (91, 88),
    (98, 97),
];

pub const ROOK_MOBILITY: IndexTable<15> = table![
    (-60, -78),
    (-20, -17),
    (2, 23),
    (3, 39),
    (3, 70),
    (11, 99),
    (22, 103),
    (31, 121),
    (40, 134),
    (40, 139),
    (41, 158),
    (48, 164),
    (57, 168),
    (57, 169),
    (62, 172),
];
pub const QUEEN_MOBILITY: IndexTable<28> = table![
    (-30, -48),
    (-12, -30),
    (-8, -7),
    (-9, 19),
    (20, 40),
    (23, 55),
    (23, 59),
    (35, 75),
    (38, 78),
    (53, 96),
    (64, 96),
    (65, 100),
    (65, 121),
    (66, 127),
    (67, 131),
    (67, 133),
    (72, 136),
    (72, 141),
    (77, 147),
    (79, 150),
    (93, 151),
    (108, 168),
    (108, 168),
    (108, 171),
    (110, 182),
    (114, 182),
    (114, 192),
    (116, 219),
];

pub const ROOK_OPEN_FILE: FileTable = table![
    (19, 0),
    (19, 0),
    (19, 0),
    (27, 0),
    (27, 0),
    (19, 0),
    (19, 0),
    (19, 0),
];
pub const ROOK_SEMIOPEN_FILE: FileTable = table![
    (5, 0),
    (5, 0),
    (5, 0),
    (9, 0),
    (9, 0),
    (5, 0),
    (5, 0),
    (5, 0),
];
pub const QUEEN_OPEN_FILE: FileTable = table![
    (-7, 0),
    (-7, 0),
    (-7, 0),
    (-8, 0),
    (-8, 0),
    (-7, 0),
    (-7, 0),
    (-7, 0),
];
pub const QUEEN_SEMIOPEN_FILE: FileTable = table![
    (4, 0),
    (4, 0),
    (4, 0),
    (7, 0),
    (7, 0),
    (4, 0),
    (4, 0),
    (4, 0),
];

/*----------------------------------------------------------------*/

pub const PASSED_PAWN: RankTable = table![
    (0, 0),
    (10, 28),
    (17, 33),
    (15, 41),
    (62, 72),
    (168, 177),
    (276, 260),
    (0, 0),
];
pub const PHALANX: RankTable = table![
    (0, 0),
    (5, 11),
    (7, 14),
    (8, 19),
    (11, 21),
    (16, 31),
    (34, 83),
    (0, 0),
];
pub const BACKWARDS_PAWN: T = T(-10, -25);
pub const ISOLATED_PAWN: T = T(-5, -20);
pub const DOUBLED_PAWN: T = T(-5, -15);
pub const SUPPORT: T = T(3, 7);


/*----------------------------------------------------------------*/

pub const PAWN_MINOR_THREAT: T = T(31, 71);
pub const PAWN_MAJOR_THREAT: T = T(46, 92);
pub const MINOR_MAJOR_THREAT: T = T(51, 89);

pub const KNIGHT_ATTACK: T = T(6, 7);
pub const BISHOP_ATTACK: T = T(5, 17);
pub const ROOK_ATTACK: T = T(16, 3);
pub const QUEEN_ATTACK: T = T(1, 21);

pub const CENTER_CONTROL: T = T(3, 0);

/*----------------------------------------------------------------*/

pub const PAWN_VALUE: T = T(124, 206);
pub const KNIGHT_VALUE: T = T(781, 854);
pub const BISHOP_VALUE: T = T(825, 915);
pub const ROOK_VALUE: T = T(1276, 1380);
pub const QUEEN_VALUE: T = T(2538, 2682);

pub const PAWN_PSQT: SquareTable = table! [
    (0, 0),     (0, 0),     (0, 0),     (0, 0),     (0, 0),     (0, 0),     (0, 0),     (0, 0),
    (3, -10),   (3, -6),    (10, 10),   (19, 0),    (16, 14),   (19, 7),    (7, -5),    (-5, -19),
    (-9, -10),  (-15, -10), (11, -10),  (15, 4),    (32, 4),    (22, 3),    (5, -6),    (-22, -4),
    (-4, 6),    (-23, -2),  (6, -8),    (20, -4),   (40, -13),  (17, -12),  (4, -10),   (-8, -9),
    (13, 10),   (4, 5),     (-13, 4),   (1, -5),    (11, -5),   (-2, -5),   (-13, 14),  (5, 9),
    (5, 28),    (-12, 20),  (-7, 21),   (22, 28),   (-8, 30),   (-5, 7),    (-15, 6),   (-8, 13),
    (-7, 0),    (7, -11),   (-3, 12),   (-13, 21),  (5, 25),    (-16, 19),  (10, 4),    (-8, 7),
    (0, 0),     (0, 0),     (0, 0),     (0, 0),     (0, 0),     (0, 0),     (0, 0),     (0, 0),
];

pub const KNIGHT_PSQT: SquareTable = table! [
    (-175, -96), (-92, -65), (-74, -49), (-73, -21), (-73, -21), (-74, -49), (-92, -65), (-175, -96),
    (-77, -67),  (-41, -54), (-27, -18), (-15, 8),   (-15, 8),   (-27, -18), (-41, -54), (-77, -67),
    (-61, -40),  (-17, -27), (6, -8),    (12, 29),   (12, 29),   (6, -8),    (-17, -27), (-61, -40),
    (-35, -35),  (8, -2),    (40, 13),   (49, 28),   (49, 28),   (40, 13),   (8, -2),    (-35, -35),
    (-34, -45),  (13, -16),  (44, 9),    (51, 39),   (51, 39),   (44, 9),    (13, -16),  (-34, -45),
    (-9, -51),   (22, -44),  (58, -16),  (53, 17),   (53, 17),   (58, -16),  (22, -44),  (-9, -51),
    (-67, -69),  (-27, -50), (4, -51),   (37, 12),   (37, 12),   (4, -51),   (-27, -50), (-67, -69),
    (-201, -100), (-83, -88), (-56, -56), (-26, -17), (-26, -17), (-56, -56), (-83, -88), (-201, -100),
];


pub const BISHOP_PSQT: SquareTable = table! [
    (-53, -57), (-5, -30),  (-8, -37),  (-23, -12), (-23, -12), (-8, -37),  (-5, -30),  (-53, -57),
    (-15, -37), (8, -13),   (19, -17),  (4, 1),      (4, 1),     (19, -17),  (8, -13),   (-15, -37),
    (-7, -16),  (21, -1),   (-5, -2),   (17, 10),    (17, 10),   (-5, -2),   (21, -1),   (-7, -16),
    (-5, -20),  (11, -6),   (25, 25),    (39, 17),    (39, 17),   (25, 25),    (11, -6),   (-5, -20),
    (-12, -17), (29, -1),   (22, -14),  (31, 15),    (31, 15),   (22, -14),  (29, -1),   (-12, -17),
    (-16, -30), (6, 6),     (1, 4),     (11, 6),     (11, 6),    (1, 4),     (6, 6),     (-16, -30),
    (-17, -31), (-14, -20), (5, -1),    (2, 1),      (2, 1),     (5, -1),    (-14, -20), (-17, -31),
    (-48, -46), (1, -42),   (-14, -37), (-23, -24),  (-23, -24), (-14, -37), (1, -42),   (-48, -46),
];

pub const ROOK_PSQT: SquareTable = table! [
    (-31, -9),  (-20, -13), (-14, -10), (-5, -9),   (-5, -9),   (-14, -10), (-20, -13), (-31, -9),
    (-21, -12), (-13, -9),  (-8, -1),   (6, -2),    (6, -2),    (-8, -1),   (-13, -9),  (-21, -12),
    (-25, 6),   (-11, -8),  (-1, -2),   (3, -6),    (3, -6),    (-1, -2),   (-11, -8),  (-25, 6),
    (-13, -6),  (-5, 1),    (-4, -9),   (-6, 7),    (-6, 7),    (-4, -9),   (-5, 1),    (-13, -6),
    (-27, -5),  (-15, 8),   (-4, 7),    (3, -6),    (3, -6),    (-4, 7),    (-15, 8),   (-27, -5),
    (-22, 6),   (-2, 1),    (6, -7),    (12, 10),   (12, 10),   (6, -7),    (-2, 1),    (-22, 6),
    (-2, 4),    (12, 5),    (16, 20),   (18, -5),   (18, -5),   (16, 20),   (12, 5),    (-2, 4),
    (-17, 18),  (-19, -11),   (-1, 19),   (9, 13),    (9, 13),    (-1, 19),   (-19, -11),   (-17, 18),
];

pub const QUEEN_PSQT: SquareTable = table! [
    (3, -69),  (-5, -57), (-5, -47), (4, -26),  (4, -26),  (-5, -47), (-5, -57), (3, -69),
    (-3, -55), (5, -31),  (8, -22),  (12, -4), (12, -4),  (8, -22),  (5, -31),  (-3, -55),
    (-3, -39), (6, -18),  (13, -9),  (7, 3),   (7, 3),    (13, -9),  (6, -18),  (-3, -39),
    (4, -23),  (5, -3),   (9, 13),   (8, 24),  (8, 24),   (9, 13),   (5, -3),   (4, -23),
    (-11, -29),  (14, -6),  (12, 9),   (5, 21),  (5, 21),   (12, 9),   (14, -6),  (-11, -29),
    (-4, -38), (10, -18), (6, -12),  (8, 1),   (8, 1),    (6, -12),  (10, -18), (-4, -38),
    (-5, -50), (6, -27),  (10, -24), (8, -8),  (8, -8),   (10, -24), (6, -27),  (-5, -50),
    (-2, -75), (-2, -52), (1, -43),  (-2, -36), (-2, -36), (1, -43),  (-2, -52), (-2, -75),
];

pub const KING_PSQT: SquareTable = table! [
    (271, 1),  (327, 45), (271, 85), (198, 76), (198, 76), (271, 85), (327, 45), (271, 1),
    (278, 53), (303, 100), (234, 133), (179, 135), (179, 135), (234, 133), (303, 100), (278, 53),
    (195, 88), (258, 130), (169, 169), (120, 175), (120, 175), (169, 169), (258, 130), (195, 88),
    (164, 103), (190, 156), (138, 172), (98, 172), (98, 172), (138, 172), (190, 156), (164, 103),
    (154, 96), (179, 166), (105, 199), (70, 199), (70, 199), (105, 199), (179, 166), (154, 96),
    (123, 92), (145, 172), (81, 184), (31, 191), (31, 191), (81, 184), (145, 172), (123, 92),
    (88, 47), (120, 121), (65, 116), (33, 131), (33, 131), (65, 116), (120, 121), (88, 47),
    (59, 11), (89, 59), (45, 73), (-3, 78), (-3, 78), (45, 73), (89, 59), (59, 11),
];

pub const fn calc_piece_table(table: SquareTable, value: T) -> SquareTable {
    let mut result = table;
    let mut i = 0;
    
    while i < Square::NUM {
        result.0[i] = T(value.0 + result.0[i].0, value.1 + result.0[i].1);
        i += 1;
    }
    
    result
}

pub const PAWN_TABLE: SquareTable = calc_piece_table(PAWN_PSQT, PAWN_VALUE);
pub const KNIGHT_TABLE: SquareTable = calc_piece_table(KNIGHT_PSQT, KNIGHT_VALUE);
pub const BISHOP_TABLE: SquareTable = calc_piece_table(BISHOP_PSQT, BISHOP_VALUE);
pub const ROOK_TABLE: SquareTable = calc_piece_table(ROOK_PSQT, ROOK_VALUE);
pub const QUEEN_TABLE: SquareTable = calc_piece_table(QUEEN_PSQT, QUEEN_VALUE);
pub const KING_TABLE: SquareTable = KING_PSQT;

/*----------------------------------------------------------------*/

pub const fn king_zone(sq: Square, color: Color) -> BitBoard {
    const fn calc_zone(sq: Square, color: Color) -> BitBoard {
        let moves = BitBoard(get_king_moves(sq).0 | sq.bitboard().0);
        
        match color {
            Color::White => BitBoard(moves.0 | (moves.0 << 8)),
            Color::Black => BitBoard(moves.0 | (moves.0 >> 8)),
        }
    }
    
    const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];
        let mut i = 0;
        
        while i < Color::NUM {
            let mut j = 0;
            while j < Square::NUM {
                table[i][j] = calc_zone(Square::index_const(j), Color::index_const(i));
                
                j += 1;
            }
            
            i += 1;
        }
        
        table
    };
    
    TABLE[color as usize][sq as usize]
}