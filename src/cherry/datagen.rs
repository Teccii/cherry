use std::{
    sync::*,
    time::*,
    io::Write,
    fs::OpenOptions,
};
use rand::Rng;
use arrayvec::ArrayVec;
use cozy_chess::*;
use super::*;

fn gen_single(
    searcher: &mut Searcher,
    time_man: &TimeManager,
    limits: &[SearchLimit]
) -> Vec<(Board, Score, f32)> {
    let mut scores: Vec<(Board, Score)> = Vec::new();
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

        if searcher.pos.is_draw(ply) {
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
            time_man.init(&searcher.pos, limits);

            let (mv, _, score, _, _) = searcher.search(limits, false);
            let turn = match searcher.pos.board().side_to_move() {
                Color::White => 1,
                Color::Black => -1,
            };
            
            if !score.is_mate() && !score.is_infinite() {
                scores.push((searcher.pos.board().clone(), score * turn));
            }
            
            searcher.pos.make_move(mv);
        }
    }

    scores.iter()
        .map(|(b, s)| (b.clone(), s.clone(), result))
        .collect()
}

fn gen_many(duration: Duration, depth: u8) -> Vec<(Board, Score, f32)> {
    let start = Instant::now();
    let time_man = Arc::new(TimeManager::new());
    let mut searcher = Searcher::new(Board::default(), time_man.clone());
    let mut scores = Vec::new();

    while start.elapsed() < duration {
        scores.append(&mut gen_single(
            &mut searcher,
            &time_man,
            &[SearchLimit::MaxDepth(depth)],
        ));

        searcher.clean_ttable();
        searcher.pos.reset(Board::default());
    }

    scores
}

pub fn gen_games(file_path: &str, threads: u16, depth: u8) {
    loop {
        let (tx, rx) = mpsc::channel();
        let mut join_handlers = Vec::new();
        
        for _ in 0..threads {
            let tx = tx.clone();

            join_handlers.push(std::thread::spawn(move || {
                tx.send(gen_many(Duration::from_secs(120), depth)).unwrap()
            }));
        }
        
        for join_handler in join_handlers {
            join_handler.join().unwrap();
        }
        
        drop(tx);
        let mut output = String::new();
        let mut count: u64 = 0;
        for (board, q_score, wdl) in rx.iter().flatten() {
            output += &format!("{} | {} | {:.1}\n", board, q_score.0, wdl);
            count += 1;
        }

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)
            .unwrap();

        write!(&mut file, "{}", output).unwrap();
        
        println!("Wrote {} positions to {}", count, file_path);
    }
}