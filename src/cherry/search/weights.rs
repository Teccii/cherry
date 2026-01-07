use core::cell::SyncUnsafeCell;

use crate::*;

/*----------------------------------------------------------------*/

type LmrLookup = [[i32; MAX_PLY as usize]; MAX_PLY as usize];

pub static LMR_QUIET: SyncUnsafeCell<LmrLookup> =
    SyncUnsafeCell::new([[0; MAX_PLY as usize]; MAX_PLY as usize]);
pub static LMR_QUIET_IMP: SyncUnsafeCell<LmrLookup> =
    SyncUnsafeCell::new([[0; MAX_PLY as usize]; MAX_PLY as usize]);
pub static LMR_TACTIC: SyncUnsafeCell<LmrLookup> =
    SyncUnsafeCell::new([[0; MAX_PLY as usize]; MAX_PLY as usize]);
pub static LMR_TACTIC_IMP: SyncUnsafeCell<LmrLookup> =
    SyncUnsafeCell::new([[0; MAX_PLY as usize]; MAX_PLY as usize]);

#[inline]
pub fn get_lmr(is_tactic: bool, improving: bool, depth: u8, moves_seen: u16) -> i32 {
    let table = match (is_tactic, improving) {
        (true, true) => &LMR_TACTIC_IMP,
        (true, false) => &LMR_TACTIC,
        (false, true) => &LMR_QUIET_IMP,
        (false, false) => &LMR_QUIET,
    };

    unsafe { (*table.get())[depth as usize][moves_seen as usize] }
}

