#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
mod avx2;
#[cfg(target_feature = "avx512f")]
mod avx512;
mod common;

#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
pub use avx2::*;
#[cfg(target_feature = "avx512f")]
pub use avx512::*;
pub use common::*;
