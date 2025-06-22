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
    #[inline(always)]
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

    #[inline(always)]
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

pub struct PieceParseError;

impl TryFrom<char> for Piece {
    type Error = PieceParseError;

    #[inline(always)]
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
    #[inline(always)]
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