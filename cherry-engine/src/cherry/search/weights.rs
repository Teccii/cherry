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
    rfp_depth: u8 => RFP_DEPTH,
    rfp_margin: i16 => RFP_MARGIN,

    nmp_depth: u8 => NMP_DEPTH,

    iir_depth: u8 => IIR_DEPTH,
    corr_frac: i16 => CORR_FRAC,
    
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

pub const RFP_DEPTH: u8 = 12;
pub const RFP_MARGIN: i16 = 93;

pub const NMP_DEPTH: u8 = 5;

pub const IIR_DEPTH: u8 = 6;
pub const CORR_FRAC: i16 = 66;

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