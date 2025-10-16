use core::{fmt, str::FromStr};
use core::ops::{Index, IndexMut};
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
    #[inline]
    pub const fn index(i: usize) -> File {
        if i < File::COUNT {
            return unsafe {
                ::core::mem::transmute::<u8, File>(i as u8)
            };
        }
        panic!("File::index(): Index out of bounds");
    }

    #[inline]
    pub const fn try_index(i: usize) -> Option<File> {
        if i < File::COUNT {
            return Some(unsafe {
                ::core::mem::transmute::<u8, File>(i as u8)
            });
        }
        
        None
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn offset(self, dx: i8) -> File {
        let i = self as i8 + dx;

        if i < 0 || i >= File::COUNT as i8 {
            panic!("File::offset(): New index out of bounds")
        }

        File::index(i as usize)
    }

    #[inline]
    pub const fn try_offset(self, dx: i8) -> Option<File> {
        let i = self as i8 + dx;

        if i < 0 || i >= File::COUNT as i8 {
            return None;
        }

        File::try_index(i as usize)
    }
    
    #[inline]
    pub const fn flip(self) -> File {
        File::index(File::H as usize - self as usize)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn bitboard(self) -> Bitboard {
        Bitboard(0x101010101010101 << self as u8)
    }
    
    #[inline]
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

impl<T> Index<File> for [T; File::COUNT] {
    type Output = T;

    #[inline]
    fn index(&self, file: File) -> &Self::Output {
        unsafe { self.get_unchecked(file as usize) }
    }
}

impl<T> IndexMut<File> for [T; File::COUNT] {
    #[inline]
    fn index_mut(&mut self, file: File) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(file as usize) }
    }
}

/*----------------------------------------------------------------*/

impl fmt::Display for File {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FileParseError;

impl From<File> for char {
    #[inline]
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

    #[inline]
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

    #[inline]
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