pub trait Direction {
    const DX: i8;
    const DY: i8;
}

pub(crate) const fn horizontal_shift_mask(shift: i8) -> u64 {
    0x101010101010101u64 * if shift > 0 {
        0xFFu8 << shift
    } else {
        0xFFu8 >> -shift
    } as u64
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
    const DX: i8 = 0;
    const DY: i8 = 1;
}

impl Direction for Down {
    const DX: i8 = 0;
    const DY: i8 = -1;
}

impl Direction for Right {
    const DX: i8 = 1;
    const DY: i8 = 0;
}

impl Direction for Left {
    const DX: i8 = -1;
    const DY: i8 = 0;
}

impl Direction for UpRight {
    const DX: i8 = 1;
    const DY: i8 = 1;
}

impl Direction for UpLeft {
    const DX: i8 = -1;
    const DY: i8 = 1;
}

impl Direction for DownRight {
    const DX: i8 = 1;
    const DY: i8 = -1;
}

impl Direction for DownLeft {
    const DX: i8 = -1;
    const DY: i8 = -1;
}