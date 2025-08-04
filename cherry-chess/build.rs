use std::{env, fs, io::{BufWriter, Write}, path::PathBuf};
use cherry_types::*;

/*----------------------------------------------------------------*/

fn write_moves(
    table: &mut [Bitboard],
    blockers: impl Fn(Square) -> Bitboard,
    moves: impl Fn(Square, Bitboard) -> Bitboard,
    index: impl Fn(Square, Bitboard) -> usize,
) {
    for &sq in &Square::ALL {
        for blockers in blockers(sq).iter_subsets() {
            table[index(sq, blockers)] = moves(sq, blockers);
        }
    }
}

/*----------------------------------------------------------------*/

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut table = [Bitboard::EMPTY; SLIDER_TABLE_SIZE];
    write_moves(
        &mut table,
        bishop_relevant_blockers,
        bishop_moves_slow,
        bishop_magic_index,
    );
    write_moves(
        &mut table,
        rook_relevant_blockers,
        rook_moves_slow,
        rook_magic_index,
    );

    let mut out_file: PathBuf = env::var_os("OUT_DIR").unwrap().into();
    out_file.push("slider_moves.rs");

    let mut out_file = BufWriter::new(fs::File::create(out_file).unwrap());

    writeln!(out_file, "const SLIDER_MOVES: &[Bitboard; {}] = &[", SLIDER_TABLE_SIZE).unwrap();
    for &bb in table.iter() {
        writeln!(out_file, "{:?},", bb).unwrap();
    }
    writeln!(out_file, "];").unwrap();
}