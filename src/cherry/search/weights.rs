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
    ($($name:ident | $tunable:ident : $ty:ty => $default:expr,)*) => {
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
        }
    }
}

/*----------------------------------------------------------------*/

weights! {
    pawn_corr_frac   | PAWN_CORR_FRAC:   i32 => 64,
    minor_corr_frac  | MINOR_CORR_FRAC:  i32 => 64,
    major_corr_frac  | MAJOR_CORR_FRAC:  i32 => 64,
    stm_corr_frac    | STM_CORR_FRAC:    i32 => 64,
    ntm_corr_frac    | NTM_CORR_FRAC:    i32 => 64,
    corr_bonus_scale | CORR_BONUS_SCALE: i64 => 128,

    quiet_bonus_base  | QUIET_BONUS_BASE:  i32 => 128,
    quiet_bonus_scale | QUIET_BONUS_SCALE: i32 => 128,
    quiet_bonus_max   | QUIET_BONUS_MAX:   i32 => 2048,
    quiet_malus_base  | QUIET_MALUS_BASE:  i32 => 128,
    quiet_malus_scale | QUIET_MALUS_SCALE: i32 => 128,
    quiet_malus_max   | QUIET_MALUS_MAX:   i32 => 2048,

    tactic_bonus_base  | TACTIC_BONUS_BASE:  i32 => 128,
    tactic_bonus_scale | TACTIC_BONUS_SCALE: i32 => 128,
    tactic_bonus_max   | TACTIC_BONUS_MAX:   i32 => 2048,
    tactic_malus_base  | TACTIC_MALUS_BASE:  i32 => 128,
    tactic_malus_scale | TACTIC_MALUS_SCALE: i32 => 128,
    tactic_malus_max   | TACTIC_MALUS_MAX:   i32 => 2048,

    pawn_bonus_base  | PAWN_BONUS_BASE:  i32 => 128,
    pawn_bonus_scale | PAWN_BONUS_SCALE: i32 => 128,
    pawn_bonus_max   | PAWN_BONUS_MAX:   i32 => 2048,
    pawn_malus_base  | PAWN_MALUS_BASE:  i32 => 128,
    pawn_malus_scale | PAWN_MALUS_SCALE: i32 => 128,
    pawn_malus_max   | PAWN_MALUS_MAX:   i32 => 2048,

    cont1_bonus_base  | CONT1_BONUS_BASE:  i32 => 128,
    cont1_bonus_scale | CONT1_BONUS_SCALE: i32 => 128,
    cont1_bonus_max   | CONT1_BONUS_MAX:   i32 => 2048,
    cont1_malus_base  | CONT1_MALUS_BASE:  i32 => 128,
    cont1_malus_scale | CONT1_MALUS_SCALE: i32 => 128,
    cont1_malus_max   | CONT1_MALUS_MAX:   i32 => 2048,

    cont2_bonus_base  | CONT2_BONUS_BASE:  i32 => 128,
    cont2_bonus_scale | CONT2_BONUS_SCALE: i32 => 128,
    cont2_bonus_max   | CONT2_BONUS_MAX:   i32 => 2048,
    cont2_malus_base  | CONT2_MALUS_BASE:  i32 => 128,
    cont2_malus_scale | CONT2_MALUS_SCALE: i32 => 128,
    cont2_malus_max   | CONT2_MALUS_MAX:   i32 => 2048,

    pawn_see_value   | PAWN_SEE_VALUE:   i32 => 101,
    knight_see_value | KNIGHT_SEE_VALUE: i32 => 324,
    bishop_see_value | BISHOP_SEE_VALUE: i32 => 332,
    rook_see_value   | ROOK_SEE_VALUE:   i32 => 578,
    queen_see_value  | QUEEN_SEE_VALUE:  i32 => 981,

    pawn_mat_scale   | PAWN_MAT_SCALE:   i32 => 111,
    knight_mat_scale | KNIGHT_MAT_SCALE: i32 => 349,
    bishop_mat_scale | BISHOP_MAT_SCALE: i32 => 338,
    rook_mat_scale   | ROOK_MAT_SCALE:   i32 => 590,
    queen_mat_scale  | QUEEN_MAT_SCALE:  i32 => 973,
    mat_scale_base   | MAT_SCALE_BASE:   i32 => 25100,
    eval_scale       | EVAL_SCALE:       i32 => 400,

    rfp_depth     | RFP_DEPTH:     i32 => 6144,
    rfp_base      | RFP_BASE:      i32 => 0,
    rfp_scale     | RFP_SCALE:     i32 => 80,
    rfp_imp_base  | RFP_IMP_BASE:  i32 => -80,
    rfp_imp_scale | RFP_IMP_SCALE: i32 => 80,
    rfp_lerp      | RFP_LERP:      i32 => 512,

    nmp_depth       | NMP_DEPTH:       i32 => 3072,
    nmp_base        | NMP_BASE:        i64 => 3072,
    nmp_scale       | NMP_SCALE:       i64 => 340,
    nmp_verif_depth | NMP_VERIF_DEPTH: i32 => 14336,

    lmp_base      | LMP_BASE:      i64 => 2048,
    lmp_scale     | LMP_SCALE:     i64 => 512,
    lmp_imp_base  | LMP_IMP_BASE:  i64 => 4096,
    lmp_imp_scale | LMP_IMP_SCALE: i64 => 1024,

    fp_depth     | FP_DEPTH:     i32 => 8192,
    fp_base      | FP_BASE:      i32 => 93,
    fp_scale     | FP_SCALE:     i32 => 79,
    fp_imp_base  | FP_IMP_BASE:  i32 => 93,
    fp_imp_scale | FP_IMP_SCALE: i32 => 79,

    hist_depth | HIST_DEPTH: i32 => 6144,
    hist_base  | HIST_BASE:  i32 => 0,
    hist_scale | HIST_SCALE: i32 => -2000,

    see_quiet_depth | SEE_QUIET_DEPTH: i32 => 10240,
    see_quiet_base  | SEE_QUIET_BASE:  i32 => 0,
    see_quiet_scale | SEE_QUIET_SCALE: i32 => -89,

    see_tactic_depth | SEE_TACTIC_DEPTH: i32 => 10240,
    see_tactic_base  | SEE_TACTIC_BASE:  i32 => 0,
    see_tactic_scale | SEE_TACTIC_SCALE: i32 => -62,

    singular_depth        | SINGULAR_DEPTH:        i32 => 6144,
    singular_tt_depth     | SINGULAR_TT_DEPTH:     i32 => 3072,
    singular_beta_margin  | SINGULAR_BETA_MARGIN:  i32 => 96,
    singular_search_depth | SINGULAR_SEARCH_DEPTH: i32 => 512,
    singular_dext_margin  | SINGULAR_DEXT_MARGIN:  i32 => 30,
    singular_ext          | SINGULAR_EXT:          i32 => 1024,
    singular_dext         | SINGULAR_DEXT:         i32 => 2048,
    singular_tt_ext       | SINGULAR_TT_EXT:       i32 => -1024,
    singular_cut_ext      | SINGULAR_CUT_EXT:      i32 => -1024,

    lmr_quiet_base  | LMR_QUIET_BASE:  i32 => 579,
    lmr_quiet_div   | LMR_QUIET_DIV:   i32 => 1626,
    lmr_tactic_base | LMR_TACTIC_BASE: i32 => 450,
    lmr_tactic_div  | LMR_TACTIC_DIV:  i32 => 3688,

    lmr_depth     | LMR_DEPTH:     i32 => 2048,
    cut_lmr       | CUT_LMR:       i32 => 1024,
    improving_lmr | IMPROVING_LMR: i32 => 1024,
    non_pv_lmr    | NON_PV_LMR:    i32 => 1024,
    tt_pv_lmr     | TT_PV_LMR:     i32 => 1024,
    check_lmr     | CHECK_LMR:     i32 => 1024,

    lmr_depth_bias    | LMR_DEPTH_BIAS:    i32 => 0,
    lmr_depth_pv_bias | LMR_DEPTH_PV_BIAS: i32 => 0,
    tt_depth_bias     | TT_DEPTH_BIAS:     i32 => 0,
    tt_depth_pv_bias  | TT_DEPTH_PV_BIAS:  i32 => 0,

    asp_window_initial | ASP_WINDOW_INITIAL: i32 => 20,
    asp_window_expand  | ASP_WINDOW_EXPAND:  i32 => 48,

    soft_time_div         | SOFT_TIME_DIV:         u64 => 262144,
    hard_time_div         | HARD_TIME_DIV:         u64 => 12288,
    subtree_base          | SUBTREE_BASE:          i64 => 10240,
    subtree_scale         | SUBTREE_SCALE:         i64 => 6144,
    subtree_min           | SUBTREE_MIN:           i64 => 4096,
    move_stability_base   | MOVE_STABILITY_BASE:   i64 => 7373,
    move_stability_scale  | MOVE_STABILITY_SCALE:  i64 => 410,
    move_stability_min    | MOVE_STABILITY_MIN:    i64 => 3686,
    score_stability_edge  | SCORE_STABILITY_EDGE:  i32 => 20,
    score_stability_base  | SCORE_STABILITY_BASE:  i64 => 7373,
    score_stability_scale | SCORE_STABILITY_SCALE: i64 => 410,
    score_stability_min   | SCORE_STABILITY_MIN:   i64 => 3686,
    score_trend_base      | SCORE_TREND_BASE:      i64 => 3277,
    score_trend_scale     | SCORE_TREND_SCALE:     i64 => 205,
    score_trend_max       | SCORE_TREND_MAX:       i64 => 5939,
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
    pub const fn rfp_margin(improving: bool, depth: i32) -> i32 {
        let (base, scale) = if improving {
            (W::rfp_imp_base(), W::rfp_imp_scale())
        } else {
            (W::rfp_base(), W::rfp_scale())
        };

        base + scale * depth / DEPTH_SCALE
    }

    #[inline]
    pub const fn nmp_reduction(depth: i32) -> i32 {
        (W::nmp_base() + W::nmp_scale() * depth as i64 / DEPTH_SCALE as i64) as i32
    }

    #[inline]
    pub const fn lmp_margin(improving: bool, depth: i32) -> i64 {
        let (base, scale) = if improving {
            (W::lmp_imp_base(), W::lmp_imp_scale())
        } else {
            (W::lmp_base(), W::lmp_scale())
        };

        base + scale * depth as i64 * depth as i64 / (DEPTH_SCALE as i64 * DEPTH_SCALE as i64)
    }

    #[inline]
    pub const fn fp_margin(improving: bool, depth: i32) -> i32 {
        let (base, scale) = if improving {
            (W::fp_imp_base(), W::fp_imp_scale())
        } else {
            (W::fp_base(), W::fp_scale())
        };

        base + scale * depth / DEPTH_SCALE
    }

    #[inline]
    pub const fn hist_margin(depth: i32) -> i32 {
        W::hist_base() + W::hist_scale() * depth / DEPTH_SCALE
    }

    #[inline]
    pub const fn see_quiet_margin(depth: i32) -> i32 {
        W::see_quiet_base() + W::see_quiet_scale() * depth / DEPTH_SCALE
    }

    #[inline]
    pub const fn see_tactic_margin(depth: i32) -> i32 {
        W::see_tactic_base() + W::see_tactic_scale() * depth / DEPTH_SCALE
    }

    #[inline]
    pub fn lmr(is_tactic: bool, depth: u8, moves_seen: u8) -> i32 {
        let (base, div) = if is_tactic {
            (W::lmr_tactic_base(), W::lmr_tactic_div())
        } else {
            (W::lmr_quiet_base(), W::lmr_quiet_div())
        };
        let (base, div) = (base as f32 / 1024.0, div as f32 / 1024.0);

        DEPTH_SCALE * (base + LOG[depth as usize] * LOG[moves_seen as usize] / div) as i32
    }
}
