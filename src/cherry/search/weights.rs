#[cfg(feature = "tune")]
use core::cell::SyncUnsafeCell;

use crate::*;

/*----------------------------------------------------------------*/

type LmrLookup = [[i32; MAX_PLY as usize]; MAX_PLY as usize];

#[allow(clippy::approx_constant)]
static LOG: [f32; MAX_PLY as usize] = [
    0.0, 0.0, 0.6931472, 1.0986123, 1.3862944, 1.609438, 1.7917595, 1.9459101, 2.0794415,
    2.1972246, 2.3025851, 2.3978953, 2.4849067, 2.5649493, 2.6390574, 2.7080503, 2.7725887,
    2.8332133, 2.8903718, 2.944439, 2.9957323, 3.0445225, 3.0910425, 3.1354942, 3.1780539,
    3.218876, 3.2580965, 3.295837, 3.3322046, 3.3672957, 3.4011974, 3.4339871, 3.465736, 3.4965076,
    3.5263605, 3.5553482, 3.583519, 3.6109178, 3.637586, 3.6635616, 3.6888795, 3.713572, 3.7376697,
    3.7612002, 3.7841897, 3.8066626, 3.8286414, 3.8501475, 3.871201, 3.8918202, 3.912023,
    3.9318256, 3.9512436, 3.9702919, 3.988984, 4.0073333, 4.0253515, 4.0430512, 4.060443,
    4.0775375, 4.0943446, 4.1108737, 4.1271343, 4.1431346, 4.158883, 4.1743875, 4.189655, 4.204693,
    4.2195077, 4.2341065, 4.248495, 4.26268, 4.276666, 4.2904596, 4.304065, 4.317488, 4.3307333,
    4.3438053, 4.356709, 4.3694477, 4.3820267, 4.394449, 4.406719, 4.4188404, 4.4308167, 4.4426513,
    4.454347, 4.465908, 4.477337, 4.4886365, 4.4998097, 4.5108595, 4.5217886, 4.5325994, 4.543295,
    4.553877, 4.564348, 4.574711, 4.5849676, 4.59512, 4.6051702, 4.6151204, 4.624973, 4.634729,
    4.644391, 4.65396, 4.6634393, 4.6728287, 4.6821313, 4.691348, 4.7004805, 4.7095304, 4.7184987,
    4.727388, 4.7361984, 4.744932, 4.75359, 4.762174, 4.7706847, 4.7791233, 4.787492, 4.7957907,
    4.804021, 4.8121843, 4.8202815, 4.828314, 4.836282, 4.8441873, 4.8520303, 4.8598123, 4.8675346,
    4.8751974, 4.882802, 4.890349, 4.89784, 4.905275, 4.912655, 4.919981, 4.9272537, 4.934474,
    4.9416423, 4.94876, 4.955827, 4.962845, 4.9698133, 4.9767337, 4.983607, 4.9904327, 4.9972124,
    5.0039463, 5.0106354, 5.0172796, 5.0238805, 5.030438, 5.0369525, 5.043425, 5.049856, 5.056246,
    5.062595, 5.0689044, 5.075174, 5.081404, 5.0875964, 5.09375, 5.0998664, 5.1059456, 5.1119876,
    5.117994, 5.123964, 5.1298985, 5.1357985, 5.1416636, 5.1474943, 5.1532917, 5.159055, 5.164786,
    5.170484, 5.17615, 5.1817837, 5.187386, 5.192957, 5.198497, 5.2040067, 5.209486, 5.214936,
    5.220356, 5.2257466, 5.2311087, 5.236442, 5.241747, 5.247024, 5.2522736, 5.2574954, 5.26269,
    5.267858, 5.273, 5.278115, 5.2832036, 5.288267, 5.293305, 5.2983174, 5.3033047, 5.3082676,
    5.313206, 5.31812, 5.32301, 5.327876, 5.332719, 5.3375382, 5.3423343, 5.3471074, 5.351858,
    5.3565865, 5.3612924, 5.365976, 5.370638, 5.3752785, 5.379897, 5.3844953, 5.389072, 5.3936276,
    5.398163, 5.4026775, 5.4071717, 5.411646, 5.4161005, 5.420535, 5.42495, 5.4293456, 5.433722,
    5.4380794, 5.4424176, 5.4467373, 5.4510384, 5.4553213, 5.4595857, 5.463832, 5.46806, 5.4722705,
    5.4764633, 5.480639, 5.484797, 5.488938, 5.4930615, 5.497168, 5.5012584, 5.5053315, 5.5093884,
    5.5134287, 5.5174527, 5.521461, 5.525453, 5.529429, 5.5333896, 5.5373344, 5.5412636,
];

