use core::{fmt, ops::*, str::FromStr};

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    #[inline]
    pub const fn index(i: usize) -> Piece {
        if i < Piece::COUNT {
            return unsafe { core::mem::transmute::<u8, Piece>(i as u8) };
        }

        panic!("Piece::index(): Index out of bounds");
    }

    #[inline]
    pub const fn try_index(i: usize) -> Option<Piece> {
        if i < Piece::COUNT {
            return Some(unsafe { core::mem::transmute::<u8, Piece>(i as u8) });
        }

        None
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn from_bits(bits: u8) -> Option<Piece> {
        match bits {
            0b010 => Some(Piece::Pawn),
            0b011 => Some(Piece::Knight),
            0b101 => Some(Piece::Bishop),
            0b110 => Some(Piece::Rook),
            0b111 => Some(Piece::Queen),
            0b001 => Some(Piece::King),
            _ => None,
        }
    }

    #[inline]
    pub const fn bits(self) -> u8 {
        match self {
            Piece::Pawn => 0b010,
            Piece::Knight => 0b011,
            Piece::Bishop => 0b101,
            Piece::Rook => 0b110,
            Piece::Queen => 0b111,
            Piece::King => 0b001,
        }
    }

    #[inline]
    pub const fn is_slider(self) -> bool {
        (self.bits() & 0b100) != 0
    }

    /*----------------------------------------------------------------*/

    pub const COUNT: usize = 6;
    pub const ALL: [Piece; Self::COUNT] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King];
}

impl<T> Index<Piece> for [T; Piece::COUNT] {
    type Output = T;

    #[inline]
    fn index(&self, piece: Piece) -> &Self::Output {
        unsafe { self.get_unchecked(piece as usize) }
    }
}

impl<T> IndexMut<Piece> for [T; Piece::COUNT] {
    #[inline]
    fn index_mut(&mut self, piece: Piece) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(piece as usize) }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PieceParseError;

impl TryFrom<char> for Piece {
    type Error = PieceParseError;

    #[inline]
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'p' => Ok(Piece::Pawn),
            'n' => Ok(Piece::Knight),
            'b' => Ok(Piece::Bishop),
            'r' => Ok(Piece::Rook),
            'q' => Ok(Piece::Queen),
            'k' => Ok(Piece::King),
            _ => Err(PieceParseError),
        }
    }
}

impl From<Piece> for char {
    #[inline]
    fn from(p: Piece) -> Self {
        match p {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        }
    }
}

impl FromStr for Piece {
    type Err = PieceParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Piece, PieceParseError> {
        let mut chars = s.chars();
        let c = chars.next().ok_or(PieceParseError)?;

        if chars.next().is_none() {
            c.try_into()
        } else {
            Err(PieceParseError)
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}
