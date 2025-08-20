#[cfg(feature = "tune")]
use std::cell::SyncUnsafeCell;

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

weights! {
    pawn_corr_frac  | PAWN_CORR_FRAC:  i32 => 64,
    minor_corr_frac | MINOR_CORR_FRAC: i32 => 64,
    major_corr_frac | MAJOR_CORR_FRAC: i32 => 64,

    quiet_bonus_base | QUIET_BONUS_BASE: i32 => 0,
    quiet_bonus_mul  | QUIET_BONUS_MUL:  i32 => 14,
    quiet_malus_base | QUIET_MALUS_BASE: i32 => 0,
    quiet_malus_mul  | QUIET_MALUS_MUL:  i32 => 14,
    quiet_hist_max   | QUIET_HIST_MAX:   i32 => 8192,

    tactic_bonus_base | TACTIC_BONUS_BASE: i32 => 0,
    tactic_bonus_mul  | TACTIC_BONUS_MUL:  i32 => 14,
    tactic_malus_base | TACTIC_MALUS_BASE: i32 => 0,
    tactic_malus_mul  | TACTIC_MALUS_MUL:  i32 => 14,
    tactic_hist_max   | TACTIC_HIST_MAX:   i32 => 8192,

    cont1_bonus_base | CONT1_BONUS_BASE: i32 => 0,
    cont1_bonus_mul  | CONT1_BONUS_MUL:  i32 => 14,
    cont1_malus_base | CONT1_MALUS_BASE: i32 => 0,
    cont1_malus_mul  | CONT1_MALUS_MUL:  i32 => 14,
    cont1_hist_max   | CONT1_HIST_MAX:   i32 => 8192,

    cont2_bonus_base | CONT2_BONUS_BASE: i32 => 0,
    cont2_bonus_mul  | CONT2_BONUS_MUL:  i32 => 14,
    cont2_malus_base | CONT2_MALUS_BASE: i32 => 0,
    cont2_malus_mul  | CONT2_MALUS_MUL:  i32 => 14,
    cont2_hist_max   | CONT2_HIST_MAX:   i32 => 8192,

    cont3_bonus_base | CONT3_BONUS_BASE: i32 => 0,
    cont3_bonus_mul  | CONT3_BONUS_MUL:  i32 => 14,
    cont3_malus_base | CONT3_MALUS_BASE: i32 => 0,
    cont3_malus_mul  | CONT3_MALUS_MUL:  i32 => 14,
    cont3_hist_max   | CONT3_HIST_MAX:   i32 => 8192,

    pawn_see_value   | PAWN_SEE_VALUE:   i16 => 100,
    knight_see_value | KNIGHT_SEE_VALUE: i16 => 320,
    bishop_see_value | BISHOP_SEE_VALUE: i16 => 330,
    rook_see_value   | ROOK_SEE_VALUE:   i16 => 580,
    queen_see_value  | QUEEN_SEE_VALUE:  i16 => 920,

    pawn_mat_scale   | PAWN_MAT_SCALE:   i32 => 100,
    knight_mat_scale | KNIGHT_MAT_SCALE: i32 => 320,
    bishop_mat_scale | BISHOP_MAT_SCALE: i32 => 330,
    rook_mat_scale   | ROOK_MAT_SCALE:   i32 => 580,
    queen_mat_scale  | QUEEN_MAT_SCALE:  i32 => 920,
    mat_scale_base   | MAT_SCALE_BASE:   i32 => 26000,
    mat_scale_div    | MAT_SCALE_DIV:    i32 => 33360,

    rfp_margin        | RFP_MARGIN:        i16 => 93,
    see_quiet_margin  | SEE_QUIET_MARGIN:  i16 => -91,
    see_tactic_margin | SEE_TACTIC_MARGIN: i16 => -64,
    cont_margin       | CONT_MARGIN:       i32 => -3600,
    futile_base       | FUTILE_BASE:       i16 => 106,
    futile_margin     | FUTILE_MARGIN:     i16 => 81,

    tt_pv_reduction         | TT_PV_REDUCTION:         i32 => 926,
    tt_tactic_reduction     | TT_TACTIC_REDUCTION:     i32 => 1024,
    high_corr_reduction     | HIGH_CORR_REDUCTION:     i32 => 1024,
    high_corr_threshold     | HIGH_CORR_THRESHOLD:     i32 => 128,
    hist_tactic_reduction   | HIST_TACTIC_REDUCTION:   i32 => 32,
    hist_quiet_reduction    | HIST_QUIET_REDUCTION:    i32 => 32,
    not_improving_reduction | NOT_IMPROVING_REDUCTION: i32 => 926,
    cut_node_reduction      | CUT_NODE_REDUCTION:      i32 => 1024,
    non_pv_reduction        | NON_PV_REDUCTION:        i32 => 926,
    check_reduction         | CHECK_REDUCTION:         i32 => 1024,
}

pub const RFP_DEPTH: u8 = 12;
pub const NMP_DEPTH: u8 = 5;
pub const SEE_DEPTH: u8 = 10;
pub const HIST_DEPTH: u8 = 10;
pub const FUTILE_DEPTH: u8 = 6;