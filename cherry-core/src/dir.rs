pub trait Direction {
    const DX: i8;
    const DY: i8;
}

pub(crate) const fn horizontal_shift_mask(shift: i8) -> u64 {
    0x101010101010101u64
        * if shift > 0 {
            0xFFu8 << shift
        } else {
            0xFFu8 >> -shift
        } as u64
}

/*----------------------------------------------------------------*/

pub struct North;
pub struct South;
pub struct East;
pub struct West;

pub struct NorthEast;
pub struct NorthWest;
pub struct SouthEast;
pub struct SouthWest;

/*----------------------------------------------------------------*/

impl Direction for North {
    const DX: i8 = 0;
    const DY: i8 = 1;
}

impl Direction for South {
    const DX: i8 = 0;
    const DY: i8 = -1;
}

impl Direction for East {
    const DX: i8 = 1;
    const DY: i8 = 0;
}

impl Direction for West {
    const DX: i8 = -1;
    const DY: i8 = 0;
}

impl Direction for NorthEast {
    const DX: i8 = 1;
    const DY: i8 = 1;
}

impl Direction for NorthWest {
    const DX: i8 = -1;
    const DY: i8 = 1;
}

impl Direction for SouthEast {
    const DX: i8 = 1;
    const DY: i8 = -1;
}

impl Direction for SouthWest {
    const DX: i8 = -1;
    const DY: i8 = -1;
}
