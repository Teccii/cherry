use std::fmt::Write;

use crate::*;

#[derive(Debug, Clone)]
pub enum SearchInfo {
    Uci {
        minimal: bool,
        normalisation: bool,
        wdl: bool,
        frc: bool,
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
        mut score: Score,
        depth: u8,
        last: bool,
    ) {
        match self {
            SearchInfo::Uci {
                minimal,
                normalisation,
                wdl,
                frc,
            } => {
                let nodes = thread.nodes.global();
                let time = shared.time_man.elapsed();

                if *minimal && !last {
                    return;
                }

                let material = if *normalisation | *wdl {
                    board.classic_material()
                } else {
                    0
                };

                let mut output = String::from("info ");
                output.push_str(format!("depth {} seldepth {} ", depth, thread.sel_depth).as_str());

                if multipv > 1 {
                    output.push_str(format!("multipv {} ", pv_index + 1).as_str());
                }

                if score.abs() <= 2 {
                    score = Score::ZERO;
                }

                let out_score = if *normalisation {
                    score.normalise(material)
                } else {
                    score
                };
                output.push_str(format!("score {} ", out_score).as_str());
                match bound {
                    TTFlag::Exact => {}
                    TTFlag::UpperBound => output.push_str("upperbound "),
                    TTFlag::LowerBound => output.push_str("lowerbound "),
                    TTFlag::None => {}
                }

                if *wdl {
                    let (w, l) = wdl_model(score, material);
                    let d = 1000 - w - l;

                    output.push_str(format!("wdl {} {} {} ", w, d, l).as_str());
                }

                output.push_str(
                    format!(
                        "hashfull {} time {} nodes {} nps {} pv {}",
                        shared.ttable.hash_usage(),
                        time,
                        nodes,
                        ((nodes as f64) / (time.max(1) as f64) * 1000.0) as u64,
                        pv.display(board, *frc)
                    )
                    .as_str(),
                );

                println!("{output}");
            }
            SearchInfo::None => {}
        }
    }

    #[inline]
    pub fn best_move(&mut self, board: &Board, best_move: Move, ponder_move: Option<Move>) {
        match self {
            SearchInfo::Uci {
                minimal: _minimal,
                normalisation: _normalisation,
                wdl: _wdl,
                frc,
            } => {
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
