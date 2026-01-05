#![feature(str_split_whitespace_remainder)]
#![feature(stmt_expr_attributes)]
#![feature(generic_const_exprs)]
#![feature(sync_unsafe_cell)]
#![allow(incomplete_features)]
#![allow(dead_code)]

mod cherry;

use std::{env, io};

use cherry::*;
use colored::Colorize;

/*----------------------------------------------------------------*/

fn main() {
    init_lmr();

    let mut buffer = String::new();
    let mut engine = Engine::new();
    let args = env::args().skip(1).collect::<Vec<String>>();

    if !args.is_empty() {
        for cmd in args {
            let cmd = cmd.trim();
            engine.input(cmd, cmd.len());
        }

        return;
    }

    println!("Cherry v{} by Tecci", ENGINE_VERSION.bright_green());
    while let Ok(bytes) = io::stdin().read_line(&mut buffer) {
        if !engine.input(buffer.trim(), bytes) {
            break;
        }

        buffer.clear();
    }
}
