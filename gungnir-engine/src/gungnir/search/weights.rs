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
    rfp_tt: i16 => RFP_TT,

    nmp_depth: u8 => NMP_DEPTH,
    nmp_verification_depth: u8 => NMP_VERIFY_DEPTH,

    hist_margin: i16 => HIST_MARGIN,
    futile_depth: u8 => FUTILE_DEPTH,
    futile_base: i16 => FUTILE_BASE,
    futile_margin: i16 => FUTILE_MARGIN,
    futile_improving: i16 => FUTILE_IMPROVING,

    singular_depth: u8 => SINGULAR_DEPTH,
    double_ext_base: i16 => DOUBLE_EXT_BASE,
    double_ext_pv: i16 => DOUBLE_EXT_PV,
    triple_ext_base: i16 => TRIPLE_EXT_BASE,
    triple_ext_pv: i16 => TRIPLE_EXT_PV,

    delta_margin: i16 => DELTA_MARGIN,
}

/*----------------------------------------------------------------*/

pub const RAZOR_DEPTH: u8 = 4;
pub const RAZOR_MARGIN: i16 = 337;

pub const RFP_DEPTH: u8 = 12;
pub const RFP_MARGIN: i16 = 93;
pub const RFP_TT: i16 = 20;

pub const NMP_DEPTH: u8 = 4;
pub const NMP_VERIFY_DEPTH: u8 = 12;

pub const HIST_MARGIN: i16 = -4300;
pub const FUTILE_DEPTH: u8 = 12;
pub const FUTILE_BASE: i16 = 47;
pub const FUTILE_MARGIN: i16 = 107;
pub const FUTILE_IMPROVING: i16 = 77;

pub const SINGULAR_DEPTH: u8 = 6;
pub const DOUBLE_EXT_BASE: i16 = -8;
pub const DOUBLE_EXT_PV: i16 = 244;
pub const TRIPLE_EXT_BASE: i16 = 84;
pub const TRIPLE_EXT_PV: i16 = 269;

pub const DELTA_MARGIN: i16 = 211;
