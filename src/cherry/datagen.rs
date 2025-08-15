use std::{
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
    sync::{Arc, atomic::*},
    fs,
};
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
use rand::{Rng, rngs::ThreadRng};
use colored::Colorize;
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
    println!("\n{}", "============ Data Generation Options ============".bright_green());
    println!("Count: {}", count.to_string().bright_green());
    println!("Threads: {}", threads.to_string().bright_green());
    println!("DFRC: {}", dfrc.to_string().bright_green());

    let options = DataGenOptions { count, threads, dfrc };
    let time_stamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let dir_name = if dfrc {
        format!("DFRC_{}", time_stamp)
    } else {
        time_stamp
    };

    let data_dir = PathBuf::from("data").join(dir_name);
    fs::create_dir_all(&data_dir).unwrap();

    println!("Output Directory: {}", data_dir.display().to_string().bright_green());
    println!("{}\n", "=================================================".bright_green());

    let stats = Arc::new(GameStats::new());
    let pos_counter = Arc::new(AtomicU64::new(0));
    let game_counter = Arc::new(AtomicUsize::new(0));
    let abort = Arc::new(AtomicBool::new(false));
    let a = Arc::clone(&abort);
    let start = Instant::now();

    viriformat::chess::CHESS960.store(options.dfrc, Ordering::Relaxed);
    ctrlc::set_handler(move || a.store(true, Ordering::Relaxed)).unwrap();
    std::thread::scope(|s| {
        for thread in 0..threads {
            let data_dir = &data_dir;
            let stats = Arc::clone(&stats);
            let pos_counter = Arc::clone(&pos_counter);
            let game_counter = Arc::clone(&game_counter);
            let abort = Arc::clone(&abort);

            s.spawn(move || datagen_worker(thread, options, data_dir, stats, pos_counter, game_counter, abort));
        }
    });

    let games = game_counter.load(Ordering::Relaxed);
    let white_wins = stats.white_wins.load(Ordering::Relaxed);
    let black_wins = stats.black_wins.load(Ordering::Relaxed);
    let draws = stats.draws.load(Ordering::Relaxed);

    let white_percentage = 100f32 * (white_wins as f32 / games as f32);
    let black_percentage = 100f32 * (black_wins as f32 / games as f32);
    let draw_percentage = 100f32 * (draws as f32 / games as f32);

    let (hours, minutes, seconds) = secs_to_hms(start.elapsed().as_secs() as u32);

    println!("{}", "=== Data Generation Summary ===".bright_green());
    println!("Total Games: {}", games.to_string().bright_green());
    println!("Total Positions: {}", fmt_big_num(pos_counter.load(Ordering::Relaxed)).bright_green());
    println!("\nWhite Wins: {} ({}%)", white_wins.to_string().bright_green(), format!("{:.1}", white_percentage).bright_green());
    println!("Black Wins: {} ({}%)", black_wins.to_string().bright_green(), format!("{:.1}", black_percentage).bright_green());
    println!("Draws:      {} ({}%)", draws.to_string().bright_green(), format!("{:.1}", draw_percentage).bright_green());
    println!(
        "\nTotal Time: {}h {}m {}s",
         hours.to_string().bright_green(),
         minutes.to_string().bright_green(),
         seconds.to_string().bright_green()
    );
    println!("{}", "===============================".bright_green());
}

fn datagen_worker(
    thread: usize,
    options: DataGenOptions,
    data_dir: &PathBuf,
    stats: Arc<GameStats>,
    pos_counter: Arc<AtomicU64>,
    game_counter: Arc<AtomicUsize>,
    abort: Arc<AtomicBool>,
) {
    let mut rng = rand::rng();
    let mut output = fs::File::create(data_dir.join(format!("thread{}.bin", thread))).unwrap();
    let mut writer = BufWriter::new(&mut output);
    let mut searcher = Searcher::new(
        Board::default(),
        Arc::new(TimeManager::new()),
    );

    let start = Instant::now();
    let limits = vec![SearchLimit::MaxNodes(5000)];
    let count = usize::max(options.count / options.threads, 1);
    let mut i = 0;

    while i < count {
        if abort.load(Ordering::Relaxed) {
            if thread == 0 {
                println!("\x1B[9EReceived Ctrl+C, aborting..."); //Move cursor down 7 lines
            }

            break;
        }

        searcher.clean_ttable();
        searcher.pos.set_board(if options.dfrc {
            DfrcOpeningGenerator::gen_opening(&mut rng)
        } else {
            StdOpeningGenerator::gen_opening(&mut rng)
        }, &searcher.shared_ctx.weights);

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
        let mut game_len = 0;
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
            game_len += 1;

            if eval.is_decisive() {
                result = if eval > 0 {
                    GameOutcome::WhiteWin(WinType::Mate)
                } else {
                    GameOutcome::BlackWin(WinType::Mate)
                };

                break 'game;
            }

            searcher.make_move(mv);
            searcher.reset_nnue();

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

        let curr = game_counter.fetch_add(1, Ordering::Relaxed);
        let curr_pos = pos_counter.fetch_add(game_len, Ordering::Relaxed);

        if thread == 0 && !abort.load(Ordering::Relaxed) && curr != 0 {
            let percentage = 100f32 * (curr as f32 / options.count as f32);
            let elapsed = start.elapsed().as_secs_f32();
            let progress = percentage as usize / 2;
            let seconds = (options.count - curr) as f32 * (elapsed / curr as f32);
            let (hours, minutes, seconds) = secs_to_hms(seconds as u32);

            println!(
                "\x1B[0JGenerated {}/{} Games {} ({}%).",
                curr.to_string().bright_green(),
                options.count.to_string().bright_green(),
                progress_bar(progress, 50),
                format!("{:.1}", percentage).bright_green()
            );
            println!("Number of Positions: {}", fmt_big_num(curr_pos).bright_green());
            println!(
                "Positions Per Second: {}",
                format!("{}", (curr_pos as f32 / elapsed) as usize).bright_green()
            );
            println!(
                "Games Per Second: {}",
                format!("{:.3}", curr as f32 / elapsed).bright_green()
            );
            println!(
                "Estimated Time Remaining: {}h {}m {}s",
                hours.to_string().bright_green(),
                minutes.to_string().bright_green(),
                seconds.to_string().bright_green()
            );

            let white_wins = stats.white_wins.load(Ordering::Relaxed);
            let black_wins = stats.black_wins.load(Ordering::Relaxed);
            let draws = stats.draws.load(Ordering::Relaxed);

            let white_percentage = 100f32 * (white_wins as f32 / curr as f32);
            let black_percentage = 100f32 * (black_wins as f32 / curr as f32);
            let draw_percentage = 100f32 * (draws as f32 / curr as f32);

            println!("White Wins: {} ({}%)", white_wins.to_string().bright_green(), format!("{:.1}", white_percentage).bright_green());
            println!("Black Wins: {} ({}%)", black_wins.to_string().bright_green(), format!("{:.1}", black_percentage).bright_green());
            println!("Draws:      {} ({}%)", draws.to_string().bright_green(), format!("{:.1}", draw_percentage).bright_green());
            println!("\x1B[9F");

            io::stdout().flush().unwrap();
        }

        i += 1;
    }

    if thread == 0 && !abort.load(Ordering::Relaxed) {
        println!("\x1B[9E");
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