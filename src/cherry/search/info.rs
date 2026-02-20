use std::fmt::Write;

use crate::*;

#[derive(Debug, Clone)]
pub enum SearchInfo {
    Uci {
        frc: bool,
        minimal: bool,
    },
    None,
}

impl SearchInfo {
    pub fn update(
        &mut self,
        board: &Board,
        thread: &ThreadData,
        shared: &SharedData,
        multipv: u8,
        pv_index: usize,
        pv: &PrincipalVariation,
        bound: TTFlag,
        score: Score,
        depth: u8,
        last: bool,
    ) {
        match self {
            SearchInfo::Uci { frc, minimal } => {
                let nodes = thread.nodes.global();
                let time = shared.time_man.elapsed();

                if *minimal && !last {
                    return;
                }

                println!(
                    "info depth {} seldepth {} {}score {} {}hashfull {} time {} nodes {} nps {} pv {}",
                    depth,
                    thread.sel_depth,
                    if multipv > 1 {
                        format!("multipv {} ", pv_index + 1)
                    } else {
                        String::new()
                    },
                    score,
                    match bound {
                        TTFlag::Exact => "",
                        TTFlag::UpperBound => "upperbound ",
                        TTFlag::LowerBound => "lowerbound ",
                        TTFlag::None => "",
                    },
                    shared.ttable.hash_usage(),
                    time,
                    nodes,
                    ((nodes as f64) / (time.max(1) as f64) * 1000.0) as u64,
                    pv.display(board, *frc)
                );
            }
            SearchInfo::None => {}
        }
    }

    #[inline]
    pub fn best_move(&mut self, board: &Board, best_move: Move, ponder_move: Option<Move>) {
        match self {
            SearchInfo::Uci { frc, minimal: _minimal } => {
                let mut output = String::new();

                write!(output, "bestmove {}", best_move.display(board, *frc)).unwrap();
                if let Some(mv) = ponder_move {
                    write!(output, " ponder {}", mv.display(board, *frc)).unwrap();
                }

                println!("{}", output);
            }
            SearchInfo::None => {}
        }
    }
}
