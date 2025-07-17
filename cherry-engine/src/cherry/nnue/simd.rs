use std::simd::Simd;

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