/*----------------------------------------------------------------*/

macro_rules! weights {
    ($($name:ident | $tunable:ident : $ty:ty => $default:literal | $min:literal..=$max:literal;)*) => {
        pub struct W;

        $(
            #[cfg(feature = "tune")]
            pub static $tunable: SyncUnsafeCell<$ty> = SyncUnsafeCell::new($default);
        )*

        impl W {
            $(

                #[cfg(not(feature = "tune"))]
                pub const fn $name() -> $ty { $default }
                #[cfg(feature = "tune")]
                pub const fn $name() -> $ty { unsafe { *$tunable.get() } }
            )*

            #[cfg(feature = "tune")]
            pub fn set_weight(name: &str, value: String) {
                match name {
                    $(
                        stringify!($tunable) => {
                            let value = match value.parse::<$ty>() {
                                Ok(value) => value,
                                Err(e) => {
                                    println!("info string {:?}", UciParseError::InvalidInteger(e));
                                    return;
                                }
                            };

                            if value > $max || value < $min {
                                println!("info string Invalid {} value: `{value}`", stringify!($tunable));
                                return;
                            }

                            unsafe {
                                *$tunable.get() = value;
                            }
                        },
                    )*
                    _ => println!("info string Unknown Option: `{name}`"),
                }
            }

            #[cfg(feature = "tune")]
            pub fn is_weight(name: &str) -> bool {
                match name {
                    $(stringify!($tunable) => true,)*
                    _ => false
                }
            }

            #[cfg(feature = "tune")]
            pub fn print_spsa() {
                $(
                    println!("{}, int, {:.1}, {:.1}, {:.1}, {:.2}, 0.002", stringify!($tunable), $default as f32, $min as f32, $max as f32, ($max as f32 - $min as f32).abs() / 25.0);
                )*
            }

            #[cfg(feature = "tune")]
            pub fn print_uci() {
                $(
                    println!("option name {} type spin default {} min {} max {}", stringify!($tunable), $default, $min, $max);
                )*
            }
        }
    }
}

/*----------------------------------------------------------------*/

