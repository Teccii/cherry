use cozy_chess::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct FeatureUpdate {
    pub piece: Piece,
    pub color: Color,
    pub sq: Square,
}

impl FeatureUpdate {
    pub fn to_index(self, perspective: Color) -> usize {
        let (sq, color) = match perspective {
            Color::White => (self.sq, self.color),
            Color::Black => (self.sq.flip_rank(), !self.color),
        };

        color as usize * Square::NUM * Piece::NUM
            + self.piece as usize * Square::NUM
            + sq as usize
    }
}