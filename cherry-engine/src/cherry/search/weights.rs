macro_rules! weights {
    ($($elem:ident: $ty:ty => $default:expr,)*) => {
        #[derive(Debug, Clone)]
        pub struct SearchWeights {
            $(pub $elem: $ty,)*
        }

        impl Default for SearchWeights {
            fn default() -> Self {
                SearchWeights { $($elem: $default,)* }
            }
        }
    }
}

weights! {
    pawn_corr_frac: i16 => PAWN_CORR_FRAC,
    cont1_frac: i32 => CONT1_FRAC,
    cont2_frac: i32 => CONT2_FRAC,
    cont3_frac: i32 => CONT3_FRAC,

    quiet_bonus_base: i32 => QUIET_BONUS_BASE,
    quiet_bonus_mul: i32 => QUIET_BONUS_MUL,
    quiet_malus_base: i32 => QUIET_MALUS_BASE,
    quiet_malus_mul: i32 => QUIET_MALUS_MUL,

    capt_bonus_base: i32 => CAPT_BONUS_BASE,
    capt_bonus_mul: i32 => CAPT_BONUS_MUL,
    capt_malus_base: i32 => CAPT_MALUS_BASE,
    capt_malus_mul: i32 => CAPT_MALUS_MUL,

    cont1_bonus_base: i32 => CONT1_BONUS_BASE,
    cont1_bonus_mul: i32 => CONT1_BONUS_MUL,
    cont1_malus_base: i32 => CONT1_MALUS_BASE,
    cont1_malus_mul: i32 => CONT1_MALUS_MUL,

    cont2_bonus_base: i32 => CONT2_BONUS_BASE,
    cont2_bonus_mul: i32 => CONT2_BONUS_MUL,
    cont2_malus_base: i32 => CONT2_MALUS_BASE,
    cont2_malus_mul: i32 => CONT2_MALUS_MUL,

    cont3_bonus_base: i32 => CONT3_BONUS_BASE,
    cont3_bonus_mul: i32 => CONT3_BONUS_MUL,
    cont3_malus_base: i32 => CONT3_MALUS_BASE,
    cont3_malus_mul: i32 => CONT3_MALUS_MUL,

    rfp_depth: u8 => RFP_DEPTH,
    rfp_margin: i16 => RFP_MARGIN,

    nmp_depth: u8 => NMP_DEPTH,
    
    see_depth: u8 => SEE_DEPTH,
    see_margin: i16 => SEE_MARGIN,
    see_hist: i32 => SEE_HIST,

    hist_depth: u8 => HIST_DEPTH,
    hist_margin: i32 => HIST_MARGIN,

    futile_depth: u8 => FUTILE_DEPTH,
    futile_base: i16 => FUTILE_BASE,
    futile_margin: i16 => FUTILE_MARGIN,

    non_pv_reduction: i32 => NON_PV_REDUCTION,
    not_improving_reduction: i32 => NOT_IMPROVING_REDUCTION,
    cut_node_reduction: i32 => CUT_NODE_REDUCTION,
    hist_reduction: i32 => HIST_REDUCTION,
}

/*----------------------------------------------------------------*/

pub const PAWN_CORR_FRAC: i16 = 66;
pub const CONT1_FRAC: i32 = 512;
pub const CONT2_FRAC: i32 = 512;
pub const CONT3_FRAC: i32 = 512;

pub const QUIET_BONUS_BASE: i32 = 0;
pub const QUIET_BONUS_MUL: i32 = 14;
pub const QUIET_MALUS_BASE: i32 = 0;
pub const QUIET_MALUS_MUL: i32 = 14;

pub const CAPT_BONUS_BASE: i32 = 0;
pub const CAPT_BONUS_MUL: i32 = 14;
pub const CAPT_MALUS_BASE: i32 = 0;
pub const CAPT_MALUS_MUL: i32 = 14;

pub const CONT1_BONUS_BASE: i32 = 0;
pub const CONT1_BONUS_MUL: i32 = 14;
pub const CONT1_MALUS_BASE: i32 = 0;
pub const CONT1_MALUS_MUL: i32 = 14;

pub const CONT2_BONUS_BASE: i32 = 0;
pub const CONT2_BONUS_MUL: i32 = 14;
pub const CONT2_MALUS_BASE: i32 = 0;
pub const CONT2_MALUS_MUL: i32 = 14;

pub const CONT3_BONUS_BASE: i32 = 0;
pub const CONT3_BONUS_MUL: i32 = 14;
pub const CONT3_MALUS_BASE: i32 = 0;
pub const CONT3_MALUS_MUL: i32 = 14;

pub const RFP_DEPTH: u8 = 12;
pub const RFP_MARGIN: i16 = 93;

pub const NMP_DEPTH: u8 = 5;

pub const SEE_DEPTH: u8 = 10;
pub const SEE_MARGIN: i16 = -91;
pub const SEE_HIST: i32 = 61;

pub const HIST_DEPTH: u8 = 10;
pub const HIST_MARGIN: i32 = -4300;

pub const FUTILE_DEPTH: u8 = 6;
pub const FUTILE_BASE: i16 = 106;
pub const FUTILE_MARGIN: i16 = 81;

pub const NON_PV_REDUCTION: i32 = 926;
pub const NOT_IMPROVING_REDUCTION: i32 = 926;
pub const CUT_NODE_REDUCTION: i32 = 2113;
pub const HIST_REDUCTION: i32 = 131;