weights! {
    pawn_corr    | PAWN_CORR:    i32 => 70  | 32..=96;
    minor_corr   | MINOR_CORR:   i32 => 55  | 32..=96;
    major_corr   | MAJOR_CORR:   i32 => 67  | 32..=96;
    nonpawn_corr | NONPAWN_CORR: i32 => 68  | 32..=96;
    cont1_corr   | CONT1_CORR:   i32 => 56  | 32..=96;
    cont2_corr   | CONT2_CORR:   i32 => 55  | 32..=96;
    corr_bonus   | CORR_BONUS:   i64 => 127 | 96..=192;

    quiet_bonus_base   | QUIET_BONUS_BASE:   i64 => 132  | 64..=256;
    quiet_bonus_scale1 | QUIET_BONUS_SCALE1: i64 => 126  | 64..=256;
    quiet_bonus_scale2 | QUIET_BONUS_SCALE2: i64 => 12   | 0..=128;
    quiet_bonus_max    | QUIET_BONUS_MAX:    i64 => 1927 | 1024..=4096;
    quiet_malus_base   | QUIET_MALUS_BASE:   i64 => 101  | 64..=256;
    quiet_malus_scale1 | QUIET_MALUS_SCALE1: i64 => 134  | 64..=256;
    quiet_malus_scale2 | QUIET_MALUS_SCALE2: i64 => 6    | 0..=128;
    quiet_malus_max    | QUIET_MALUS_MAX:    i64 => 1845 | 1024..=4096;

    noisy_bonus_base   | NOISY_BONUS_BASE:   i64 => 127  | 64..=256;
    noisy_bonus_scale1 | NOISY_BONUS_SCALE1: i64 => 122  | 64..=256;
    noisy_bonus_scale2 | NOISY_BONUS_SCALE2: i64 => 12   | 0..=128;
    noisy_bonus_max    | NOISY_BONUS_MAX:    i64 => 2047 | 1024..=4096;
    noisy_malus_base   | NOISY_MALUS_BASE:   i64 => 131  | 64..=256;
    noisy_malus_scale1 | NOISY_MALUS_SCALE1: i64 => 119  | 64..=256;
    noisy_malus_scale2 | NOISY_MALUS_SCALE2: i64 => 10   | 0..=128;
    noisy_malus_max    | NOISY_MALUS_MAX:    i64 => 2020 | 1024..=4096;

    pawn_bonus_base   | PAWN_BONUS_BASE:   i64 => 136  | 64..=256;
    pawn_bonus_scale1 | PAWN_BONUS_SCALE1: i64 => 126  | 64..=256;
    pawn_bonus_scale2 | PAWN_BONUS_SCALE2: i64 => 3    | 0..=128;
    pawn_bonus_max    | PAWN_BONUS_MAX:    i64 => 2280 | 1024..=4096;
    pawn_malus_base   | PAWN_MALUS_BASE:   i64 => 155  | 64..=256;
    pawn_malus_scale1 | PAWN_MALUS_SCALE1: i64 => 118  | 64..=256;
    pawn_malus_scale2 | PAWN_MALUS_SCALE2: i64 => 3    | 0..=128;
    pawn_malus_max    | PAWN_MALUS_MAX:    i64 => 1876 | 1024..=4096;

    cont1_bonus_base   | CONT1_BONUS_BASE:   i64 => 124  | 64..=256;
    cont1_bonus_scale1 | CONT1_BONUS_SCALE1: i64 => 127  | 64..=256;
    cont1_bonus_scale2 | CONT1_BONUS_SCALE2: i64 => 2    | 0..=128;
    cont1_bonus_max    | CONT1_BONUS_MAX:    i64 => 1638 | 1024..=4096;
    cont1_malus_base   | CONT1_MALUS_BASE:   i64 => 111  | 64..=256;
    cont1_malus_scale1 | CONT1_MALUS_SCALE1: i64 => 138  | 64..=256;
    cont1_malus_scale2 | CONT1_MALUS_SCALE2: i64 => 5    | 0..=128;
    cont1_malus_max    | CONT1_MALUS_MAX:    i64 => 2009 | 1024..=4096;

    cont2_bonus_base   | CONT2_BONUS_BASE:   i64 => 115  | 64..=256;
    cont2_bonus_scale1 | CONT2_BONUS_SCALE1: i64 => 114  | 64..=256;
    cont2_bonus_scale2 | CONT2_BONUS_SCALE2: i64 => 12   | 0..=128;
    cont2_bonus_max    | CONT2_BONUS_MAX:    i64 => 1744 | 1024..=4096;
    cont2_malus_base   | CONT2_MALUS_BASE:   i64 => 126  | 64..=256;
    cont2_malus_scale1 | CONT2_MALUS_SCALE1: i64 => 152  | 64..=256;
    cont2_malus_scale2 | CONT2_MALUS_SCALE2: i64 => 3    | 0..=128;
    cont2_malus_max    | CONT2_MALUS_MAX:    i64 => 2304 | 1024..=4096;

    cont4_bonus_base   | CONT4_BONUS_BASE:   i64 => 134  | 64..=256;
    cont4_bonus_scale1 | CONT4_BONUS_SCALE1: i64 => 112  | 64..=256;
    cont4_bonus_scale2 | CONT4_BONUS_SCALE2: i64 => 6    | 0..=128;
    cont4_bonus_max    | CONT4_BONUS_MAX:    i64 => 1869 | 1024..=4096;
    cont4_malus_base   | CONT4_MALUS_BASE:   i64 => 135  | 64..=256;
    cont4_malus_scale1 | CONT4_MALUS_SCALE1: i64 => 141  | 64..=256;
    cont4_malus_scale2 | CONT4_MALUS_SCALE2: i64 => 3    | 0..=128;
    cont4_malus_max    | CONT4_MALUS_MAX:    i64 => 2203 | 1024..=4096;

    pawn_see_value   | PAWN_SEE_VALUE:   i32 => 101 | 50..=150;
    knight_see_value | KNIGHT_SEE_VALUE: i32 => 330 | 150..=400;
    bishop_see_value | BISHOP_SEE_VALUE: i32 => 308 | 150..=400;
    rook_see_value   | ROOK_SEE_VALUE:   i32 => 602 | 400..=700;
    queen_see_value  | QUEEN_SEE_VALUE:  i32 => 992 | 700..=1100;

    mat_scale_pawn   | MAT_SCALE_PAWN:   i32 => 119   | 50..=150;
    mat_scale_knight | MAT_SCALE_KNIGHT: i32 => 367   | 150..=400;
    mat_scale_bishop | MAT_SCALE_BISHOP: i32 => 343   | 150..=400;
    mat_scale_rook   | MAT_SCALE_ROOK:   i32 => 565   | 400..=700;
    mat_scale_queen  | MAT_SCALE_QUEEN:  i32 => 981   | 700..=1100;
    mat_scale_base   | MAT_SCALE_BASE:   i32 => 25242 | 24000..=26000;

    rfp_depth      | RFP_DEPTH:      i32 => 6550 | 4096..=8192;
    rfp_base       | RFP_BASE:       i64 => 5    | -100..=100;
    rfp_scale1     | RFP_SCALE1:     i64 => 75   | 0..=200;
    rfp_scale2     | RFP_SCALE2:     i64 => 741  | 384..=1280;
    rfp_imp_base   | RFP_IMP_BASE:   i64 => -29  | -100..=100;
    rfp_imp_scale1 | RFP_IMP_SCALE1: i64 => 40   | 0..=200;
    rfp_imp_scale2 | RFP_IMP_SCALE2: i64 => 800  | 384..=1280;
    rfp_lerp       | RFP_LERP:       i32 => 571  | 256..=768;

    razor_base       | RAZOR_BASE:       i64 => 335 | 150..=400;
    razor_scale1     | RAZOR_SCALE1:     i64 => 1   | 0..=200;
    razor_scale2     | RAZOR_SCALE2:     i64 => 258 | 150..=400;
    razor_imp_base   | RAZOR_IMP_BASE:   i64 => 344 | 150..=400;
    razor_imp_scale1 | RAZOR_IMP_SCALE1: i64 => 10  | 0..=200;
    razor_imp_scale2 | RAZOR_IMP_SCALE2: i64 => 240 | 150..=400;

    nmp_depth       | NMP_DEPTH:       i32 => 3080  | 2048..=4096;
    nmp_base        | NMP_BASE:        i64 => 6151  | 4096..=8192;
    nmp_scale1      | NMP_SCALE1:      i64 => 219   | 0..=512;
    nmp_scale2      | NMP_SCALE2:      i64 => 0     | 0..=256;
    nmp_verif_depth | NMP_VERIF_DEPTH: i32 => 14535 | 12288..=16384;

    lmp_base       | LMP_BASE:       i64 => 2176 | 1024..=3072;
    lmp_scale1     | LMP_SCALE1:     i64 => 16   | 0..=512;
    lmp_scale2     | LMP_SCALE2:     i64 => 487  | 256..=768;
    lmp_imp_base   | LMP_IMP_BASE:   i64 => 4146 | 3072..=5120;
    lmp_imp_scale1 | LMP_IMP_SCALE1: i64 => 1    | 0..=512;
    lmp_imp_scale2 | LMP_IMP_SCALE2: i64 => 1062 | 512..=1536;

    fp_depth      | FP_DEPTH:      i32 => 8482 | 6144..=10240;
    fp_base       | FP_BASE:       i64 => 92   | 0..=200;
    fp_scale1     | FP_SCALE1:     i64 => 93   | 0..=200;
    fp_scale2     | FP_SCALE2:     i64 => 25   | 0..=1280;
    fp_imp_base   | FP_IMP_BASE:   i64 => 87   | 0..=200;
    fp_imp_scale1 | FP_IMP_SCALE1: i64 => 83   | 0..=200;
    fp_imp_scale2 | FP_IMP_SCALE2: i64 => 75   | 0..=1280;
    fp_hist_scale | FP_HIST_SCALE: i64 => 257  | 0..=512;

    hist_depth  | HIST_DEPTH:  i32 => 5810  | 4096..=8192;
    hist_base   | HIST_BASE:   i64 => -67   | -1000..=0;
    hist_scale1 | HIST_SCALE1: i64 => -1849 | -5000..=-1000;
    hist_scale2 | HIST_SCALE2: i64 => -11   | -500..=0;

    see_quiet_depth  | SEE_QUIET_DEPTH:  i32 => 10311 | 8192..=12288;
    see_quiet_base   | SEE_QUIET_BASE:   i64 => -6    | -200..=0;
    see_quiet_scale1 | SEE_QUIET_SCALE1: i64 => -80   | -200..=0;
    see_quiet_scale2 | SEE_QUIET_SCALE2: i64 => -6    | -100..=0;

    see_noisy_depth      | SEE_NOISY_DEPTH:      i32 => 10409 | 8192..=12288;
    see_noisy_base       | SEE_NOISY_BASE:       i64 => -3    | -200..=0;
    see_noisy_scale1     | SEE_NOISY_SCALE1:     i64 => -49   | -200..=0;
    see_noisy_scale2     | SEE_NOISY_SCALE2:     i64 => -10   | -100..=0;
    see_noisy_hist_scale | SEE_NOISY_HIST_SCALE: i64 => 233   | 0..=512;

    se_depth             | SE_DEPTH:             i32 => 6037 | 4096..=6144;
    se_tt_depth          | SE_TT_DEPTH:          i32 => 3018 | 2560..=3584;
    se_search_depth      | SE_SEARCH_DEPTH:      i64 => 480  | 256..=768;
    se_beta_margin       | SE_BETA_MARGIN:       i32 => 95   | 64..=96;
    se_double_ext_base   | SE_DOUBLE_EXT_BASE:   i32 => 32   | 0..=40;
    se_double_ext_pv     | SE_DOUBLE_EXT_PV:     i32 => 160  | 96..=256;
    se_triple_ext_margin | SE_TRIPLE_EXT_MARGIN: i32 => 69   | 40..=80;
    se_ext               | SE_EXT:               i32 => 1064 | 512..=1536;
    se_double_ext        | SE_DOUBLE_EXT:        i32 => 2176 | 1536..=2560;
    se_triple_ext        | SE_TRIPLE_EXT:        i32 => 3100 | 2560..=3584;
    se_beta_ext          | SE_BETA_EXT:          i32 => -983 | -1536..=-768;
    se_cut_ext           | SE_CUT_EXT:           i32 => -945 | -1536..=-768;

    lmr_quiet_base | LMR_QUIET_BASE: i32 => 690  | 256..=768;
    lmr_quiet_div  | LMR_QUIET_DIV:  i32 => 1588 | 1024..=2048;
    lmr_noisy_base | LMR_NOISY_BASE: i32 => 492  | 256..=768;
    lmr_noisy_div  | LMR_NOISY_DIV:  i32 => 3675 | 3072..=4096;

    lmr_depth       | LMR_DEPTH:       i32 => 2029 | 1536..=2560;
    quiet_hist_lmr  | QUIET_HIST_LMR:  i32 => 960  | 512..=1536;
    noisy_hist_lmr  | NOISY_HIST_LMR:  i32 => 1040 | 512..=1536;
    check_lmr       | CHECK_LMR:       i32 => 1110 | 512..=1536;
    non_pv_lmr      | NON_PV_LMR:      i32 => 1070 | 512..=1536;
    tt_pv_lmr       | TT_PV_LMR:       i32 => 1021 | 512..=1536;
    cut_lmr         | CUT_LMR:         i32 => 992  | 512..=1536;
    imp_lmr         | IMP_LMR:         i32 => 966  | 512..=1536;

    mp_see_margin    | MP_SEE_MARGIN:    i32 => -31 | -200..=100;
    mp_qs_see_margin | MP_QS_SEE_MARGIN: i32 => -18 | -200..=100;

    qfp_margin | QFP_MARGIN: i32 => 127 | 50..=300;

    asp_window_initial | ASP_WINDOW_INITIAL: i32 => 16 | 10..=40;
    asp_window_expand  | ASP_WINDOW_EXPAND:  i32 => 66 | 48..=96;

    soft_time_div         | SOFT_TIME_DIV:         u64 => 250648 | 196608..=294912;
    soft_time_inc         | SOFT_TIME_INC:         u64 => 4027   | 2048..=4096;
    hard_time_div         | HARD_TIME_DIV:         u64 => 11864  | 8192..=16384;
    hard_time_inc         | HARD_TIME_INC:         u64 => 3877   | 2048..=4096;
    subtree_base          | SUBTREE_BASE:          i64 => 9843   | 8192..=12288;
    subtree_scale         | SUBTREE_SCALE:         i64 => 6510   | 4096..=8192;
    subtree_min           | SUBTREE_MIN:           i64 => 4057   | 2048..=4096;
    move_stability_base   | MOVE_STABILITY_BASE:   i64 => 7463   | 6144..=9216;
    move_stability_scale  | MOVE_STABILITY_SCALE:  i64 => 465    | 256..=768;
    move_stability_min    | MOVE_STABILITY_MIN:    i64 => 3553   | 2048..=4096;
    score_stability_edge  | SCORE_STABILITY_EDGE:  i32 => 19     | 10..=40;
    score_stability_base  | SCORE_STABILITY_BASE:  i64 => 7617   | 6144..=9216;
    score_stability_scale | SCORE_STABILITY_SCALE: i64 => 353    | 256..=768;
    score_stability_min   | SCORE_STABILITY_MIN:   i64 => 3864   | 2048..=4096;
}

