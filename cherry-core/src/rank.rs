use core::{
    fmt,
    ops::{Index, IndexMut},
    str::FromStr,
};

use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

impl Rank {
    #[inline]
    pub const fn index(i: usize) -> Rank {
        if i < Rank::COUNT {
            return unsafe { core::mem::transmute::<u8, Rank>(i as u8) };
        }

        panic!("Rank::index(): Index out of bounds");
    }

    #[inline]
    pub const fn try_index(i: usize) -> Option<Rank> {
        if i < Rank::COUNT {
            return Some(unsafe { core::mem::transmute::<u8, Rank>(i as u8) });
        }

        None
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn offset(self, dy: i8) -> Rank {
        let i = self as i8 + dy;

        if i < 0 || i >= Rank::COUNT as i8 {
            panic!("Rank::offset(): New index out of bounds")
        }

        Rank::index(i as usize)
    }

    #[inline]
    pub const fn try_offset(self, dy: i8) -> Option<Rank> {
        let i = self as i8 + dy;

        if i < 0 || i >= Rank::COUNT as i8 {
            return None;
        }

        Rank::try_index(i as usize)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn flip(self) -> Rank {
        Rank::index(Rank::Eighth as usize - self as usize)
    }

    #[inline]
    pub const fn relative_to(self, color: Color) -> Rank {
        match color {
            Color::White => self,
            Color::Black => self.flip(),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn bitboard(self) -> Bitboard {
        Bitboard(0xFF << (8 * self as u8))
    }

    /*----------------------------------------------------------------*/

    pub const COUNT: usize = 8;
    pub const ALL: [Rank; Self::COUNT] = [
        Rank::First,
        Rank::Second,
        Rank::Third,
        Rank::Fourth,
        Rank::Fifth,
        Rank::Sixth,
        Rank::Seventh,
        Rank::Eighth,
    ];
}

impl<T> Index<Rank> for [T; Rank::COUNT] {
    type Output = T;

    #[inline]
    fn index(&self, rank: Rank) -> &Self::Output {
        unsafe { self.get_unchecked(rank as usize) }
    }
}

impl<T> IndexMut<Rank> for [T; Rank::COUNT] {
    #[inline]
    fn index_mut(&mut self, rank: Rank) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(rank as usize) }
    }
}

impl fmt::Display for Rank {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RankParseError;

impl From<Rank> for char {
    #[inline]
    fn from(f: Rank) -> char {
        match f {
            Rank::First => '1',
            Rank::Second => '2',
            Rank::Third => '3',
            Rank::Fourth => '4',
            Rank::Fifth => '5',
            Rank::Sixth => '6',
            Rank::Seventh => '7',
            Rank::Eighth => '8',
        }
    }
}

impl TryFrom<char> for Rank {
    type Error = RankParseError;

    #[inline]
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '1' => Ok(Rank::First),
            '2' => Ok(Rank::Second),
            '3' => Ok(Rank::Third),
            '4' => Ok(Rank::Fourth),
            '5' => Ok(Rank::Fifth),
            '6' => Ok(Rank::Sixth),
            '7' => Ok(Rank::Seventh),
            '8' => Ok(Rank::Eighth),
            _ => Err(RankParseError),
        }
    }
}

impl FromStr for Rank {
    type Err = RankParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Rank, RankParseError> {
        let mut chars = s.chars();
        let c = chars.next().ok_or(RankParseError)?;

        if chars.next().is_none() {
            c.try_into()
        } else {
            Err(RankParseError)
        }
    }
}
