use cherry_chess::*;

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

        color as usize * Square::COUNT * Piece::COUNT
            + self.piece as usize * Square::COUNT
            + sq as usize
    }
}