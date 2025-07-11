use crate::{Bitboard, File, Rank};

/*----------------------------------------------------------------*/

pub trait Direction {
    const MASK: Bitboard;
    const SHIFT: isize;
    const DX: i8;
    const DY: i8;
}

/*----------------------------------------------------------------*/

pub struct Up;
pub struct Down;
pub struct Right;
pub struct Left;

pub struct UpRight;
pub struct UpLeft;
pub struct DownRight;
pub struct DownLeft;

/*----------------------------------------------------------------*/

impl Direction for Up {
    const MASK: Bitboard = Bitboard::FULL;
    const SHIFT: isize = 8;
    const DX: i8 = 0;
    const DY: i8 = 1;
}

impl Direction for Down {
    const MASK: Bitboard = Bitboard::FULL;
    const SHIFT: isize = -8;
    const DX: i8 = 0;
    const DY: i8 = -1;
}

impl Direction for Right {
    const MASK: Bitboard = Bitboard(!File::H.bitboard().0);
    const SHIFT: isize = 1;
    const DX: i8 = 1;
    const DY: i8 = 0;
}

impl Direction for Left {
    const MASK: Bitboard = Bitboard(!File::A.bitboard().0);
    const SHIFT: isize = -1;
    const DX: i8 = -1;
    const DY: i8 = 0;
}

impl Direction for UpRight {
    const MASK: Bitboard = Bitboard(!File::H.bitboard().0 & !Rank::Eighth.bitboard().0);
    const SHIFT: isize = 9;
    const DX: i8 = 1;
    const DY: i8 = 1;
}

impl Direction for UpLeft {
    const MASK: Bitboard = Bitboard(!File::A.bitboard().0 & !Rank::Eighth.bitboard().0);
    const SHIFT: isize = 7;
    const DX: i8 = -1;
    const DY: i8 = 1;
}

impl Direction for DownRight {
    const MASK: Bitboard = Bitboard(!File::H.bitboard().0 & !Rank::First.bitboard().0);
    const SHIFT: isize = -7;
    const DX: i8 = 1;
    const DY: i8 = -1;
}

impl Direction for DownLeft {
    const MASK: Bitboard = Bitboard(!File::A.bitboard().0 & !Rank::First.bitboard().0);
    const SHIFT: isize = -9;
    const DX: i8 = -1;
    const DY: i8 = -1;
}