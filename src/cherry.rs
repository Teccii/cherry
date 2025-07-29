pub use cherry_chess::*;

#[cfg(not(feature = "nnue"))] mod eval {
    mod eval;
    mod weights;
    
    pub use eval::*;
    pub use weights::*;
}

/*
MIT License | Copyright (c) 2022-2023 Cosmo Bobak
Cherry's NNUE is heavily based on code from the engines Viridithas and Black Marlin.
*/
#[cfg(feature="nnue")] mod nnue {
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
    mod move_picker;
    mod search;
    mod searcher;
    mod time;
    mod ttable;
    mod weights;
    mod window;

    pub use history::*;
    pub use move_picker::*;
    pub use search::*;
    pub use searcher::*;
    pub use time::*;
    pub use ttable::*;
    pub use weights::*;
    pub use window::*;
}

mod datagen;
mod engine;
mod position;
mod score;
mod syzygy;
#[cfg(feature = "tune")] mod tune;
mod uci;
mod util;

pub use datagen::*;
#[cfg(not(feature = "nnue"))] pub use eval::*;
pub use engine::*;
#[cfg(feature = "nnue")] pub use nnue::*;
pub use position::*;
pub use score::*;
pub use search::*;
pub use syzygy::*;
#[cfg(feature = "tune")] pub use tune::*;
pub use uci::*;
pub use util::*;