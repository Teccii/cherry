use core::{
    fmt,
    ops::{Index, IndexMut},
    str::FromStr,
};

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
            return unsafe { core::mem::transmute::<u8, File>(i as u8) };
        }
        panic!("File::index(): Index out of bounds");
    }

    #[inline]
    pub const fn try_index(i: usize) -> Option<File> {
        if i < File::COUNT {
            return Some(unsafe { core::mem::transmute::<u8, File>(i as u8) });
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
        File::H,
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
            File::H => 'h',
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
