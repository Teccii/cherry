#![feature(str_split_whitespace_remainder)]
#![feature(generic_const_exprs)]

mod cherry;
mod engine;

use std::{
    sync::{Arc, Mutex},
    sync::mpsc::*,
    fmt::Write as _,
    io::Write,
    io
};
use cherry_core::*;
use cherry::*;
use engine::*;

/*----------------------------------------------------------------*/

fn main() -> Result<()> {
    let mut buffer = String::new();
    let engine = Engine::new();
    
    while let Ok(bytes) = io::stdin().read_line(&mut buffer) {
        let cmd = if bytes == 0 { UciCommand::Quit } else {
            match engine.parse(buffer.trim()) {
                Ok(cmd) => cmd,
                Err(e) => {
                    buffer.clear();
                    println!("{:?}", e);
                    continue;
                }
            }
        };
        
        engine.input(cmd)?;
        buffer.clear();
    }

    Ok(())
}