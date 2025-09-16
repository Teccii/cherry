use std::cell::SyncUnsafeCell;
use cherry_chess::Piece;
use crate::MAX_PLY;


/*----------------------------------------------------------------*/

type LmrLookup = [[i32; MAX_PLY as usize]; MAX_PLY as usize];

pub static LMR_QUIET: SyncUnsafeCell<LmrLookup> = SyncUnsafeCell::new([[0; MAX_PLY as usize]; MAX_PLY as usize]);
pub static LMR_TACTICAL: SyncUnsafeCell<LmrLookup> = SyncUnsafeCell::new([[0; MAX_PLY as usize]; MAX_PLY as usize]);

pub fn get_lmr(is_tactical: bool, depth: u8, moves_seen: u16) -> i32 {
    if is_tactical {
        unsafe { (*LMR_TACTICAL.get())[depth as usize][moves_seen as usize] }
    } else {
        unsafe { (*LMR_QUIET.get())[depth as usize][moves_seen as usize] }
    }
}

pub fn init_lmr() {
    let mut quiet_table = [[0; MAX_PLY as usize]; MAX_PLY as usize];
    let mut tactical_table = [[0; MAX_PLY as usize]; MAX_PLY as usize];

    let (quiet_base, quiet_div) = (W::lmr_quiet_base() as f32 / 1024.0, W::lmr_quiet_div() as f32 / 1024.0);
    let (tactical_base, tactical_div) = (W::lmr_tactical_base() as f32 / 1024.0, W::lmr_tactical_div() as f32 / 1024.0);

    for i in 0..MAX_PLY as usize {
        for j in 0..MAX_PLY as usize {
            let x = if i != 0 { (i as f32).ln() } else { 0.0 };
            let y = if j != 0 { (j as f32).ln() } else { 0.0 };

            quiet_table[i][j] = 1024 * (quiet_base + x * y / quiet_div) as i32;
            tactical_table[i][j] = 1024 * (tactical_base + x * y / tactical_div) as i32;
        }
    }

    unsafe {
        let lmr_quiet: &mut LmrLookup = &mut *LMR_QUIET.get();
        let lmr_tactical: &mut LmrLookup = &mut *LMR_TACTICAL.get();

        *lmr_quiet = quiet_table;
        *lmr_tactical = tactical_table;
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
    pawn_corr_frac  | PAWN_CORR_FRAC:  i32 => 61,
    minor_corr_frac | MINOR_CORR_FRAC: i32 => 70,
    major_corr_frac | MAJOR_CORR_FRAC: i32 => 64,

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
    tactic_malus_max  | TACTIC_MALUS_MAX: i32 => 2012,

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

    pawn_mvv_value   | PAWN_MVV_VALUE:   i32 => 101,
    knight_mvv_value | KNIGHT_MVV_VALUE: i32 => 324,
    bishop_mvv_value | BISHOP_MVV_VALUE: i32 => 332,
    rook_mvv_value   | ROOK_MVV_VALUE:   i32 => 578,
    queen_mvv_value  | QUEEN_MVV_VALUE:  i32 => 981,

    pawn_mat_scale   | PAWN_MAT_SCALE:   i32 => 111,
    knight_mat_scale | KNIGHT_MAT_SCALE: i32 => 349,
    bishop_mat_scale | BISHOP_MAT_SCALE: i32 => 338,
    rook_mat_scale   | ROOK_MAT_SCALE:   i32 => 590,
    queen_mat_scale  | QUEEN_MAT_SCALE:  i32 => 973,
    mat_scale_base   | MAT_SCALE_BASE:   i32 => 25100,

    rfp_margin        | RFP_MARGIN:        i16 => 80,
    see_quiet_margin  | SEE_QUIET_MARGIN:  i16 => -89,
    see_tactic_margin | SEE_TACTIC_MARGIN: i16 => -62,
    cont_margin       | CONT_MARGIN:       i32 => -3268,
    futile_base       | FUTILE_BASE:       i16 => 93,
    futile_margin     | FUTILE_MARGIN:     i16 => 79,

    lmr_quiet_base    | LMR_QUIET_BASE:    i32 => 579,
    lmr_quiet_div     | LMR_QUIET_DIV:     i32 => 1626,
    lmr_tactical_base | LMR_TACTICAL_BASE: i32 => 450,
    lmr_tactical_div  | LMR_TACTICAL_DIV:  i32 => 3688,

    tt_pv_reduction         | TT_PV_REDUCTION:         i32 => 1030,
    tt_tactic_reduction     | TT_TACTIC_REDUCTION:     i32 => 985,
    high_corr_reduction     | HIGH_CORR_REDUCTION:     i32 => 1062,
    high_corr_threshold     | HIGH_CORR_THRESHOLD:     i32 => 130,
    hist_tactic_reduction   | HIST_TACTIC_REDUCTION:   i32 => 34,
    hist_quiet_reduction    | HIST_QUIET_REDUCTION:    i32 => 31,
    not_improving_reduction | NOT_IMPROVING_REDUCTION: i32 => 1075,
    cut_node_reduction      | CUT_NODE_REDUCTION:      i32 => 1112,
    non_pv_reduction        | NON_PV_REDUCTION:        i32 => 965,
    check_reduction         | CHECK_REDUCTION:         i32 => 1034,
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

    #[inline]
    pub const fn mvv_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn =>   W::pawn_mvv_value(),
            Piece::Knight => W::knight_mvv_value(),
            Piece::Bishop => W::bishop_mvv_value(),
            Piece::Rook =>   W::rook_mvv_value(),
            Piece::Queen =>  W::queen_mvv_value(),
            Piece::King =>   20000,
        }
    }
}

pub const RFP_DEPTH: u8 = 12;
pub const NMP_DEPTH: u8 = 5;
pub const SEE_DEPTH: u8 = 10;
pub const HIST_DEPTH: u8 = 10;
pub const FUTILE_DEPTH: u8 = 6;