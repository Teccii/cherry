use rayon::iter::{
    IntoParallelRefIterator,
    ParallelIterator
};
use std::{
    collections::HashMap,
    fmt::{self, Write as _},
    fs::OpenOptions,
    io::Write,
    ops::*,
    str::FromStr,
    time::Instant
};
use rand::Rng;
use cozy_chess::*;
use crate::*;

/*----------------------------------------------------------------*/

pub struct TrainingData {
    trace: EvalTrace,
    result: f32,
}

/*----------------------------------------------------------------*/

macro_rules! set_grad {
    ($grad:expr, $trace:expr, $delta:expr, $elem:ident) => {
        $grad.$elem += $delta * $trace.$elem.clone();
    };
    ($grad:expr, $trace:expr, $delta:expr, $elem:ident, $($elems:ident),*) => {
        set_grad!($grad, $trace, $delta, $elem);
        set_grad!($grad, $trace, $delta, $($elems),*);
    }
}

macro_rules! tuner {
    ($($elem:ident: $ty:ty,)*) => {
        #[derive(Debug, Clone, Default)]
        pub struct TunedWeights {
            $(pub $elem: $ty),*
        }

        impl TunedWeights {
            pub fn from_weights(weights: EvalWeights_f32) -> TunedWeights {
                let mut result = TunedWeights::default();

                $(result.$elem = weights.$elem;)*

                result
            }

            pub fn to_weights(&self) -> EvalWeights_f32 {
                let mut weights = EvalWeights_f32::from(EvalWeights::default());

                $(weights.$elem = self.$elem; )*

                weights
            }

            pub fn sqrt(&self) -> TunedWeights {
                TunedWeights { $($elem: self.$elem.sqrt()),* }
            }
        }

        impl fmt::Display for TunedWeights {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                $(writeln!(f, "{}: {}", stringify!($elem), self.$elem)?;)*
                Ok(())
            }
        }

        /*----------------------------------------------------------------*/

        macro_rules! impl_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<TunedWeights> for TunedWeights {
                    type Output = Self;

                    fn $fn(self, rhs: TunedWeights) -> Self::Output {
                        TunedWeights { $($elem: self.$elem.$fn(rhs.$elem)),* }
                    }
                }
            }
        }

        macro_rules! impl_assign_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<TunedWeights> for TunedWeights {
                    fn $fn(&mut self, rhs: TunedWeights) {
                        $(self.$elem.$fn(rhs.$elem);)*
                    }
                }
            }
        }

        macro_rules! impl_f32_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<f32> for TunedWeights {
                    type Output = Self;

                    fn $fn(self, rhs: f32) -> Self::Output {
                        TunedWeights { $($elem: self.$elem.$fn(rhs)),* }
                    }
                }
            }
        }

        macro_rules! impl_f32_assign_ops {
            ($trait:ident, $fn:ident) => {
                impl $trait<f32> for TunedWeights {
                    fn $fn(&mut self, rhs: f32) {
                        $(self.$elem.$fn(rhs);)*
                    }
                }
            }
        }

        /*----------------------------------------------------------------*/

        impl_ops!(Add, add);
        impl_ops!(Sub, sub);
        impl_ops!(Mul, mul);
        impl_ops!(Div, div);

        impl_assign_ops!(AddAssign, add_assign);
        impl_assign_ops!(SubAssign, sub_assign);
        impl_assign_ops!(MulAssign, mul_assign);
        impl_assign_ops!(DivAssign, div_assign);

        impl_f32_ops!(Add, add);
        impl_f32_ops!(Sub, sub);
        impl_f32_ops!(Mul, mul);
        impl_f32_ops!(Div, div);

        impl_f32_assign_ops!(AddAssign, add_assign);
        impl_f32_assign_ops!(SubAssign, sub_assign);
        impl_f32_assign_ops!(MulAssign, mul_assign);
        impl_f32_assign_ops!(DivAssign, div_assign);

        /*----------------------------------------------------------------*/

        #[derive(Debug, Clone)]
        pub struct SgdTuner {
            weights: TunedWeights,
            grad: TunedWeights,
            cache: TunedWeights,
            learning_rate: f32,
            forgetting_factor: f32,
            regression_factor: f32,
        }

        impl SgdTuner {
            pub fn new(
                initial_weights: TunedWeights,
                learning_rate: f32,
                forgetting_factor: f32,
                regression_factor: f32
            ) -> SgdTuner {
                SgdTuner {
                    weights: initial_weights,
                    grad: TunedWeights::default(),
                    cache: TunedWeights::default(),
                    learning_rate,
                    forgetting_factor,
                    regression_factor
                }
            }

            /*----------------------------------------------------------------*/

            pub fn error(&self, data: &[TrainingData]) -> f32 {
                let weights = self.weights.to_weights();

                data.par_iter().map(|d| {
                    let pred = d.trace.apply_weights_f32(&weights);
                    let diff = d.result - sigmoid(pred, self.regression_factor);

                    diff * diff
                }).sum::<f32>() / data.len() as f32
            }

            pub fn feed_forward(&self, data: &TrainingData) -> f32 {
                data.trace.apply_weights_f32(&self.weights.to_weights())
            }

            pub fn back_prop(&mut self, data: &TrainingData) {
                let pred = sigmoid(self.feed_forward(data), self.regression_factor);
                let grad = pred - data.result;
                let sig_grad = grad * pred * (1.0 - pred) * self.learning_rate * self.regression_factor;

                let mg = (TOTAL_PHASE as f32 - data.trace.phase as f32) / TOTAL_PHASE as f32;
                let eg = 1.0 - mg;
                let delta = T_f32(mg * sig_grad, eg * sig_grad);

                let mut delta_grad = TunedWeights::default();
                set_grad!(delta_grad, data.trace, delta, $($elem),*);

                self.grad += delta_grad;
            }

            /*----------------------------------------------------------------*/

            pub fn apply(&mut self) {
                self.cache = self.cache.clone() * self.forgetting_factor
                    + self.grad.clone() * self.grad.clone() * (1.0 - self.forgetting_factor);
                self.weights -= (self.grad.clone() / (self.cache.sqrt() + 1e-8) * self.learning_rate);

                self.grad = TunedWeights::default();
            }
        }

        impl fmt::Display for SgdTuner {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.weights)
            }
        }
    }
}

