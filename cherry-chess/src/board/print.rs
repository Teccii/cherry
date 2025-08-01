use std::fmt::Write;
use crate::*;

impl Board {
    pub fn pretty_print(&self, chess960: bool) -> String {
        let mut result = String::new();
        let mut features = Vec::new();

        features.push(format!("FEN: {}", self));

        if let Some(sq) = self.ep_square() {
            features.push(format!("En Passant Square: {}", sq));
        }

        if self.castle_rights[0] != CastleRights::EMPTY || self.castle_rights[1] != CastleRights::EMPTY {
            let mut rights = String::new();
            for &color in &Color::ALL {
                let mut write_rights = |file: Option<File>, right_char: char| {
                    if let Some(file) = file {
                        let mut right = if chess960 {
                            file.into()
                        } else {
                            right_char
                        };

                        if color == Color::White {
                            right = right.to_ascii_uppercase();
                        }

                        write!(&mut rights, "{}", right).unwrap();
                    }
                };

                write_rights(self.castle_rights[color as usize].short, 'k');
                write_rights(self.castle_rights[color as usize].long, 'q');
            }

            features.push(format!("Castle Rights: {}", rights));
        }

        features.push(format!("Full Move Count: {}", self.fullmove_count));
        features.push(format!("Halfmove Clock: {}", self.halfmove_clock));
        features.push(format!("Side To Move: {:?}", self.stm));

        writeln!(&mut result, "+-----------------+").unwrap();

        for &rank in Rank::ALL.iter().rev() {
            write!(&mut result, "|").unwrap();

            for &file in File::ALL.iter() {
                let sq = Square::new(file, rank);

                if !self.occupied().has(sq) {
                    write!(&mut result, " .").unwrap();
                } else {
                    let piece: char = self.piece_on(sq).unwrap().into();

                    if self.colors(Color::White).has(sq) {
                        write!(&mut result, " {}", piece.to_ascii_uppercase()).unwrap();
                    } else {
                        write!(&mut result, " {}" , piece).unwrap();
                    }
                }
            }

            if let Some(feature) = features.get(7 - rank as usize) {
                writeln!(&mut result, " | {}", feature).unwrap();
            } else {
                writeln!(&mut result, " |").unwrap();
            }
        }

        writeln!(&mut result, "+-----------------+").unwrap();
        result
    }
}