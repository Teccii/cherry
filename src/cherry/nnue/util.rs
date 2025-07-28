use std::ops::{Deref, DerefMut};
use std::simd::{prelude::*, Simd};
use super::*;

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
pub type I16Reg = Simd<i16, CHUNK_SIZE>;
pub type I32Reg = Simd<i32, CHUNK_SIZE>;

pub fn feed_forward<const L: usize>(
    input: &[i16; L],
    weights: &[i16; L],
    output: &mut i32
) {
    let mut sum = I32Reg::splat(0);
    let zero = I32Reg::splat(0);
    let qa = I32Reg::splat(QA);

    for i in 0..(L / CHUNK_SIZE) {
        let offset = i * CHUNK_SIZE;
        let input: I32Reg = I16Reg::from_slice(&input[offset..]).cast();
        let weight: I32Reg = I16Reg::from_slice(&weights[offset..]).cast();
        let input = input.simd_clamp(zero, qa);

        sum += input * input * weight;
    }

    *output += sum.reduce_sum();
    for i in (L - L % CHUNK_SIZE)..L {
        let input = i32::from(input[i]).clamp(0, QA);
        *output += input * input * i32::from(weights[i]);
    }
}