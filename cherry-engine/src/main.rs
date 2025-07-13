#![feature(str_split_whitespace_remainder)]
#![feature(generic_const_exprs)]

mod cherry;
mod engine;

use std::io;
use cherry_core::*;
use cherry::*;
use engine::*;

/*----------------------------------------------------------------*/

fn main() -> Result<()> {
    let mut buffer = String::new();
    let mut engine = Engine::new();
    
    while let Ok(bytes) = io::stdin().read_line(&mut buffer) {
        if !engine.input(buffer.trim(), bytes) {
            break;
        }

        buffer.clear();
    }

    Ok(())
}