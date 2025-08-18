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

        println!("\x1B[0J{}\n", board.pretty_print(self.chess960));

        let hash_size = shared_ctx.t_table.size();
        let hash_usage = shared_ctx.t_table.hash_usage() as f64 / 1000.0;
        let used_mb = hash_size as f64 * hash_usage;

        println!(
            "{}: {:.0}/{}MB ({}%)",
            "Hash Usage".bright_black(),
            used_mb,
            hash_size,
            format!("{:.1}", hash_usage * 100.0).bright_black()
        );

        let nodes = ctx.nodes.global();
        let time = shared_ctx.time_man.elapsed();
        let nps = nodes / time.max(1) * 1000;

        println!("\n{}: {}/{}", "Depth".bright_black(), depth, ctx.sel_depth);
        println!("{}: {}", "Nodes".bright_black(), fmt_big_num(nodes));
        println!("{}:   {}nps", "NPS".bright_black(), fmt_big_num(nps));
        println!("{}:  {}", "Time".bright_black(), fmt_time(time));

        println!("\n{}:     {:#}", "Score".bright_black(), score);
        println!("{}: {}", "Best Move".bright_black(), ctx.root_pv.moves[0].unwrap().display(board, self.chess960));
        println!("{}: {}", "Main Line".bright_black(), ctx.root_pv.display(board, depth, self.chess960));
        println!("\x1B[31F");
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

        let nodes = ctx.nodes.global();
        let time = shared_ctx.time_man.elapsed();

        println!(
            "info depth {} seldepth {} score {} hashfull {} time {} nodes {} nps {} pv {}",
            depth,
            ctx.sel_depth,
            score,
            shared_ctx.t_table.hash_usage(),
            time,
            nodes,
            nodes / time.max(1) * 1000,
            ctx.root_pv.display(board, depth, self.chess960)
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