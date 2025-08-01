use std::{
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
    fs,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::time::Instant;
use rand::{Rng, rngs::ThreadRng};
use viriformat::{
    chess::{
        board::{
            Board as ViriBoard,
            GameOutcome,
            DrawType,
            WinType,
        },
        piece::PieceType as ViriPiece,
        types::Square as ViriSquare,
        chessmove::{
            Move as ViriMove,
            MoveFlags as ViriMoveFlag,
        },
    },
    dataformat::Game,
};
use cherry_chess::*;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
struct DataGenOptions {
    pub count: usize,
    pub threads: usize,
    pub dfrc: bool,
}

struct GameStats {
    white_wins: AtomicUsize,
    black_wins: AtomicUsize,
    draws: AtomicUsize,
}

impl GameStats {
    #[inline]
    pub fn new() -> Self {
        Self {
            white_wins: AtomicUsize::new(0),
            black_wins: AtomicUsize::new(0),
            draws: AtomicUsize::new(0),
        }
    }
}

/*----------------------------------------------------------------*/

pub fn datagen(count: usize, threads: usize, dfrc: bool) {
    println!("Starting data generation...");

    let options = DataGenOptions { count, threads, dfrc };
    let data_dir = PathBuf::from("data")
        .join(chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string());
    fs::create_dir_all(&data_dir).unwrap();

    println!("Generated data will be stored in {}", data_dir.display());
    println!("You can safely stop generating by pressing Ctrl+C");

    let stats = Arc::new(GameStats::new());
    let counter = Arc::new(AtomicUsize::new(0));
    let abort = Arc::new(AtomicBool::new(false));
    let a = Arc::clone(&abort);

    viriformat::chess::CHESS960.store(options.dfrc, Ordering::Relaxed);
    ctrlc::set_handler(move || a.store(true, Ordering::Relaxed)).unwrap();
    rayon::scope(|s| {
        for thread in 0..threads {
            let data_dir = &data_dir;
            let stats = Arc::clone(&stats);
            let counter = Arc::clone(&counter);
            let abort = Arc::clone(&abort);

            s.spawn(move |_| datagen_worker(thread, options, data_dir, stats, counter, abort));
        }
    });

    println!("\x1B[7BData Generation Finished!"); //Move cursor down 7 lines
}

fn datagen_worker(
    thread: usize,
    options: DataGenOptions,
    data_dir: &PathBuf,
    stats: Arc<GameStats>,
    counter: Arc<AtomicUsize>,
    abort: Arc<AtomicBool>,
) {
    let mut rng = rand::rng();
    let mut output = fs::File::create(data_dir.join(format!("thread{}.bin", thread))).unwrap();
    let mut writer = BufWriter::new(&mut output);
    let mut searcher = Searcher::new(
        Board::default(),
        Arc::new(TimeManager::new()),
        #[cfg(feature = "nnue")]NetworkWeights::default(),
    );

    let start = Instant::now();
    let limits = vec![SearchLimit::MaxNodes(5000)];
    let count = usize::max(options.count / options.threads, 1);
    let mut i = 0;

    while i < count {
        if abort.load(Ordering::Relaxed) {
            if thread == 0 {
                println!("\x1B[7BReceived Ctrl+C, aborting..."); //Move cursor down 7 lines
            }

            break;
        }

        searcher.clean_ttable();
        searcher.pos.set_board(if options.dfrc {
            DfrcOpeningGenerator::gen_opening(&mut rng)
        } else {
            StdOpeningGenerator::gen_opening(&mut rng)
        }, #[cfg(feature = "nnue")]&searcher.shared_ctx.nnue_weights);

        let eval = searcher.search::<NoInfo>(limits.clone()).2;
        if eval.abs() > 1000 {
            continue;
        }

        let mut initial_board = ViriBoard::new();
        initial_board.set_from_fen(if options.dfrc {
            format!("{:#}", searcher.pos.board())
        } else {
            format!("{}", searcher.pos.board())
        }.as_str()).unwrap();

        let mut game = Game::new(&initial_board);
        let result;

        'game: loop {
            let (mv, _, eval, _, _) = searcher.search::<NoInfo>(limits.clone());
            let (from, to) = (
                ViriSquare::new(mv.from() as u8).unwrap(),
                ViriSquare::new(mv.to() as u8).unwrap()
            );

            let viri_move = match mv.flag() {
                MoveFlag::None => ViriMove::new(from, to),
                MoveFlag::Promotion => ViriMove::new_with_promo(from, to, ViriPiece::new(mv.promotion().unwrap() as u8).unwrap()),
                MoveFlag::EnPassant => ViriMove::new_with_flags(from, to, ViriMoveFlag::EnPassant),
                MoveFlag::Castling => ViriMove::new_with_flags(from, to, ViriMoveFlag::Castle),
            };
            let eval = eval * searcher.pos.stm().sign();

            game.add_move(viri_move, eval.0);
            if eval.is_decisive() {
                result = if eval > 0 {
                    GameOutcome::WhiteWin(WinType::Mate)
                } else {
                    GameOutcome::BlackWin(WinType::Mate)
                };

                break 'game;
            }

            searcher.pos.make_move(mv);
            #[cfg(feature = "nnue")]searcher.pos.reset(&searcher.shared_ctx.nnue_weights);

            if searcher.pos.is_draw() || searcher.pos.board().fullmove_count() >= 300 {
                result = if searcher.pos.insufficient_material() {
                    GameOutcome::Draw(DrawType::InsufficientMaterial)
                } else if searcher.pos.repetition() {
                    GameOutcome::Draw(DrawType::Repetition)
                } else if searcher.pos.board().halfmove_clock() >= 100 {
                    GameOutcome::Draw(DrawType::FiftyMoves)
                } else {
                    GameOutcome::Draw(DrawType::Stalemate)
                };

                break 'game;
            }

            if searcher.pos.board().status() == BoardStatus::Checkmate {
                result = match searcher.pos.stm() {
                    Color::White => GameOutcome::WhiteWin(WinType::Mate),
                    Color::Black => GameOutcome::BlackWin(WinType::Mate)
                };

                break 'game;
            }
        }

        game.set_outcome(result);
        game.serialise_into(&mut writer).unwrap();

        match result {
            GameOutcome::WhiteWin(_) => { stats.white_wins.fetch_add(1, Ordering::Relaxed); },
            GameOutcome::BlackWin(_) => { stats.black_wins.fetch_add(1, Ordering::Relaxed); },
            GameOutcome::Draw(_) => { stats.draws.fetch_add(1, Ordering::Relaxed); },
            _ => { }
        }

        counter.fetch_add(1, Ordering::Relaxed);
        let curr = counter.load(Ordering::Relaxed);

        if !abort.load(Ordering::Relaxed) && curr != 0 && curr % 10 == 0 {
            let percentage = 100f32 * (curr as f32 / options.count as f32);
            let elapsed = start.elapsed().as_secs_f32();

            println!("Generated {}/{} Games ({:.1}%). ", curr, options.count, percentage);
            println!("Average Time Per Game: {:.2} seconds. ", elapsed / curr as f32);
            println!("Estimated Time Remaining: {:.2} seconds. ", (options.count - curr) as f32 * (elapsed / curr as f32));

            let white_wins = stats.white_wins.load(Ordering::Relaxed);
            let black_wins = stats.black_wins.load(Ordering::Relaxed);
            let draws = stats.draws.load(Ordering::Relaxed);

            let white_percentage = 100f32 * (white_wins as f32 / curr as f32);
            let black_percentage = 100f32 * (black_wins as f32 / curr as f32);
            let draw_percentage = 100f32 * (draws as f32 / curr as f32);

            println!("White Wins: {} ({:.1}%)", white_wins, white_percentage);
            println!("Black Wins: {} ({:.1}%)", black_wins, black_percentage);
            println!("Draws: {} ({:.1}%)", draws, draw_percentage);
            println!("\x1B[7A"); //Move cursor up 7 lines

            io::stdout().flush().unwrap();
        }

        i += 1;
    }

    writer.flush().unwrap();
}

/*----------------------------------------------------------------*/

trait OpeningGenerator {
    fn gen_opening(rng: &mut ThreadRng) -> Board;
}

/*----------------------------------------------------------------*/

pub struct StdOpeningGenerator;
pub struct DfrcOpeningGenerator;

/*----------------------------------------------------------------*/

impl OpeningGenerator for StdOpeningGenerator {
    fn gen_opening(rng: &mut ThreadRng) -> Board {
        let mut board = Board::default();
        let moves = 8 + rng.random_bool(0.5) as usize;

        for _ in 0..moves {
            let mut legals = Vec::new();
            board.gen_moves(|moves| {
                legals.extend(moves);
                false
            });

            board.make_move(legals[rng.random_range(0..legals.len())]);
            if board.status() != BoardStatus::Ongoing {
                return Self::gen_opening(rng);
            }
        }

        board
    }
}


/*----------------------------------------------------------------*/

impl OpeningGenerator for DfrcOpeningGenerator {
    fn gen_opening(rng: &mut ThreadRng) -> Board {
        let mut board = BoardBuilder::double_chess960(
            rng.random_range(0..960),
            rng.random_range(0..960),
        ).build().unwrap();
        let moves = 8 + rng.random_bool(0.5) as usize;

        for _ in 0..moves {
            let mut legals = Vec::new();
            board.gen_moves(|moves| {
                legals.extend(moves);
                false
            });

            board.make_move(legals[rng.random_range(0..legals.len())]);

            if board.status() != BoardStatus::Ongoing {
                return Self::gen_opening(rng);
            }
        }

        board
    }
}