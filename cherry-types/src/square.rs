use crate::{Bitboard, Color, File, Rank};
use std::{fmt, str::FromStr};

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    #[inline]
    pub const fn new(file: File, rank: Rank) -> Square {
        Square::index(((rank as usize) << 3) | file as usize)
    }
    
    #[inline]
    pub const fn index(i: usize) -> Square {
        match i {
            0 => Square::A1,
            1 => Square::B1,
            2 => Square::C1,
            3 => Square::D1,
            4 => Square::E1,
            5 => Square::F1,
            6 => Square::G1,
            7 => Square::H1,

            8 => Square::A2,
            9 => Square::B2,
            10 => Square::C2,
            11 => Square::D2,
            12 => Square::E2,
            13 => Square::F2,
            14 => Square::G2,
            15 => Square::H2,

            16 => Square::A3,
            17 => Square::B3,
            18 => Square::C3,
            19 => Square::D3,
            20 => Square::E3,
            21 => Square::F3,
            22 => Square::G3,
            23 => Square::H3,

            24 => Square::A4,
            25 => Square::B4,
            26 => Square::C4,
            27 => Square::D4,
            28 => Square::E4,
            29 => Square::F4,
            30 => Square::G4,
            31 => Square::H4,

            32 => Square::A5,
            33 => Square::B5,
            34 => Square::C5,
            35 => Square::D5,
            36 => Square::E5,
            37 => Square::F5,
            38 => Square::G5,
            39 => Square::H5,

            40 => Square::A6,
            41 => Square::B6,
            42 => Square::C6,
            43 => Square::D6,
            44 => Square::E6,
            45 => Square::F6,
            46 => Square::G6,
            47 => Square::H6,

            48 => Square::A7,
            49 => Square::B7,
            50 => Square::C7,
            51 => Square::D7,
            52 => Square::E7,
            53 => Square::F7,
            54 => Square::G7,
            55 => Square::H7,

            56 => Square::A8,
            57 => Square::B8,
            58 => Square::C8,
            59 => Square::D8,
            60 => Square::E8,
            61 => Square::F8,
            62 => Square::G8,
            63 => Square::H8,

            _ => panic!("Square::index(): Index out of bounds")
        }
    }

    #[inline]
    pub const fn try_index(i: usize) -> Option<Square> {
        match i {
            0 => Some(Square::A1),
            1 => Some(Square::B1),
            2 => Some(Square::C1),
            3 => Some(Square::D1),
            4 => Some(Square::E1),
            5 => Some(Square::F1),
            6 => Some(Square::G1),
            7 => Some(Square::H1),

            8 => Some(Square::A2),
            9 => Some(Square::B2),
            10 => Some(Square::C2),
            11 => Some(Square::D2),
            12 => Some(Square::E2),
            13 => Some(Square::F2),
            14 => Some(Square::G2),
            15 => Some(Square::H2),

            16 => Some(Square::A3),
            17 => Some(Square::B3),
            18 => Some(Square::C3),
            19 => Some(Square::D3),
            20 => Some(Square::E3),
            21 => Some(Square::F3),
            22 => Some(Square::G3),
            23 => Some(Square::H3),

            24 => Some(Square::A4),
            25 => Some(Square::B4),
            26 => Some(Square::C4),
            27 => Some(Square::D4),
            28 => Some(Square::E4),
            29 => Some(Square::F4),
            30 => Some(Square::G4),
            31 => Some(Square::H4),

            32 => Some(Square::A5),
            33 => Some(Square::B5),
            34 => Some(Square::C5),
            35 => Some(Square::D5),
            36 => Some(Square::E5),
            37 => Some(Square::F5),
            38 => Some(Square::G5),
            39 => Some(Square::H5),

            40 => Some(Square::A6),
            41 => Some(Square::B6),
            42 => Some(Square::C6),
            43 => Some(Square::D6),
            44 => Some(Square::E6),
            45 => Some(Square::F6),
            46 => Some(Square::G6),
            47 => Some(Square::H6),

            48 => Some(Square::A7),
            49 => Some(Square::B7),
            50 => Some(Square::C7),
            51 => Some(Square::D7),
            52 => Some(Square::E7),
            53 => Some(Square::F7),
            54 => Some(Square::G7),
            55 => Some(Square::H7),

            56 => Some(Square::A8),
            57 => Some(Square::B8),
            58 => Some(Square::C8),
            59 => Some(Square::D8),
            60 => Some(Square::E8),
            61 => Some(Square::F8),
            62 => Some(Square::G8),
            63 => Some(Square::H8),

            _ => None
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn offset(self, dx: i8, dy: i8) -> Square {
        let i = self.file() as i8 + dx;
        let j = self.rank() as i8 + dy;

        if i < 0 || i >= File::COUNT as i8 {
            panic!("Square::offset(): New file index out of bounds");
        }

        if j < 0 || j >= Rank::COUNT as i8 {
            panic!("Square::offset(): New rank index out of bounds");
        }
        
        Square::new(File::index(i as usize), Rank::index(j as usize))
    }

    #[inline]
    pub const fn try_offset(self, dx: i8, dy: i8) -> Option<Square> {
        let i = self.file() as i8 + dx;
        let j = self.rank() as i8 + dy;

        if i < 0 || i >= File::COUNT as i8 {
            return None;
        }

        if j < 0 || j >= Rank::COUNT as i8 {
            return None;
        }

        Some(Square::new(File::index(i as usize), Rank::index(j as usize)))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn flip_rank(self) -> Square {
        Square::index(self as usize ^ 56)
    }
    
    #[inline]
    pub const fn flip_file(self) -> Square {
        Square::index(self as usize ^ 7)
    }
    
    #[inline]
    pub const fn relative_to(self, color: Color) -> Square {
        match color {
            Color::White => self,
            Color::Black => self.flip_rank(),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn dist(self, other: Square) -> u8 {
        ((other.rank() as i8 - self.rank() as i8).abs() + (other.file() as i8 - self.file() as i8).abs()) as u8
    }

    #[inline]
    pub const fn center_dist(self) -> u8 {
        const TABLE: [u8; Square::COUNT] = [
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
    
    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn file(self) -> File {
        File::index(self as usize & 7)
    }
    
    #[inline]
    pub const fn rank(self) -> Rank {
        Rank::index(self as usize >> 3)
    }

    #[inline]
    pub const fn bitboard(self) -> Bitboard {
        Bitboard(1u64 << self as u8)
    }

    /*----------------------------------------------------------------*/

    pub const COUNT: usize = 64;
    pub const ALL: [Square; Self::COUNT] = [
        Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
        Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
        Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
        Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
        Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
        Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
        Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
        Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
    ];
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SquareParseError {
    InvalidFile,
    InvalidRank,
}

impl FromStr for Square {
    type Err = SquareParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let file = chars.next()
            .and_then(|c| File::try_from(c).ok())
            .ok_or(SquareParseError::InvalidFile)?;
        let rank = chars.next()
            .and_then(|c| Rank::try_from(c).ok())
            .ok_or(SquareParseError::InvalidRank)?;
        
        Ok(Square::new(file, rank))
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

/*----------------------------------------------------------------*/

#[test]
fn validate_square() {
    let a1 = Square::A1;

    assert_eq!(Square::index(0), a1);
    assert_eq!(Square::try_index(0).unwrap(), a1);
    assert_eq!(a1.bitboard(), Bitboard(0x1));
    assert_eq!(a1.try_offset(-1, 0), None);
    assert_eq!(a1.try_offset(1, 0), Some(Square::B1));
    assert_eq!(a1.try_offset(0, -1), None);
    assert_eq!(a1.try_offset(0, 1), Some(Square::A2));
    
    let e4 = Square::E4;
    
    assert_eq!(Square::index(28), e4);
    assert_eq!(Square::try_index(28).unwrap(), e4);
    assert_eq!(e4.bitboard(), Bitboard(0x10000000));
    assert_eq!(e4.try_offset(1, 0), Some(Square::F4));
    assert_eq!(e4.try_offset(-1, 0), Some(Square::D4));
    assert_eq!(e4.try_offset(0, 1), Some(Square::E5));
    assert_eq!(e4.try_offset(0, -1), Some(Square::E3));
    assert_eq!(e4.try_offset(1, 1), Some(Square::F5));
    assert_eq!(e4.try_offset(1, -1), Some(Square::F3));
    
    let d2 = Square::D2;
    
    assert_eq!(Square::index(11), d2);
    assert_eq!(Square::try_index(11).unwrap(), d2);
    assert_eq!(d2.bitboard(), Bitboard(0x800));
    assert_eq!(d2.try_offset(1, 0), Some(Square::E2));
    assert_eq!(d2.try_offset(-1, 0), Some(Square::C2));
    assert_eq!(d2.try_offset(0, 1), Some(Square::D3));
    assert_eq!(d2.try_offset(0, -1), Some(Square::D1));
    assert_eq!(d2.flip_rank(), Square::D7);
    assert_eq!(d2.flip_file(), Square::E2);
    
    let h8 = Square::H8;
    
    assert_eq!(Square::index(63), h8);
    assert_eq!(Square::try_index(63).unwrap(), h8);
    assert_eq!(h8.bitboard(), Bitboard(0x8000000000000000));
    assert_eq!(h8.try_offset(1, 0), None);
    assert_eq!(h8.try_offset(-1, 0), Some(Square::G8));
    assert_eq!(h8.try_offset(0, 1), None);
    assert_eq!(h8.try_offset(0, -1), Some(Square::H7));
    assert_eq!(h8.try_offset(1, 1), None);
}