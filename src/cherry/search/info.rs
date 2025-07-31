use std::fmt::Write;
use crate::*;

/*----------------------------------------------------------------*/

pub trait SearchInfo {
    fn update(
        thread: u16,
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        score: Score,
        depth: u8,
        chess960: bool,
    );
}

/*----------------------------------------------------------------*/

pub struct UciInfo;
pub struct NoInfo;

/*----------------------------------------------------------------*/

impl SearchInfo for UciInfo {
    fn update(
        thread: u16,
        board: &Board,
        ctx: &ThreadContext,
        shared_ctx: &SharedContext,
        score: Score,
        depth: u8,
        chess960: bool,
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

                    write!(pv_text, "{} ", mv.display(&board, chess960)).unwrap();
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
    fn update(
        _: u16,
        _: &Board,
        _: &ThreadContext,
        _: &SharedContext,
        _: Score,
        _: u8,
        _: bool,
    ) { }
}