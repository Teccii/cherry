#![allow(incomplete_features)]
#![feature(str_split_whitespace_remainder)]
#![feature(generic_const_exprs)]
#![cfg_attr(feature="nnue", feature(portable_simd))]

mod cherry;

use std::{env, io};
use cherry::*;

/*----------------------------------------------------------------*/

fn main() {
    let mut buffer = String::new();
    let mut engine = Engine::new();
    let args = env::args()
        .skip(1)
        .collect::<Vec<String>>()
        .join(" ");

    if !args.is_empty() {
        engine.input(args.trim(), args.len());
        return;
    }

    println!("Cherry v{} by Tecci", ENGINE_VERSION);
    
    while let Ok(bytes) = io::stdin().read_line(&mut buffer) {
        if !engine.input(buffer.trim(), bytes) {
            break;
        }

        buffer.clear();
    }
}