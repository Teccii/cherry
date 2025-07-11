use std::{env, fs, io::Write, path::PathBuf};
#[cfg(feature = "pext")] use std::arch::x86_64::_pext_u64;
use std::io::BufWriter;
use gungnir_types::*;

/*----------------------------------------------------------------*/

fn write_moves(
    #[cfg(not(feature = "pext"))] table: &mut [Bitboard],
    #[cfg(feature = "pext")] table: &mut [u16],
    blockers: impl Fn(Square) -> Bitboard,
    moves: impl Fn(Square, Bitboard) -> Bitboard,
    #[cfg(feature = "pext")] rays: impl Fn(Square) -> Bitboard,
    index: impl Fn(Square, Bitboard) -> usize,
) {
    for &sq in &Square::ALL {
        for blockers in blockers(sq).iter_subsets() {
            #[cfg(not(feature = "pext"))] {
                table[index(sq, blockers)] = moves(sq, blockers);
            }
            
            #[cfg(feature = "pext")] {
                table[index(sq, blockers)] = unsafe { _pext_u64(moves(sq, blockers).0, rays(sq).0) as u16 } ;
            }
        }
    }
}

/*----------------------------------------------------------------*/

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(not(feature = "pext"))] let mut table = [Bitboard::EMPTY; SLIDER_TABLE_SIZE];
    #[cfg(feature = "pext")] let mut table = [0; SLIDER_TABLE_SIZE];
    
    write_moves(
        &mut table,
        bishop_relevant_blockers,
        bishop_moves_slow,
        #[cfg(feature = "pext")] bishop_rays,
        bishop_magic_index,
    );
    write_moves(
        &mut table,
        rook_relevant_blockers,
        rook_moves_slow,
        #[cfg(feature = "pext")] rook_rays,
        rook_magic_index,
    );

    let mut out_file: PathBuf = env::var_os("OUT_DIR").unwrap().into();
    out_file.push("slider_moves.rs");

    let mut out_file = BufWriter::new(fs::File::create(out_file).unwrap());

    #[cfg(not(feature = "pext"))] writeln!(out_file, "const SLIDER_MOVES: &[Bitboard; {}] = &[", SLIDER_TABLE_SIZE).unwrap();
    #[cfg(feature = "pext")] writeln!(out_file, "const SLIDER_MOVES: &[u16; {}] = &[", SLIDER_TABLE_SIZE).unwrap();

    for (i, &bb) in table.iter().enumerate() {
        if i % 4 < 3 {
            write!(out_file, "{:?},\t", bb).unwrap();
        } else {
            write!(out_file, "{:?},\n", bb).unwrap();
        }
    }
    writeln!(out_file, "];").unwrap();
}