impl W {
    #[inline]
    pub const fn see_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => W::pawn_see_value(),
            Piece::Knight => W::knight_see_value(),
            Piece::Bishop => W::bishop_see_value(),
            Piece::Rook => W::rook_see_value(),
            Piece::Queen => W::queen_see_value(),
            Piece::King => 20000,
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub const fn rfp_margin(improving: bool, depth: i32) -> i64 {
        let depth = depth as i64;
        let depth_scale = DEPTH_SCALE as i64;
        let (base, scale1, scale2) = if improving {
            (W::rfp_imp_base(), W::rfp_imp_scale1(), W::rfp_imp_scale2())
        } else {
            (W::rfp_base(), W::rfp_scale1(), W::rfp_scale2())
        };

        let scale1 = scale1 * depth / depth_scale;
        let scale2 = scale2 * depth * depth / (depth_scale * depth_scale * 128);

        base + scale1 + scale2
    }

    #[inline]
    pub const fn razor_margin(improving: bool, depth: i32) -> i64 {
        let depth = depth as i64;
        let depth_scale = DEPTH_SCALE as i64;
        let (base, scale1, scale2) = if improving {
            (
                W::razor_imp_base(),
                W::razor_imp_scale1(),
                W::razor_imp_scale2(),
            )
        } else {
            (W::razor_base(), W::razor_scale1(), W::razor_scale2())
        };
        let scale1 = scale1 * depth / depth_scale;
        let scale2 = scale2 * depth * depth / (depth_scale * depth_scale);

        base + scale1 + scale2
    }

