use std::{fmt, str::FromStr};
use crate::Bitboard;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl File {
    #[inline(always)]
    pub const fn index(i: usize) -> File {
        match i {
            0 => File::A,
            1 => File::B,
            2 => File::C,
            3 => File::D,
            4 => File::E,
            5 => File::F,
            6 => File::G,
            7 => File::H,
            _ => panic!("File::index(): Index out of bounds")
        }
    }

    #[inline(always)]
    pub const fn try_index(i: usize) -> Option<File> {
        match i {
            0 => Some(File::A),
            1 => Some(File::B),
            2 => Some(File::C),
            3 => Some(File::D),
            4 => Some(File::E),
            5 => Some(File::F),
            6 => Some(File::G),
            7 => Some(File::H),
            _ => None
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn offset(self, dx: i8) -> File {
        let i = self as i8 + dx;

        if i < 0 || i >= File::COUNT as i8 {
            panic!("File::offset(): New index out of bounds")
        }

        File::index(i as usize)
    }

    #[inline(always)]
    pub const fn try_offset(self, dx: i8) -> Option<File> {
        let i = self as i8 + dx;

        if i < 0 || i >= File::COUNT as i8 {
            return None;
        }

        File::try_index(i as usize)
    }
    
    #[inline(always)]
    pub const fn flip(self) -> File {
        File::index(File::H as usize - self as usize)
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn bitboard(self) -> Bitboard {
        Bitboard(0x101010101010101 << self as u8)
    }
    
    #[inline(always)]
    pub const fn adjacent(self) -> Bitboard {
        const TABLE: [Bitboard; File::COUNT] = {
            let mut table = [Bitboard::EMPTY; File::COUNT];
            let mut i = 0;
            
            while i < File::COUNT {
                let file = File::index(i);
                let mut bb = Bitboard::EMPTY;
                
                if let Some(left) = file.try_offset(-1) {
                    bb = Bitboard(bb.0 | left.bitboard().0);
                }
                
                if let Some(right) = file.try_offset(1) {
                    bb = Bitboard(bb.0 | right.bitboard().0);
                }
                
                table[i] = bb;
                i += 1;
            }
            
            table
        };
        
        TABLE[self as usize]
    }

    /*----------------------------------------------------------------*/

    pub const COUNT: usize = 8;
    pub const ALL: [File; Self::COUNT] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H
    ];
}

impl fmt::Display for File {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

/*----------------------------------------------------------------*/

pub struct FileParseError;

impl From<File> for char {
    #[inline(always)]
    fn from(f: File) -> char {
        match f {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h'
        }
    }
}

impl TryFrom<char> for File {
    type Error = FileParseError;

    #[inline(always)]
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'a' => Ok(File::A),
            'b' => Ok(File::B),
            'c' => Ok(File::C),
            'd' => Ok(File::D),
            'e' => Ok(File::E),
            'f' => Ok(File::F),
            'g' => Ok(File::G),
            'h' => Ok(File::H),
            _ => Err(FileParseError),
        }
    }
}

impl FromStr for File {
    type Err = FileParseError;

    #[inline(always)]
    fn from_str(s: &str) -> Result<File, FileParseError> {
        let mut chars = s.chars();
        let c = chars.next().ok_or(FileParseError)?;

        if chars.next().is_none() {
            c.try_into()
        } else {
            Err(FileParseError)
        }
    }
}

/*----------------------------------------------------------------*/

#[test]
fn validate_file() {
    let a = File::A;
    
    assert_eq!(File::index(0), a);
    assert_eq!(File::try_index(0).unwrap(), a);
    assert_eq!(a.bitboard(), Bitboard(0x101010101010101));
    assert_eq!(a.adjacent(), Bitboard(0x202020202020202));
    assert_eq!(a.try_offset(-1), None);
    assert_eq!(a.try_offset(1), Some(File::B));

    let b = File::B;

    assert_eq!(File::index(1), b);
    assert_eq!(File::try_index(1).unwrap(), b);
    assert_eq!(b.bitboard(), Bitboard(0x202020202020202));
    assert_eq!(b.adjacent(), Bitboard(0x505050505050505));
    assert_eq!(b.try_offset(-1), Some(File::A));
    assert_eq!(b.try_offset(1), Some(File::C));
    
    let g = File::G;

    assert_eq!(File::index(6), g);
    assert_eq!(File::try_index(6).unwrap(), g);
    assert_eq!(g.bitboard(), Bitboard(0x4040404040404040));
    assert_eq!(g.adjacent(), Bitboard(0xA0A0A0A0A0A0A0A0));
    assert_eq!(g.try_offset(-1), Some(File::F));
    assert_eq!(g.try_offset(1), Some(File::H));
    
    let h = File::H;

    assert_eq!(File::index(7), h);
    assert_eq!(File::try_index(7).unwrap(), h);
    assert_eq!(h.bitboard(), Bitboard(0x8080808080808080));
    assert_eq!(h.adjacent(), Bitboard(0x4040404040404040));
    assert_eq!(h.try_offset(-1), Some(File::G));
    assert_eq!(h.try_offset(1), None);
}