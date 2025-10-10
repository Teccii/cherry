use core::fmt::Write;
use colored::Colorize;
use crate::*;

impl Board {
    pub fn pretty_print(&self, chess960: bool) -> String {
        let mut result = String::new();
        let mut features = Vec::new();

        features.push(format!("{}: {}", String::from("FEN").bright_green(), self.to_fen(chess960)));
        features.push(format!("{}: {:#016X}", String::from("Zobrist Key").bright_green(), self.hash));

        if let Some(sq) = self.ep_square() {
            features.push(format!("{}: {}", String::from("En Passant").bright_green(), sq));
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

            features.push(format!("{}: {}", String::from("Castle Rights").bright_green(), rights));
        }

        features.push(format!("{}: {}", String::from("Halfmove Clock").bright_green(), self.halfmove_clock));
        features.push(format!("{}: {}", String::from("Fullmove Count").bright_green(), self.fullmove_count));
        features.push(format!("{}: {:?}", String::from("Side To Move").bright_green(), self.stm));

        writeln!(&mut result, "╔═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╗").unwrap();

        for &rank in Rank::ALL.iter().rev() {
            write!(&mut result, "║").unwrap();

            for &file in File::ALL.iter() {
                let sq = Square::new(file, rank);

                if let Some(piece) = self.piece_on(sq) {
                    let piece: char = piece.into();
                    if self.color_on(sq).unwrap() == Color::White {
                        write!(&mut result, " {}", String::from(piece.to_ascii_uppercase()).bright_green()).unwrap();
                    } else {
                        write!(&mut result, " {}" , String::from(piece).bright_blue()).unwrap();
                    }
                } else {
                    write!(&mut result, " .").unwrap();
                }

                write!(&mut result, " {}", if file == File::H { '║' } else { '│' }).unwrap();
            }

            if let Some(feature) = features.get(7 - rank as usize) {
                writeln!(&mut result, " {}\t{}", rank, feature).unwrap();
            } else {
                writeln!(&mut result, " {}", rank).unwrap();
            }

            writeln!(&mut result, "{}", if rank == Rank::First {
                "╚═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╝"
            } else {
                "╟───┼───┼───┼───┼───┼───┼───┼───╢"
            }).unwrap();
        }

        for &file in &File::ALL {
            write!(&mut result, "  {:?} ", file).unwrap();
        }

        result
    }
}