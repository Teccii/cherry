use core::{ops::*, str::FromStr};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline]
    pub const fn index(i: usize) -> Color {
        match i {
            0 => Color::White,
            1 => Color::Black,
            _ => panic!("Color::index(): Index out of bounds")
        }
    }

    #[inline]
    pub const fn try_index(i: usize) -> Option<Color> {
        match i {
            0 => Some(Color::White),
            1 => Some(Color::Black),
            _ => None
        }
    }

    #[inline]
    pub const fn sign(self) -> i16 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    #[inline]
    pub const fn msb(self) -> u8 {
        match self {
            Color::White => 0,
            Color::Black => 0x80,
        }
    }

    /*----------------------------------------------------------------*/

    pub const COUNT: usize = 2;
    pub const ALL: [Color; Self::COUNT] = [Color::White, Color::Black];
}

impl Not for Color {
    type Output = Color;

    #[inline]
    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl<T> Index<Color> for [T; Color::COUNT] {
    type Output = T;

    #[inline]
    fn index(&self, color: Color) -> &Self::Output {
        unsafe { self.get_unchecked(color as usize) }
    }
}

impl<T> IndexMut<Color> for [T; Color::COUNT] {
    #[inline]
    fn index_mut(&mut self, color: Color) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(color as usize) }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ColorParseError;

impl From<Color> for char {
    #[inline]
    fn from(color: Color) -> char {
        match color {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

impl TryFrom<char> for Color {
    type Error = ColorParseError;

    #[inline]
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'w' => Ok(Color::White),
            'b' => Ok(Color::Black),
            _ => Err(ColorParseError),
        }
    }
}

impl FromStr for Color {
    type Err = ColorParseError;
    
    #[inline]
    fn from_str(s: &str) -> Result<Color, ColorParseError> {
        let mut chars = s.chars();
        let c = chars.next().ok_or(ColorParseError)?;
        
        if chars.next().is_none() {
            c.try_into()
        } else {
            Err(ColorParseError)
        }
    }
}