pub fn init_lmr() {
    let mut quiet_table: Box<LmrLookup> = new_zeroed();
    let mut quiet_imp_table: Box<LmrLookup> = new_zeroed();
    let mut tactic_table: Box<LmrLookup> = new_zeroed();
    let mut tactic_imp_table: Box<LmrLookup> = new_zeroed();

    let (quiet_base, quiet_div) = (
        W::lmr_quiet_base() as f32 / 1024.0,
        W::lmr_quiet_div() as f32 / 1024.0,
    );
    let (quiet_imp_base, quiet_imp_div) = (
        W::lmr_quiet_imp_base() as f32 / 1024.0,
        W::lmr_quiet_imp_div() as f32 / 1024.0,
    );
    let (tactic_base, tactic_div) = (
        W::lmr_tactic_base() as f32 / 1024.0,
        W::lmr_tactic_div() as f32 / 1024.0,
    );
    let (tactic_imp_base, tactic_imp_div) = (
        W::lmr_tactic_imp_base() as f32 / 1024.0,
        W::lmr_tactic_imp_div() as f32 / 1024.0,
    );

    for i in 0..MAX_PLY as usize {
        for j in 0..MAX_PLY as usize {
            let x = if i != 0 { (i as f32).ln() } else { 0.0 };
            let y = if j != 0 { (j as f32).ln() } else { 0.0 };

            quiet_table[i][j] = DEPTH_SCALE * (quiet_base + x * y / quiet_div) as i32;
            quiet_imp_table[i][j] = DEPTH_SCALE * (quiet_imp_base + x * y / quiet_imp_div) as i32;
            tactic_table[i][j] = DEPTH_SCALE * (tactic_base + x * y / tactic_div) as i32;
            tactic_imp_table[i][j] =
                DEPTH_SCALE * (tactic_imp_base + x * y / tactic_imp_div) as i32;
        }
    }

    unsafe {
        let lmr_quiet: &mut LmrLookup = &mut *LMR_QUIET.get();
        let lmr_quiet_imp: &mut LmrLookup = &mut *LMR_QUIET_IMP.get();
        let lmr_tactic: &mut LmrLookup = &mut *LMR_TACTIC.get();
        let lmr_tactic_imp: &mut LmrLookup = &mut *LMR_TACTIC_IMP.get();

        lmr_quiet.copy_from_slice(&*quiet_table);
        lmr_quiet_imp.copy_from_slice(&*quiet_imp_table);
        lmr_tactic.copy_from_slice(&*tactic_table);
        lmr_tactic_imp.copy_from_slice(&*tactic_imp_table);
    }
}

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
    white_corr_frac  | WHITE_CORR_FRAC:  i32 => 64,
    black_corr_frac  | BLACK_CORR_FRAC:  i32 => 64,
    corr_bonus_scale | CORR_BONUS_SCALE: i64 => 128,

    quiet_bonus_base  | QUIET_BONUS_BASE:  i32 => 130,
    quiet_bonus_scale | QUIET_BONUS_SCALE: i32 => 127,
    quiet_bonus_max   | QUIET_BONUS_MAX:   i32 => 2073,
    quiet_malus_base  | QUIET_MALUS_BASE:  i32 => 122,
    quiet_malus_scale | QUIET_MALUS_SCALE: i32 => 129,
    quiet_malus_max   | QUIET_MALUS_MAX:   i32 => 1903,

    tactic_bonus_base  | TACTIC_BONUS_BASE:  i32 => 131,
    tactic_bonus_scale | TACTIC_BONUS_SCALE: i32 => 133,
    tactic_bonus_max   | TACTIC_BONUS_MAX:   i32 => 1967,
    tactic_malus_base  | TACTIC_MALUS_BASE:  i32 => 128,
    tactic_malus_scale | TACTIC_MALUS_SCALE: i32 => 129,
    tactic_malus_max   | TACTIC_MALUS_MAX:   i32 => 2012,

    cont1_bonus_base  | CONT1_BONUS_BASE:  i32 => 133,
    cont1_bonus_scale | CONT1_BONUS_SCALE: i32 => 137,
    cont1_bonus_max   | CONT1_BONUS_MAX:   i32 => 1911,
    cont1_malus_base  | CONT1_MALUS_BASE:  i32 => 125,
    cont1_malus_scale | CONT1_MALUS_SCALE: i32 => 129,
    cont1_malus_max   | CONT1_MALUS_MAX:   i32 => 2204,

    cont2_bonus_base  | CONT2_BONUS_BASE:  i32 => 125,
    cont2_bonus_scale | CONT2_BONUS_SCALE: i32 => 121,
    cont2_bonus_max   | CONT2_BONUS_MAX:   i32 => 2073,
    cont2_malus_base  | CONT2_MALUS_BASE:  i32 => 122,
    cont2_malus_scale | CONT2_MALUS_SCALE: i32 => 134,
    cont2_malus_max   | CONT2_MALUS_MAX:   i32 => 2166,

    pawn_see_value   | PAWN_SEE_VALUE:   i16 => 101,
    knight_see_value | KNIGHT_SEE_VALUE: i16 => 324,
    bishop_see_value | BISHOP_SEE_VALUE: i16 => 332,
    rook_see_value   | ROOK_SEE_VALUE:   i16 => 578,
    queen_see_value  | QUEEN_SEE_VALUE:  i16 => 981,

    pawn_mat_scale   | PAWN_MAT_SCALE:   i32 => 111,
    knight_mat_scale | KNIGHT_MAT_SCALE: i32 => 349,
    bishop_mat_scale | BISHOP_MAT_SCALE: i32 => 338,
    rook_mat_scale   | ROOK_MAT_SCALE:   i32 => 590,
    queen_mat_scale  | QUEEN_MAT_SCALE:  i32 => 973,
    mat_scale_base   | MAT_SCALE_BASE:   i32 => 25100,

    rfp_depth     | RFP_DEPTH:     i32 => 6144,
    rfp_base      | RFP_BASE:      i32 => 0,
    rfp_scale     | RFP_SCALE:     i32 => 80,
    rfp_lerp      | RFP_LERP:      i32 => 512,
    rfp_imp_depth | RFP_IMP_DEPTH: i32 => 6144,
    rfp_imp_base  | RFP_IMP_BASE:  i32 => -80,
    rfp_imp_scale | RFP_IMP_SCALE: i32 => 80,
    rfp_imp_lerp  | RFP_IMP_LERP:  i32 => 512,

    nmp_depth     | NMP_DEPTH:     i32 => 3072,
    nmp_base      | NMP_BASE:      i64 => 3072,
    nmp_scale     | NMP_SCALE:     i64 => 340,
    nmp_imp_depth | NMP_IMP_DEPTH: i32 => 3072,
    nmp_imp_base  | NMP_IMP_BASE:  i64 => 3072,
    nmp_imp_scale | NMP_IMP_SCALE: i64 => 340,

    lmp_base      | LMP_BASE:      i64 => 2048,
    lmp_scale     | LMP_SCALE:     i64 => 512,
    lmp_imp_base  | LMP_IMP_BASE:  i64 => 4096,
    lmp_imp_scale | LMP_IMP_SCALE: i64 => 1024,

    fp_depth     | FP_DEPTH:     i32 => 8192,
    fp_base      | FP_BASE:      i32 => 93,
    fp_scale     | FP_SCALE:     i32 => 79,
    fp_imp_depth | FP_IMP_DEPTH: i32 => 8192,
    fp_imp_base  | FP_IMP_BASE:  i32 => 93,
    fp_imp_scale | FP_IMP_SCALE: i32 => 79,

    see_quiet_depth      | SEE_QUIET_DEPTH:      i32 => 10240,
    see_quiet_base       | SEE_QUIET_BASE:       i32 => 0,
    see_quiet_scale      | SEE_QUIET_SCALE:      i32 => -89,
    see_quiet_imp_depth  | SEE_QUIET_IMP_DEPTH:  i32 => 10240,
    see_quiet_imp_base   | SEE_QUIET_IMP_BASE:   i32 => 0,
    see_quiet_imp_scale  | SEE_QUIET_IMP_SCALE:  i32 => -89,

    see_tactic_depth     | SEE_TACTIC_DEPTH:     i32 => 10240,
    see_tactic_base      | SEE_TACTIC_BASE:      i32 => 0,
    see_tactic_scale     | SEE_TACTIC_SCALE:     i32 => -62,
    see_tactic_imp_depth | SEE_TACTIC_IMP_DEPTH: i32 => 10240,
    see_tactic_imp_base  | SEE_TACTIC_IMP_BASE:  i32 => 0,
    see_tactic_imp_scale | SEE_TACTIC_IMP_SCALE: i32 => -62,

    singular_depth        | SINGULAR_DEPTH:             i32 => 6144,
    singular_tt_depth     | SINGULAR_TT_DEPTH:          i32 => 3072,
    singular_beta_margin  | SINGULAR_BETA_MARGIN:       i32 => 196,
    singular_search_depth | SINGULAR_SEARCH_DEPTH:      i32 => 512,
    singular_dext_margin  | SINGULAR_DEXT_MARGIN:       i16 => 30,
    singular_ext          | SINGULAR_EXT:               i32 => 1024,
    singular_dext         | SINGULAR_DEXT:              i32 => 2048,
    singular_tt_ext       | SINGULAR_TT_EXT:           i32 => -1024,

    lmr_quiet_base      | LMR_QUIET_BASE:      i32 => 579,
    lmr_quiet_imp_base  | LMR_QUIET_IMP_BASE:  i32 => 579,
    lmr_quiet_div       | LMR_QUIET_DIV:       i32 => 1626,
    lmr_quiet_imp_div   | LMR_QUIET_IMP_DIV:   i32 => 1626,
    lmr_tactic_base     | LMR_TACTIC_BASE:     i32 => 450,
    lmr_tactic_imp_base | LMR_TACTIC_IMP_BASE: i32 => 450,
    lmr_tactic_div      | LMR_TACTIC_DIV:      i32 => 3688,
    lmr_tactic_imp_div  | LMR_TACTIC_IMP_DIV:  i32 => 3688,

    cutnode_lmr   | CUTNODE_LMR: i32 => 1024,
    improving_lmr | IMPROVING_LMR: i32 => 1024,

    lmr_depth_bias    | LMR_DEPTH_BIAS:    i32 => 0,
    lmr_depth_pv_bias | LMR_DEPTH_PV_BIAS: i32 => 0,
    tt_depth_bias     | TT_DEPTH_BIAS:     i32 => 0,
    tt_depth_pv_bias  | TT_DEPTH_PV_BIAS:  i32 => 0,

    asp_window_initial | ASP_WINDOW_INITIAL: i16 => 20,
    asp_window_expand  | ASP_WINDOW_EXPAND:  i32 => 48,

    soft_time_frac      | SOFT_TIME_FRAC:      u64 => 64,
    hard_time_frac      | HARD_TIME_FRAC:      u64 => 2458,
    subtree_tm_base     | SUBTREE_TM_BASE:     u64 => 10240,
    subtree_tm_scale    | SUBTREE_TM_SCALE:    u64 => 6144,
    subtree_tm_min      | SUBTREE_TM_MIN:      u64 => 4096,
    stability_tm_base   | STABILITY_TM_BASE:   u64 => 7373,
    stability_tm_scale  | STABILITY_TM_SCALE:  u64 => 410,
    stability_tm_min    | STABILITY_TM_MIN:    u64 => 4096,
    complexity_tm_base  | COMPLEXITY_TM_BASE:  u64 => 3277,
    complexity_tm_scale | COMPLEXITY_TM_SCALE: u64 => 82,
    complexity_tm_max   | COMPLEXITY_TM_MAX:   u64 => 6144,
    complexity_tm_win   | COMPLEXITY_TM_WIN:   u64 => 4096,
    complexity_tm_loss  | COMPLEXITY_TM_LOSS:  u64 => 4096,
}

impl W {
    #[inline]
    pub const fn see_value(piece: Piece) -> i16 {
        match piece {
            Piece::Pawn => W::pawn_see_value(),
            Piece::Knight => W::knight_see_value(),
            Piece::Bishop => W::bishop_see_value(),
            Piece::Rook => W::rook_see_value(),
            Piece::Queen => W::queen_see_value(),
            Piece::King => 20000,
        }
    }
}
