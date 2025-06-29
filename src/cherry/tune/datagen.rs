use std::{
    fs::OpenOptions,
    io::Write,
    sync::{mpsc, Arc},
    time::*
};
use std::io::BufWriter;
use arrayvec::ArrayVec;
use cozy_chess::*;
use rand::Rng;
use crate::cherry::*;

fn gen_single(
    searcher: &mut Searcher,
    time_man: &TimeManager,
    limits: &[SearchLimit]
) -> Vec<(Board, f32)> {
    let mut scores: Vec<Board> = Vec::new();
    let mut result = 0.5f32;
    let mut rng = rand::rng();

    for ply in 0..MAX_PLY {
        if searcher.pos.is_checkmate() {
            result = if searcher.pos.board().side_to_move() == Color::White {
                1.0f32
            } else {
                0.0f32
            };

            break;
        }

        if searcher.pos.is_draw() {
            result = 0.5f32;
            break;
        }

        if ply < 12 {
            let mut moves: ArrayVec<Move, MAX_MOVES> = ArrayVec::new();
            searcher.pos.board().generate_moves(|piece_moves| {
                for mv in piece_moves {
                    moves.push(mv);
                }
                false
            });

            let mv = moves[rng.random_range(0..moves.len())];
            searcher.pos.make_move(mv);
        } else {
            time_man.init(&mut searcher.pos, limits);
    
            let (mv, _, score, _, _) = searcher.search(limits);
            let turn = match searcher.pos.board().side_to_move() {
                Color::White => 1,
                Color::Black => -1,
            };

            if !score.is_mate() && !score.is_infinite() {
                scores.push(searcher.pos.board().clone());
            }

            searcher.pos.make_move(mv);
        }
    }

    scores.iter()
        .map(|b| (b.clone(), result))
        .collect()
}

fn gen_many(count: usize, move_time: u64) -> Vec<(Board, f32)> {
    let time_man = Arc::new(TimeManager::new());
    let mut searcher = Searcher::new(Board::default(), time_man.clone());
    let mut boards = Vec::new();
    
    searcher.resize_ttable(2048);
    
    for _ in 0..count {
        boards.append(&mut gen_single(
            &mut searcher,
            &time_man,
            &[SearchLimit::MoveTime(Duration::from_millis(move_time))],
        ));

        searcher.clean_ttable();
        searcher.pos.reset(Board::default());
    }

    boards
}

pub fn datagen(out_path: &str, threads: u16, move_time: u64) {
    let start = Instant::now();
    let mut total = 0;
    
    loop {
        let iter_start = Instant::now();
        
        let (tx, rx) = mpsc::channel();
        let mut join_handlers = Vec::new();

        for _ in 0..threads {
            let tx = tx.clone();

            join_handlers.push(std::thread::spawn(move || {
                tx.send(gen_many(100, move_time)).unwrap()
            }));
        }

        for join_handler in join_handlers {
            join_handler.join().unwrap();
        }

        drop(tx);
        
        let write_start = Instant::now();
        let mut output = String::new();
        let mut count: u64 = 0;
        for (board, wdl) in rx.iter().flatten() {
            output += &format!("{} | {:.1}\n", board, wdl);
            count += 1;
        }
        
        total += count;

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(out_path)
            .unwrap();

        write!(&mut file, "{}", output).unwrap();

        println!("Wrote {} positions to {} (total: {} total_time: {:?} iter_time {:?} write_time {:?})",
                 count, out_path, total, start.elapsed(), iter_start.elapsed(), write_start.elapsed());
    }
}