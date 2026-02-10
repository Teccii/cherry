#![feature(str_split_whitespace_remainder)]
#![feature(stmt_expr_attributes)]
#![feature(generic_const_exprs)]
#![feature(sync_unsafe_cell)]
#![allow(incomplete_features)]
#![allow(dead_code)]

mod cherry;

use std::{env, io, sync::LazyLock};

use cherry::*;
use colored::Colorize;

/*----------------------------------------------------------------*/

fn main() {
    LazyLock::force(&EPOCH);

    let mut buffer = String::new();
    let mut engine = Engine::new();
    let args = env::args().skip(1).collect::<Vec<String>>();

    if !args.is_empty() {
        for cmd in args {
            if engine.handle(cmd.trim()) == Abort::Yes {
                return;
            }
        }

        return;
    }

    println!("Cherry v{} by Tecci", ENGINE_VERSION.bright_green());
    while let Ok(_) = io::stdin().read_line(&mut buffer) {
        if buffer.trim().is_empty() {
            continue;
        }

        if engine.handle(buffer.trim()) == Abort::Yes {
            break;
        }

        buffer.clear();
    }
}
