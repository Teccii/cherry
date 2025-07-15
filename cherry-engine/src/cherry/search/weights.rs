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
    razor_depth: u8 => RAZOR_DEPTH,
    razor_margin: i16 => RAZOR_MARGIN,

    rfp_depth: u8 => RFP_DEPTH,
    rfp_margin: i16 => RFP_MARGIN,
    rfp_hist: i16 => RFP_HIST,
    rfp_tt: i16 => RFP_TT,

    nmp_depth: u8 => NMP_DEPTH,
    nmp_verification_depth: u8 => NMP_VERIFY_DEPTH,

    iir_depth: u8 => IIR_DEPTH,
    
    see_depth: u8 => SEE_DEPTH,
    see_margin: i16 => SEE_MARGIN,
    see_hist: i16 => SEE_HIST,

    hist_depth: u8 => HIST_DEPTH,
    hist_margin: i16 => HIST_MARGIN,

    futile_depth: u8 => FUTILE_DEPTH,
    futile_base: i16 => FUTILE_BASE,
    futile_margin: i16 => FUTILE_MARGIN,
    futile_improving: i16 => FUTILE_IMPROVING,

    singular_depth: u8 => SINGULAR_DEPTH,
    double_base: i16 => DOUBLE_BASE,
    double_pv: i16 => DOUBLE_PV,
    triple_base: i16 => TRIPLE_BASE,
    triple_pv: i16 => TRIPLE_PV,

    base_reduction: i32 => BASE_REDUCTION,
    non_pv_reduction: i32 => NON_PV_REDUCTION,
    not_improving_reduction: i32 => NOT_IMPROVING_REDUCTION,
    cut_node_reduction: i32 => CUT_NODE_REDUCTION,
    hist_reduction: i32 => HIST_REDUCTION,

    delta_margin: i16 => DELTA_MARGIN,
}

/*----------------------------------------------------------------*/

pub const RAZOR_DEPTH: u8 = 4;
pub const RAZOR_MARGIN: i16 = 337;

pub const RFP_DEPTH: u8 = 12;
pub const RFP_MARGIN: i16 = 93;
pub const RFP_HIST: i16 = 376;
pub const RFP_TT: i16 = 20;

pub const NMP_DEPTH: u8 = 4;
pub const NMP_VERIFY_DEPTH: u8 = 12;

pub const IIR_DEPTH: u8 = 6;

pub const SEE_DEPTH: u8 = 10;
pub const SEE_MARGIN: i16 = -91;
pub const SEE_HIST: i16 = 61;

pub const HIST_DEPTH: u8 = 10;
pub const HIST_MARGIN: i16 = -4300;

pub const FUTILE_DEPTH: u8 = 12;
pub const FUTILE_BASE: i16 = 47;
pub const FUTILE_MARGIN: i16 = 107;
pub const FUTILE_IMPROVING: i16 = 77;

pub const SINGULAR_DEPTH: u8 = 6;
pub const DOUBLE_BASE: i16 = -8;
pub const DOUBLE_PV: i16 = 244;
pub const TRIPLE_BASE: i16 = 84;
pub const TRIPLE_PV: i16 = 269;

pub const BASE_REDUCTION: i32 = 136;
pub const NON_PV_REDUCTION: i32 = 926;
pub const NOT_IMPROVING_REDUCTION: i32 = 926;
pub const CUT_NODE_REDUCTION: i32 = 2113;
pub const HIST_REDUCTION: i32 = 155;

pub const DELTA_MARGIN: i16 = 211;
