pub use cherry_types::*;

mod chess {
    /*
    Copyright (c) 2021 87flowers

    Most of cherry-chess directly copies, or at the very least
    uses a substantial portion of the source code of Rose written
    by 87flowers.

    The original source code can be found at https://github.com/87flowers/Rose/.
    */

    mod attacks;
    mod board;
    mod byteboard;
    mod chess_move;
    pub mod geometry;
    mod piece;
    mod zobrist;

    pub use attacks::*;
    pub use board::*;
    pub use byteboard::*;
    pub use chess_move::*;
    pub use piece::*;
    pub use zobrist::*;
}

mod nnue {
    /*
    MIT License | Copyright (c) 2022-2023 Cosmo Bobak
    Cherry's NNUE is heavily based on code from the engines Viridithas and Black Marlin.
    https://github.com/cosmobobak/viridithas
    https://github.com/jnlt3/blackmarlin
    */

    mod accumulator;
    mod features;
    mod network;
    mod util;

    pub use accumulator::*;
    pub use features::*;
    pub use network::*;
    pub use util::*;
}

mod search {
    mod history;
    mod info;
    mod move_picker;
    mod search;
    mod searcher;
    mod time;
    mod ttable;
    mod weights;
    mod window;

    pub use history::*;
    pub use info::*;
    pub use move_picker::*;
    pub use search::*;
    pub use searcher::*;
    pub use time::*;
    pub use ttable::*;
    pub use weights::*;
    pub use window::*;
}

#[cfg(feature = "datagen")]
mod datagen;
mod engine;
mod position;
mod score;
mod simd;
mod syzygy;
mod uci;
mod util;

pub use chess::*;
#[cfg(feature = "datagen")]
pub use datagen::*;
pub use engine::*;
pub use nnue::*;
pub use position::*;
pub use score::*;
pub use search::*;
pub use simd::*;
pub use syzygy::*;
pub use uci::*;
pub use util::*;
