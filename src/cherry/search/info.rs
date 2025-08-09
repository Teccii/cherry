use std::fmt::Write;
use colored::Colorize;
use crate::*;

/*----------------------------------------------------------------*/

pub trait SearchInfo {
    fn new(chess960: bool) -> Self;

    fn update(
        &mut self,
        thread: u16,
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        score: Score,
        depth: u8,
    );
}

/*----------------------------------------------------------------*/

pub struct PrettyInfo {
    chess960: bool,
}
pub struct UciInfo {
    chess960: bool,
}
pub struct NoInfo;

/*----------------------------------------------------------------*/

impl SearchInfo for PrettyInfo {
    #[inline]
    fn new(chess960: bool) -> Self {
        Self { chess960 }
    }

    fn update(
        &mut self,
        thread: u16,
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        score: Score,
        depth: u8,
    ) {
        if thread != 0 {
            return;
        }

        if depth <= 1 {
            println!("{}", board.pretty_print(self.chess960));
            println!("{}", String::from("\nDepth\tTime\t\tNodes\tNPS\t\tScore\tPV").bright_green());
        }

        let nodes = ctx.nodes.global();
        let time = shared_ctx.time_man.elapsed();
        let nps = nodes / time.max(1) * 1000;

        let mut board = board.clone();
        let mut pv_text = String::new();
        let root_pv = &ctx.root_pv;

        if root_pv.len != 0 {
            let len = usize::min(root_pv.len, depth as usize);

            for &mv in root_pv.moves[..len].iter() {
                if let Some(mv) = mv {
                    if !board.is_legal(mv) {
                        break;
                    }

                    write!(pv_text, "{} ", mv.display(&board, self.chess960)).unwrap();
                    board.make_move(mv);
                } else {
                    break;
                }
            }
        }

        println!(
            "{}/{}\t{}\t{}\t{}\t{:#}\t{}",
            depth,
            ctx.sel_depth,
            fmt_time(time).bright_black(),
            fmt_big_num(nodes).bright_black(),
            format!("{}nps", fmt_big_num(nps).to_ascii_lowercase()).bright_black(),
            score,
            pv_text.bright_black()
        );
    }
}

impl SearchInfo for UciInfo {
    #[inline]
    fn new(chess960: bool) -> Self {
        Self { chess960 }
    }

    fn update(
        &mut self,
        thread: u16,
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        score: Score,
        depth: u8,
    ) {
        if thread != 0 {
            return;
        }

        let mut board = board.clone();
        let mut pv_text = String::new();
        let root_pv = &ctx.root_pv;

        if root_pv.len != 0 {
            write!(pv_text, "pv ").unwrap();
            let len = usize::min(root_pv.len, depth as usize);

            for &mv in root_pv.moves[..len].iter() {
                if let Some(mv) = mv {
                    if !board.is_legal(mv) {
                        break;
                    }

                    write!(pv_text, "{} ", mv.display(&board, self.chess960)).unwrap();
                    board.make_move(mv);
                } else {
                    break;
                }
            }
        }

        let nodes = ctx.nodes.global();
        let time = shared_ctx.time_man.elapsed();

        println!(
            "info depth {} seldepth {} score {} time {} nodes {} nps {} {}",
            depth,
            ctx.sel_depth,
            score,
            time,
            nodes,
            nodes / time.max(1) * 1000,
            pv_text
        );
    }
}

/*----------------------------------------------------------------*/

impl SearchInfo for NoInfo {
    #[inline]
    fn new(_: bool) -> Self {
        Self
    }

    fn update(
        &mut self,
        _: u16,
        _: &Board,
        _: &ThreadContext,
        _: &SharedContext,
        _: Score,
        _: u8,
    ) { }
}