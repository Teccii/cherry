use core::{fmt, str::FromStr};
use core::ops::{Index, IndexMut};
use crate::*;

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
        if i < Square::COUNT {
            return unsafe {
                ::core::mem::transmute::<u8, Square>(i as u8)
            };
        }

        panic!("Square::index(): Index out of bounds");
    }

    #[inline]
    pub const fn try_index(i: usize) -> Option<Square> {
        if i < Square::COUNT {
            return Some(unsafe {
                ::core::mem::transmute::<u8, Square>(i as u8)
            });
        }

        None
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

impl<T> Index<Square> for [T; Square::COUNT] {
    type Output = T;

    #[inline]
    fn index(&self, sq: Square) -> &Self::Output {
        unsafe { self.get_unchecked(sq as usize) }
    }
}

impl<T> IndexMut<Square> for [T; Square::COUNT] {
    #[inline]
    fn index_mut(&mut self, sq: Square) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(sq as usize) }
    }
}

/*----------------------------------------------------------------*/

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