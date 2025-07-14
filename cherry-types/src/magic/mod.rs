mod common;
#[cfg(not(target_feature = "bmi2"))] mod normal;
#[cfg(target_feature = "bmi2")] mod pext;

pub use common::*;
#[cfg(not(target_feature = "bmi2"))] pub use normal::*;
#[cfg(target_feature = "bmi2")] pub use pext::*;