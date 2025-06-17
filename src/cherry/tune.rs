use std::{
    fs::OpenOptions,
    fmt::Write as _,
    io::Write,
};
use std::collections::HashMap;
use std::io::Read;
use cozy_chess::*;
use super::*;

fn sigmoid(x: f32, k: f32) -> f32 {
    1.0 / (1.0 + (-x * k).exp())
}

pub struct TrainingData {
    board: Board,
    result: f32,
}

fn error(weights: EvalWeights, data: &[TrainingData]) -> f32 {
    let mut pos = Position::with_evaluator(
        Board::default(),
        Evaluator::with_weights(weights)
    );
    let mut error = 0.0;

    for data in data {
        pos.reset(data.board.clone());

        let q_score = det_q_search(&mut pos, 0, -Score::INFINITE, Score::INFINITE);
        let diff = data.result - sigmoid(q_score.0 as f32, 0.008);

        error += diff * diff;
    }

    error / data.len() as f32
}

macro_rules! texel {
    ($improved:expr, $data:expr, $err:expr, $weights: expr, $weight:ident) => {
        let mut new_weights = $weights;
        new_weights.$weight += T::new_mg(1);
        let new_err = error(new_weights, $data);

        if new_err < $err {
            $improved = true;
            $weights = new_weights;
            $err = new_err;
        } else {
            new_weights.$weight -= T::new_mg(2);
            let new_err = error(new_weights, $data);

            if new_err < $err {
                $improved = true;
                $weights = new_weights;
                $err = new_err;
            }
        }

        new_weights.$weight += T::new_eg(1);
        let new_err = error(new_weights, $data);

        if new_err < $err {
            $improved = true;
            $weights = new_weights;
            $err = new_err;
        } else {
            new_weights.$weight -= T::new_eg(2);
            let new_err = error(new_weights, $data);

            if new_err < $err {
                $improved = true;
                $weights = new_weights;
                $err = new_err;
            }
        }
    };
    ($improved:expr, $data:expr, $err:expr, $weights: expr, $weight:ident, $i: expr) => {
        let mut new_weights = $weights;
        new_weights.$weight[$i] += T::new_mg(1);
        let new_err = error(new_weights, $data);

        if new_err < $err {
            $improved = true;
            $weights = new_weights;
            $err = new_err;
        } else {
            new_weights.$weight[$i] -= T::new_mg(2);
            let new_err = error(new_weights, $data);

            if new_err < $err {
                $improved = true;
                $weights = new_weights;
                $err = new_err;
            }
        }

        new_weights.$weight[$i] += T::new_eg(1);
        let new_err = error(new_weights, $data);

        if new_err < $err {
            $improved = true;
            $weights = new_weights;
            $err = new_err;
        } else {
            new_weights.$weight[$i] -= T::new_eg(2);
            let new_err = error(new_weights, $data);

            if new_err < $err {
                $improved = true;
                $weights = new_weights;
                $err = new_err;
            }
        }
    };
    ($improved:expr, $data:expr, $err:expr, $weights: expr, {$($weight:ident),*}) => { $(texel!($improved, $data, $err, $weights, $weight);)* };
    ($improved:expr, $data:expr, $err:expr, $weights: expr, {$($weight:ident, $size:expr;)*}) => {$(
        for i in 0..$size {
            texel!($improved, $data, $err, $weights, $weight, i);
        }
    )*}
}

pub fn tune(data_path: &str, out_path: &str) {
    let mut board_map: HashMap<Board, (f32, usize)> = HashMap::new();
    let mut data_file = std::fs::read_to_string(data_path).unwrap();
    
    for mut reader in data_file.lines().map(|s| s.split('|')) {
        let board = reader.next().and_then(|s| s.parse::<Board>().ok()).unwrap();
        let result = reader.next().and_then(|s| s.parse::<f32>().ok()).unwrap();
        
        if let Some(data) = board_map.get_mut(&board) {
            data.0 += result;
            data.1 += 1;
        } else {
            board_map.insert(board, (result, 1));
        }
    }
    
    let data: Vec<TrainingData> = board_map.iter().map(|(board, (result, count))| TrainingData {
        board: board.clone(),
        result: *result / *count as f32,
    }).collect();
    
    texel_tune(&data, out_path);
}

pub fn texel_tune(data: &[TrainingData], out_path: &str) {
    let mut out_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(out_path)
        .unwrap();

    let mut best_weights = EvalWeights::default();
    let mut best_err = error(best_weights, data);

    let mut i = 0;
    let mut improved = true;
    while improved {
        if i % 1000 == 0 {
            let mut out_str = String::new();
            out_str.push_str("\n--------------------------------\n");
            writeln!(out_str, "{}", best_weights).unwrap();
            writeln!(out_str, "Error: {}", best_err).unwrap();
            writeln!(out_str, "Iteration: {}", i).unwrap();

            print!("{}", out_str);
            write!(out_file, "{}", out_str).unwrap();
        }

        improved = false;

        texel!(improved, data, best_err, best_weights, {
            bishop_pair,
            pawn_value,
            knight_value,
            bishop_value,
            rook_value,
            queen_value,
            knight_attack,
            rook_attack,
            queen_attack,
            pawn_minor_threat,
            pawn_major_threat,
            minor_major_threat,
            backwards_pawn,
            isolated_pawn,
            doubled_pawn,
            support,
            center_control
        });

        texel!(improved, data, best_err, best_weights, {
            pawn_psqt, Square::NUM;
            knight_psqt, Square::NUM;
            bishop_psqt, Square::NUM;
            rook_psqt, Square::NUM;
            queen_psqt, Square::NUM;
            king_psqt, Square::NUM;
            knight_mobility, 9;
            bishop_mobility, 14;
            rook_mobility, 15;
            queen_mobility, 28;
            rook_open_file, File::NUM;
            rook_semiopen_file, File::NUM;
            queen_open_file, File::NUM;
            queen_semiopen_file, File::NUM;
            passed_pawn, Rank::NUM;
            phalanx, Rank::NUM;
        });

        i += 1;
    }

    let mut out_str = String::new();
    out_str.push_str("\n--------------------------------\n");
    writeln!(out_str, "{}", best_weights).unwrap();
    writeln!(out_str, "Error: {}", best_err).unwrap();
    writeln!(out_str, "Iteration: {}", i).unwrap();

    print!("{}", out_str);
    write!(out_file, "{}", out_str).unwrap();

    println!("Tuning Complete!");
}

fn det_q_search(pos: &mut Position, ply: u16, mut alpha: Score, beta: Score) -> Score {
    let static_eval = pos.eval(ply);

    if ply >= MAX_PLY || static_eval >= beta {
        return static_eval;
    }

    if static_eval > alpha {
        alpha = static_eval;
    }

    let mut moves = Vec::new();
    pos.board().generate_moves(|piece_moves| {
        for mv in piece_moves {
            if !pos.board().is_quiet_capture(mv) {
                continue;
            }

            moves.push(mv);
        }

        false
    });

    for mv in moves {
        pos.make_move(mv);
        let score = -det_q_search(pos, ply + 1, -beta, -alpha);
        pos.unmake_move();

        if score >= beta {
            return score;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}