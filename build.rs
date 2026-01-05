use std::{
    env,
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
};

use cherry_core::*;

#[inline]
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

fn main() {
    write_magics();
    write_network();
}

fn write_magics() {
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

    writeln!(
        out_file,
        "const SLIDER_MOVES: &[Bitboard; {}] = &[",
        SLIDER_TABLE_SIZE
    )
    .unwrap();
    for &bb in table.iter() {
        writeln!(out_file, "{:?},", bb).unwrap();
    }
    writeln!(out_file, "];").unwrap();
}

fn write_network() {
    let network_dir = env::var("EVALFILE").unwrap_or_else(|_| "./networks/default.bin".to_string());
    let network_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("network.bin");
    let network_bytes = fs::read(&network_dir).unwrap();

    fs::write(&network_path, &network_bytes).unwrap();

    println!("cargo:rerun-if-env-changed=EVALFILE");
    println!("cargo:rerun-if-changed={}", network_dir);
}
