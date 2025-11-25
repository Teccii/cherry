use core::cell::SyncUnsafeCell;
use crate::*;

/*----------------------------------------------------------------*/

type LmrLookup = [[[i32; MAX_PLY as usize]; MAX_PLY as usize]; 2];

pub static LMR_QUIET: SyncUnsafeCell<LmrLookup> = SyncUnsafeCell::new([[[0; MAX_PLY as usize]; MAX_PLY as usize]; 2]);
pub static LMR_TACTIC: SyncUnsafeCell<LmrLookup> = SyncUnsafeCell::new([[[0; MAX_PLY as usize]; MAX_PLY as usize]; 2]);

#[inline]
pub fn get_lmr(is_tactic: bool, improving: bool, depth: u8, moves_seen: u16) -> i32 {
    if is_tactic {
        unsafe { (*LMR_TACTIC.get())[improving as usize][depth as usize][moves_seen as usize] }
    } else {
        unsafe { (*LMR_QUIET.get())[improving as usize][depth as usize][moves_seen as usize] }
    }
}

pub fn init_lmr() {
    let mut quiet_table: Box<LmrLookup> = new_zeroed();
    let mut tactic_table: Box<LmrLookup> = new_zeroed();

    let (quiet_base, quiet_div) = (W::lmr_quiet_base()[0] as f32 / 1024.0, W::lmr_quiet_div()[0] as f32 / 1024.0);
    let (quiet_improving_base, quiet_improving_div) = (W::lmr_quiet_base()[1] as f32 / 1024.0, W::lmr_quiet_div()[1] as f32 / 1024.0);
    let (tactic_base, tactic_div) = (W::lmr_tactic_base()[0] as f32 / 1024.0, W::lmr_tactic_div()[0] as f32 / 1024.0);
    let (tactic_improving_base, tactic_improving_div) = (W::lmr_tactic_base()[1] as f32 / 1024.0, W::lmr_tactic_div()[1] as f32 / 1024.0);

    for i in 0..MAX_PLY as usize {
        for j in 0..MAX_PLY as usize {
            let x = if i != 0 { (i as f32).ln() } else { 0.0 };
            let y = if j != 0 { (j as f32).ln() } else { 0.0 };

            quiet_table[0][i][j] = DEPTH_SCALE * (quiet_base + x * y / quiet_div) as i32;
            quiet_table[1][i][j] = DEPTH_SCALE * (quiet_improving_base + x * y / quiet_improving_div) as i32;
            tactic_table[0][i][j] = DEPTH_SCALE * (tactic_base + x * y / tactic_div) as i32;
            tactic_table[1][i][j] = DEPTH_SCALE * (tactic_improving_base + x * y / tactic_improving_div) as i32;
        }
    }

    unsafe {
        let lmr_quiet: &mut LmrLookup = &mut *LMR_QUIET.get();
        let lmr_tactic: &mut LmrLookup = &mut *LMR_TACTIC.get();

        lmr_quiet.copy_from_slice(&*quiet_table);
        lmr_tactic.copy_from_slice(&*tactic_table);
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
    pawn_corr_frac  | PAWN_CORR_FRAC:  i32 => 64,
    minor_corr_frac | MINOR_CORR_FRAC: i32 => 64,
    major_corr_frac | MAJOR_CORR_FRAC: i32 => 64,
    white_corr_frac | WHITE_CORR_FRAC: i32 => 64,
    black_corr_frac | BLACK_CORR_FRAC: i32 => 64,

    quiet_bonus_base | QUIET_BONUS_BASE: i32 => 130,
    quiet_bonus_mul  | QUIET_BONUS_MUL:  i32 => 127,
    quiet_bonus_max  | QUIET_BONUS_MAX:  i32 => 2073,
    quiet_malus_base | QUIET_MALUS_BASE: i32 => 122,
    quiet_malus_mul  | QUIET_MALUS_MUL:  i32 => 129,
    quiet_malus_max  | QUIET_MALUS_MAX:  i32 => 1903,

    tactic_bonus_base | TACTIC_BONUS_BASE: i32 => 131,
    tactic_bonus_mul  | TACTIC_BONUS_MUL:  i32 => 133,
    tactic_bonus_max  | TACTIC_BONUS_MAX:  i32 => 1967,
    tactic_malus_base | TACTIC_MALUS_BASE: i32 => 128,
    tactic_malus_mul  | TACTIC_MALUS_MUL:  i32 => 129,
    tactic_malus_max  | TACTIC_MALUS_MAX:  i32 => 2012,

    cont1_bonus_base | CONT1_BONUS_BASE: i32 => 133,
    cont1_bonus_mul  | CONT1_BONUS_MUL:  i32 => 137,
    cont1_bonus_max  | CONT1_BONUS_MAX:  i32 => 1911,
    cont1_malus_base | CONT1_MALUS_BASE: i32 => 125,
    cont1_malus_mul  | CONT1_MALUS_MUL:  i32 => 129,
    cont1_malus_max  | CONT1_MALUS_MAX:  i32 => 2204,

    cont2_bonus_base | CONT2_BONUS_BASE: i32 => 125,
    cont2_bonus_mul  | CONT2_BONUS_MUL:  i32 => 121,
    cont2_bonus_max  | CONT2_BONUS_MAX:  i32 => 2073,
    cont2_malus_base | CONT2_MALUS_BASE: i32 => 122,
    cont2_malus_mul  | CONT2_MALUS_MUL:  i32 => 134,
    cont2_malus_max  | CONT2_MALUS_MAX:  i32 => 2166,

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

    rfp_depth | RFP_DEPTH: [i32; 2] => [6144, 6144],
    rfp_base  | RFP_BASE:  [i32; 2] => [0, -80],
    rfp_scale | RFP_SCALE: [i32; 2] => [80, 80],
    rfp_lerp  | RFP_LERP:  [i32; 2] => [512, 512],

    nmp_depth | NMP_DEPTH: [i32; 2] => [3072, 3072],
    nmp_base  | NMP_BASE:  [i64; 2] => [3072, 3072],
    nmp_scale | NMP_SCALE: [i64; 2] => [340, 340],

    lmp_base  | LMP_BASE:  [i64; 2] => [2048, 4096],
    lmp_scale | LMP_SCALE: [i64; 2] => [512, 1024],

    futile_quiet_depth | FUTILE_QUIET_DEPTH: [i32; 2] => [8192, 8192],
    futile_quiet_base  | FUTILE_QUIET_BASE:  [i32; 2] => [93, 93],
    futile_quiet_scale | FUTILE_QUIET_SCALE: [i32; 2] => [79, 79],
    futile_tactic_depth      | FUTILE_TACTIC_DEPTH:      [i32; 2] => [6144, 6144],
    futile_tactic_base       | FUTILE_TACTIC_BASE:       [i32; 2] => [0, 0],
    futile_tactic_scale      | FUTILE_TACTIC_SCALE:      [i32; 2] => [120, 120],
    futile_tactic_move_scale | FUTILE_TACTIC_MOVE_SCALE: [i32; 2] => [186, 186],

    see_quiet_depth | SEE_QUIET_DEPTH: [i32; 2] => [10240, 10240],
    see_quiet_base  | SEE_QUIET_BASE:  [i32; 2] => [0, 0],
    see_quiet_scale | SEE_QUIET_SCALE: [i32; 2] => [-89, -89],

    see_tactic_depth | SEE_TACTIC_DEPTH: [i32; 2] => [10240, 10240],
    see_tactic_base  | SEE_TACTIC_BASE:  [i32; 2] => [0, 0],
    see_tactic_scale | SEE_TACTIC_SCALE: [i32; 2] => [-62, -62],

    singular_depth        | SINGULAR_DEPTH:        [i32; 2] => [6144, 6144],
    singular_tt_depth     | SINGULAR_TT_DEPTH:     [i32; 2] => [3072, 3072],
    singular_beta_margin  | SINGULAR_BETA_MARGIN:  [i32; 2] => [196, 196],
    singular_search_depth | SINGULAR_SEARCH_DEPTH: [i32; 2] => [512, 512],
    singular_dext_margin  | SINGULAR_DEXT_MARGIN:  [i16; 2] => [30, 30],
    singular_ext          | SINGULAR_EXT:          [i32; 2] => [1024, 1024],
    singular_dext         | SINGULAR_DEXT:         [i32; 2] => [1024, 1024],
    singular_neg_ext      | SINGULAR_NEG_EXT:      [i32; 2] => [-1024, -1024],

    tt_depth_bias | TT_DEPTH_BIAS: [i32; 2] => [0, 0],

    lmr_quiet_base  | LMR_QUIET_BASE:  [i32; 2] => [579, 579],
    lmr_quiet_div   | LMR_QUIET_DIV:   [i32; 2] => [1626, 1626],
    lmr_tactic_base | LMR_TACTIC_BASE: [i32; 2] => [450, 450],
    lmr_tactic_div  | LMR_TACTIC_DIV:  [i32; 2] => [3688, 3688],

    asp_window_initial | ASP_WINDOW_INITIAL: i16 => 20,
    asp_window_expand  | ASP_WINDOW_EXPAND:  i16 => 48,

    soft_time_frac      | SOFT_TIME_FRAC:      u64 => 256,
    hard_time_frac      | HARD_TIME_FRAC:      u64 => 9830,
    subtree_tm_base     | SUBTREE_TM_BASE:     f32 => 2.5,
    subtree_tm_scale    | SUBTREE_TM_SCALE:    f32 => 1.5,
    stability_tm_base   | STABILITY_TM_BASE:   f32 => 1.8,
    stability_tm_scale  | STABILITY_TM_SCALE:  f32 => 0.1,
    complexity_tm_base  | COMPLEXITY_TM_BASE:  f32 => 0.8,
    complexity_tm_scale | COMPLEXITY_TM_SCALE: f32 => 0.8,
    complexity_tm_max   | COMPLEXITY_TM_MAX:   f32 => 200.0,
    complexity_tm_div   | COMPLEXITY_TM_DIV:   f32 => 400.0,
}

impl W {
    #[inline]
    pub const fn see_value(piece: Piece) -> i16 {
        match piece {
            Piece::Pawn =>   W::pawn_see_value(),
            Piece::Knight => W::knight_see_value(),
            Piece::Bishop => W::bishop_see_value(),
            Piece::Rook =>   W::rook_see_value(),
            Piece::Queen =>  W::queen_see_value(),
            Piece::King =>   20000,
        }
    }
}

pub const HIST_DEPTH: u8 = 10;