    #[inline]
    pub const fn nmp_reduction(depth: i32) -> i64 {
        let depth = depth as i64;
        let depth_scale = DEPTH_SCALE as i64;
        let scale1 = W::nmp_scale1() * depth / depth_scale;
        let scale2 = W::nmp_scale2() * depth * depth / (depth_scale * depth_scale);

        W::nmp_base() + scale1 + scale2
    }

    #[inline]
    pub const fn lmp_margin(improving: bool, depth: i32) -> i64 {
        let depth = depth as i64;
        let depth_scale = DEPTH_SCALE as i64;
        let (base, scale1, scale2) = if improving {
            (W::lmp_imp_base(), W::lmp_imp_scale1(), W::lmp_imp_scale2())
        } else {
            (W::lmp_base(), W::lmp_scale1(), W::lmp_scale2())
        };
        let scale1 = scale1 * depth / depth_scale;
        let scale2 = scale2 * depth * depth / (depth_scale * depth_scale);

        base + scale1 + scale2
    }

    #[inline]
    pub const fn fp_margin(improving: bool, depth: i32, hist_score: i32) -> i64 {
        let depth = depth as i64;
        let depth_scale = DEPTH_SCALE as i64;
        let (base, scale1, scale2) = if improving {
            (W::fp_imp_base(), W::fp_imp_scale1(), W::fp_imp_scale2())
        } else {
            (W::fp_base(), W::fp_scale1(), W::fp_scale2())
        };
        let scale1 = scale1 * depth / depth_scale;
        let scale2 = scale2 * depth * depth / (depth_scale * depth_scale * 128);
        let hist_scale = W::fp_hist_scale() * hist_score as i64 / MAX_HISTORY as i64;

        base + scale1 + scale2 + hist_scale
    }

