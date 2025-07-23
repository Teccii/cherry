use std::ops::{Deref, DerefMut};
use std::simd::Simd;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, align(64))]
pub struct Align64<T>(pub T);

impl<T> Deref for Align64<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Align64<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/*----------------------------------------------------------------*/

#[cfg(target_feature = "avx512f")] pub const CHUNK_SIZE: usize = 64;
#[cfg(all(
    target_feature = "avx2",
    not(target_feature = "avx512f"))
)] pub const CHUNK_SIZE: usize = 32;
#[cfg(all(
    not(target_feature = "avx2"),
    not(target_feature = "avx512f"))
)] pub const CHUNK_SIZE: usize = 16;

pub type U8Reg = Simd<u8, CHUNK_SIZE>;
pub type I16Reg = Simd<i16, CHUNK_SIZE>;