tuner! {
    bishop_pair: T_f32,

    pawn_value: T_f32,
    knight_value: T_f32,
    bishop_value: T_f32,
    rook_value: T_f32,
    queen_value: T_f32,

    pawn_psqt: SquareTable_f32,
    knight_psqt: SquareTable_f32,
    bishop_psqt: SquareTable_f32,
    rook_psqt: SquareTable_f32,
    queen_psqt: SquareTable_f32,
    king_psqt: SquareTable_f32,

    knight_mobility: IndexTable_f32<9>,
    bishop_mobility: IndexTable_f32<14>,
    rook_mobility: IndexTable_f32<15>,
    queen_mobility: IndexTable_f32<28>,

    rook_open_file: FileTable_f32,
    rook_semiopen_file: FileTable_f32,
    queen_open_file: FileTable_f32,
    queen_semiopen_file: FileTable_f32,

    knight_attack: T_f32,
    bishop_attack: T_f32,
    rook_attack: T_f32,
    queen_attack: T_f32,

    pawn_minor_threat: T_f32,
    pawn_major_threat: T_f32,
    minor_major_threat: T_f32,

    passed_pawn: RankTable_f32,
    phalanx: RankTable_f32,
    backwards_pawn: T_f32,
    isolated_pawn: T_f32,
    doubled_pawn: T_f32,
    support: T_f32,

    center_control: T_f32,
}

/*----------------------------------------------------------------*/

fn sgd(data: &[TrainingData], out_path: &str, iters: u64, batch_size: u64) {
    let mut out_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(out_path)
        .unwrap();

    println!("Starting Tuner...");
    
    let start = Instant::now();
    let mut tuner = SgdTuner::new(
        TunedWeights::from_weights(EvalWeights_f32::from(EvalWeights::default())),
        0.00025,
        0.95,
        0.008
    );
    
    let mut rng = rand::rng();

    let mut i = 0;
    loop {
        for _ in 0..batch_size {
            let index = rng.random_range(0..data.len());
            tuner.back_prop(&data[index]);
        }

        tuner.apply();

        //print every 512 batches
        if i % 512 == 0 {
            let mut out_str = String::new();
            writeln!(out_str, "\n--------------------------------").unwrap();
            writeln!(out_str, "{}", tuner).unwrap();
            writeln!(out_str, "Error: {}", tuner.error(data)).unwrap();
            writeln!(out_str, "Iteration: {}", (i + 1) * batch_size).unwrap();
            writeln!(out_str, "Runtime: {:?}", start.elapsed()).unwrap();

            print!("{}", out_str);
            write!(out_file, "{}", out_str).unwrap();
        }

        i += 1;
    }
}

pub fn tune(data_path: &str, out_path: &str) {
    let mut trace_map: HashMap<EvalTrace, (f32, usize)> = HashMap::new();
    let mut evaluator = Evaluator::default();
    let data_file = std::fs::read_to_string(data_path).unwrap();
    let start = Instant::now();

    println!("Parsing data...");

    for mut reader in data_file.lines().map(|s| s.split('|')) {
        let board = reader.next().and_then(|s| Board::from_str(s.trim()).ok()).unwrap();
        let result = reader.next().and_then(|s| s.trim().parse::<f32>().ok()).unwrap();

        evaluator.eval(&board, 0);
        let trace = evaluator.trace();

        let data = trace_map.entry(trace).or_default();
        data.0 += result;
        data.1 += 1;
    }

    let data: Vec<TrainingData> = trace_map.iter().map(|(trace, (result, count))| TrainingData {
        trace: trace.clone(),
        result: *result / *count as f32,
    }).collect();

    println!("Parsed {} unique positions (took {:?})", data.len(), start.elapsed());

    sgd(&data, out_path, 100_000_000, 256);
}

/*----------------------------------------------------------------*/

fn sigmoid(x: f32, k: f32) -> f32 {
    1.0 / (1.0 + (-x * k).exp())
}