use std::{
    fmt,
    num::NonZeroU8,
    str::FromStr
};
use crate::Color;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

impl Piece {
    #[inline]
    pub const fn index(i: usize) -> Piece {
        match i {
            0 => Piece::Pawn,
            1 => Piece::Knight,
            2 => Piece::Bishop,
            3 => Piece::Rook,
            4 => Piece::Queen,
            5 => Piece::King,
            _ => panic!("Piece::index(): Index out of bounds")
        }
    }

    #[inline]
    pub const fn try_index(i: usize) -> Option<Piece> {
        match i {
            0 => Some(Piece::Pawn),
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            5 => Some(Piece::King),
            _ => None
        }
    }

    /*----------------------------------------------------------------*/

    pub fn see_value(self) -> i16 {
        match self {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 580,
            Piece::Queen => 920,
            Piece::King => 20000,
        }
    }

    /*----------------------------------------------------------------*/

    pub const COUNT: usize = 6;
    pub const ALL: [Piece; Self::COUNT] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];
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
            Piece::King => 'k'
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

/*----------------------------------------------------------------*/

/*
Bit Layout:
1-3: Piece (Pawn = 0, Knight = 1, King = 5)
4: Color (White = 0, Black = 1)
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ColorPiece {
    bits: NonZeroU8
}

impl ColorPiece {
    #[inline]
    pub const fn new(piece: Piece, color: Color) -> ColorPiece {
        let mut bits = 0b10000;
        bits |= piece as u8;
        bits |= (color as u8) << 3;
        
        ColorPiece { bits: NonZeroU8::new(bits).unwrap() }
    }

    /*----------------------------------------------------------------*/
    
    #[inline]
    pub const fn piece(self) -> Piece {
        Piece::index((self.bits.get() & 0b111) as usize)
    }
    
    #[inline]
    pub const fn color(self) -> Color {
        Color::index(((self.bits.get() >> 3) & 0b1) as usize)
    }
}