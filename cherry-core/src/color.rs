#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline(always)]
    pub const fn index(i: usize) -> Color {
        match i {
            0 => Color::White,
            1 => Color::Black,
            _ => panic!("Color::index(): Index out of bounds")
        }
    }

    #[inline(always)]
    pub const fn try_index(i: usize) -> Option<Color> {
        match i {
            0 => Some(Color::White),
            1 => Some(Color::Black),
            _ => None
        }
    }

    #[inline(always)]
    pub const fn sign(self) -> i16 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    /*----------------------------------------------------------------*/

    pub const COUNT: usize = 2;
    pub const ALL: [Color; Self::COUNT] = [Color::White, Color::Black];
}

impl std::ops::Not for Color {
    type Output = Color;

    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/*----------------------------------------------------------------*/

pub struct ColorParseError;

impl From<Color> for char {
    #[inline(always)]
    fn from(color: Color) -> char {
        match color {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

impl TryFrom<char> for Color {
    type Error = ColorParseError;

    #[inline(always)]
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'w' => Ok(Color::White),
            'b' => Ok(Color::Black),
            _ => Err(ColorParseError),
        }
    }
}