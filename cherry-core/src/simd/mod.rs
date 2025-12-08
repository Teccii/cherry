#[cfg(target_feature = "avx512f")]
mod avx512;
#[cfg(target_feature = "avx512f")]
pub use avx512::*;