    #[inline]
    pub const fn hist_margin(depth: i32) -> i64 {
        let depth = depth as i64;
        let depth_scale = DEPTH_SCALE as i64;
        let scale1 = W::hist_scale1() * depth / depth_scale;
        let scale2 = W::hist_scale2() * depth * depth / (depth_scale * depth_scale);

        W::hist_base() + scale1 + scale2
    }

    #[inline]
    pub const fn see_quiet_margin(depth: i32) -> i64 {
        let depth = depth as i64;
        let depth_scale = DEPTH_SCALE as i64;
        let scale1 = W::see_quiet_scale1() * depth / depth_scale;
        let scale2 = W::see_quiet_scale2() * depth / (depth_scale * depth_scale);

        W::see_quiet_base() + scale1 + scale2
    }

    #[inline]
    pub const fn see_noisy_margin(depth: i32, hist_score: i32) -> i64 {
        let depth = depth as i64;
        let depth_scale = DEPTH_SCALE as i64;
        let scale1 = W::see_noisy_scale1() * depth / depth_scale;
        let scale2 = W::see_noisy_scale2() * depth * depth / (depth_scale * depth_scale);
        let hist_scale = W::see_noisy_hist_scale() * hist_score as i64 / MAX_HISTORY as i64;

        W::see_noisy_base() + scale1 + scale2 - hist_scale
    }

    #[inline]
    pub fn hist_lmr(is_noisy: bool) -> i32 {
        if is_noisy {
            W::noisy_hist_lmr()
        } else {
            W::quiet_hist_lmr()
        }
    }

    #[inline]
    pub fn lmr(is_noisy: bool, depth: i32, moves_seen: u8) -> i32 {
        let depth = (depth / DEPTH_SCALE) as u8;
        let (base, div) = if is_noisy {
            (W::lmr_noisy_base(), W::lmr_noisy_div())
        } else {
            (W::lmr_quiet_base(), W::lmr_quiet_div())
        };
        let (base, div) = (base as f32 / 1024.0, div as f32 / 1024.0);

        DEPTH_SCALE * (base + LOG[depth as usize] * LOG[moves_seen as usize] / div) as i32
    }
}
