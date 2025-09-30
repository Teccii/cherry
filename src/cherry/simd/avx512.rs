use std::{arch::x86_64::*, ops::*};

/*----------------------------------------------------------------*/

pub type Vec128Mask8 = __mmask16;
pub type Vec128Mask16 = __mmask8;
pub type Vec128Mask32 = __mmask8;
pub type Vec128Mask64 = __mmask8;

pub type Vec256Mask8 = __mmask32;
pub type Vec256Mask16 = __mmask16;
pub type Vec256Mask32 = __mmask8;
pub type Vec256Mask64 = __mmask8;

pub type Vec512Mask8 = __mmask64;
pub type Vec512Mask16 = __mmask32;
pub type Vec512Mask32 = __mmask16;
pub type Vec512Mask64 = __mmask8;

pub type NativeVec = Vec512;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Vec128 {
    raw: __m128i
}

impl Vec128 {
    #[inline]
    pub fn load<T>(src: *const T) -> Vec128 {
        unsafe { _mm_loadu_si128(src.cast()).into() }
    }

    #[inline]
    pub fn store<T>(dst: *mut T, src: Vec128) {
        unsafe { _mm_storeu_si128(dst.cast(), src.raw); }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn into_u32(self) -> u32 {
        unsafe { _mm_cvtsi128_si32(self.raw) as u32 }
    }

    #[inline]
    pub fn into_u64(self) -> u64 {
        unsafe { _mm_cvtsi128_si64(self.raw) as u64 }
    }

    /*----------------------------------------------------------------*/
    
    #[inline]
    pub fn zero() -> Vec128 {
        unsafe { _mm_setzero_si128().into() }
    }

    #[inline]
    pub fn splat8(value: u8) -> Vec128 {
        unsafe { _mm_set1_epi8(value as i8).into() }
    }

    #[inline]
    pub fn splat16(value: u16) -> Vec128 {
        unsafe { _mm_set1_epi16(value as i16).into() }
    }

    #[inline]
    pub fn splat32(value: u32) -> Vec128 {
        unsafe { _mm_set1_epi32(value as i32).into() }
    }

    #[inline]
    pub fn splat64(value: u64) -> Vec128 {
        unsafe { _mm_set1_epi64x(value as i64).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask8(mask: Vec128Mask8, vec: Vec128) -> Vec128 {
        unsafe { _mm_maskz_mov_epi8(mask, vec.raw).into() }
    }

    #[inline]
    pub fn mask16(mask: Vec128Mask16, vec: Vec128) -> Vec128 {
        unsafe { _mm_maskz_mov_epi16(mask, vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn add8(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_add_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn add16(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_add_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn add32(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_add_epi32(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn sub8(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_sub_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn sub16(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_sub_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn sub32(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_sub_epi32(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn min8(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_min_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn min16(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_min_epi16(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn max8(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_max_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn max16(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_max_epi16(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn clamp8(value: Vec128, min: Vec128, max: Vec128) -> Vec128 {
        Vec128::max8(Vec128::min8(value, max), min)
    }

    #[inline]
    pub fn clamp16(value: Vec128, min: Vec128, max: Vec128) -> Vec128 {
        Vec128::max16(Vec128::min16(value, max), min)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl16<const SHIFT:i32>(vec: Vec128) -> Vec128 {
        unsafe { _mm_slli_epi16::<SHIFT>(vec.raw).into() }
    }

    #[inline]
    pub fn shr16<const SHIFT: i32>(vec: Vec128) -> Vec128 {
        unsafe { _mm_srli_epi16::<SHIFT>(vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl16_mz(mask: Vec128Mask16, a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_maskz_sllv_epi16(mask, a.raw, b.raw).into() }
    }

    #[inline]
    pub fn shr16_mz(mask: Vec128Mask16, a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_maskz_srlv_epi16(mask, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress8(mask: Vec128Mask8, vec: Vec128) -> Vec128 {
        unsafe { _mm_maskz_compress_epi8(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress16(mask: Vec128Mask16, vec: Vec128) -> Vec128 {
        unsafe { _mm_maskz_compress_epi16(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress32(mask: Vec128Mask32, vec: Vec128) -> Vec128 {
        unsafe { _mm_maskz_compress_epi32(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress64(mask: Vec128Mask64, vec: Vec128) -> Vec128 {
        unsafe { _mm_maskz_compress_epi64(mask, vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress_store8<T>(dest: *mut T, mask: Vec128Mask8, vec: Vec128) {
        unsafe { _mm_mask_compressstoreu_epi8(dest.cast(), mask, vec.raw) }
    }

    #[inline]
    pub fn compress_store16<T>(dest: *mut T, mask: Vec128Mask16, vec: Vec128) {
        unsafe { _mm_mask_compressstoreu_epi16(dest.cast(), mask, vec.raw) }
    }

    #[inline]
    pub fn compress_store32<T>(dest: *mut T, mask: Vec128Mask32, vec: Vec128) {
        unsafe { _mm_mask_compressstoreu_epi32(dest.cast(), mask, vec.raw) }
    }

    #[inline]
    pub fn compress_store64<T>(dest: *mut T, mask: Vec128Mask64, vec: Vec128) {
        unsafe { _mm_mask_compressstoreu_epi64(dest.cast(), mask, vec.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn bitshuffle(a: Vec128, b: Vec128) -> Vec128Mask8 {
        unsafe { _mm_bitshuffle_epi64_mask(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn mask_bitshuffle(mask: Vec128Mask8, a: Vec128, b: Vec128) -> Vec128Mask8 {
        unsafe { _mm_mask_bitshuffle_epi64_mask(mask, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn gf2p8matmul8(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_gf2p8affine_epi64_epi8::<0>(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero8(self) -> Vec128Mask8 {
        unsafe { _mm_cmpeq_epu8_mask(self.raw, Vec128::zero().raw) }
    }

    #[inline]
    pub fn zero16(self) -> Vec128Mask16 {
        unsafe { _mm_cmpeq_epu16_mask(self.raw, Vec128::zero().raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn nonzero8(self) -> Vec128Mask8 {
        unsafe { _mm_cmpneq_epu8_mask(self.raw, Vec128::zero().raw) }
    }

    #[inline]
    pub fn nonzero16(self) -> Vec128Mask16 {
        unsafe { _mm_cmpneq_epu16_mask(self.raw, Vec128::zero().raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn msb8(self) -> Vec128Mask8 {
        unsafe { _mm_movepi8_mask(self.raw) }
    }

    #[inline]
    pub fn msb16(self) -> Vec128Mask16 {
        unsafe { _mm_movepi16_mask(self.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        unsafe { _mm_cmpeq_epu8_mask(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn eq16(a: Vec128, b: Vec128) -> Vec128Mask16 {
        unsafe { _mm_cmpeq_epu16_mask(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn neq8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        unsafe { _mm_cmpneq_epu8_mask(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn neq16(a: Vec128, b: Vec128) -> Vec128Mask16 {
        unsafe { _mm_cmpneq_epu16_mask(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn test8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        unsafe { _mm_test_epi8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn test16(a: Vec128, b: Vec128) -> Vec128Mask16 {
        unsafe { _mm_test_epi16_mask(a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn testn8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        unsafe { _mm_testn_epi8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn testn16(a: Vec128, b: Vec128) -> Vec128Mask16 {
        unsafe { _mm_testn_epi16_mask(a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn findset8(haystack: Vec128, haystack_len: i32, needles: Vec128) -> u16 {
        unsafe { _mm_extract_epi16::<0>(_mm_cmpestrm::<0>(haystack.raw, haystack_len, needles.raw, 16)) as u16 }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zext8to16lo(self) -> Vec128 {
        unsafe { _mm_cvtepu8_epi16(self.raw).into() }
    }

    #[inline]
    pub fn zext8to16(self) -> Vec256 {
        unsafe { _mm256_cvtepu8_epi16(self.raw).into() }
    }

    /*----------------------------------------------------------------*/

    pub const SIZE: usize = size_of::<Vec128>();
    pub const CHUNKS_8: usize = Self::SIZE / size_of::<u8>();
    pub const CHUNKS_16: usize = Self::SIZE / size_of::<u16>();
    pub const CHUNKS_32: usize = Self::SIZE / size_of::<u32>();
    pub const CHUNKS_64: usize = Self::SIZE / size_of::<u64>();
}

impl From<u32> for Vec128 {
    #[inline]
    fn from(value: u32) -> Self {
        unsafe { _mm_cvtsi32_si128(value as i32).into() }
    }
}

impl From<u64> for Vec128 {
    #[inline]
    fn from(value: u64) -> Self {
        unsafe { _mm_cvtsi64_si128(value as i64).into() }
    }
}

impl From<__m128i> for Vec128 {
    #[inline]
    fn from(raw: __m128i) -> Self {
        Self { raw }
    }
}

impl From<[u8; 16]> for Vec128 {
    #[inline]
    fn from(arr: [u8; 16]) -> Self {
        unsafe { _mm_loadu_si128(arr.as_ptr().cast()).into() }
    }
}

impl From<[u16; 8]> for Vec128 {
    #[inline]
    fn from(arr: [u16; 8]) -> Self {
        unsafe { _mm_loadu_si128(arr.as_ptr().cast()).into() }
    }
}

impl From<[u32; 4]> for Vec128 {
    #[inline]
    fn from(arr: [u32; 4]) -> Self {
        unsafe { _mm_loadu_si128(arr.as_ptr().cast()).into() }
    }
}

impl From<[u64; 2]> for Vec128 {
    #[inline]
    fn from(arr: [u64; 2]) -> Self {
        unsafe { _mm_loadu_si128(arr.as_ptr().cast()).into() }
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_vec128_ops {
    ($($trait:ident, $fn:ident, $intrinsic:ident;)*) => {$(
        impl $trait for Vec128 {
            type Output = Self;

            #[inline]
            fn $fn(self, other: Vec128) -> Vec128 {
                unsafe { $intrinsic(self.raw, other.raw).into() }
            }
        }
    )*}
}

macro_rules! impl_vec128_assign_ops {
    ($($trait:ident, $fn:ident, $intrinsic:ident;)*) => {$(
        impl $trait for Vec128 {
            #[inline]
            fn $fn(&mut self, other: Vec128) {
                self.raw = unsafe { $intrinsic(self.raw, other.raw) };
            }
        }
    )*}
}

impl_vec128_ops! {
    BitAnd, bitand, _mm_and_si128;
    BitOr, bitor, _mm_or_si128;
    BitXor, bitxor, _mm_xor_si128;
}

impl_vec128_assign_ops! {
    BitAndAssign, bitand_assign, _mm_and_si128;
    BitOrAssign, bitor_assign, _mm_or_si128;
    BitXorAssign, bitxor_assign, _mm_xor_si128;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Vec256 {
    raw: __m256i
}

impl Vec256 {
    #[inline]
    pub fn load<T>(src: *const T) -> Vec256 {
        unsafe { _mm256_loadu_si256(src.cast()).into() }
    }

    #[inline]
    pub fn store<T>(dst: *mut T, src: Vec256) {
        unsafe { _mm256_storeu_si256(dst.cast(), src.raw); }
    }
    
    /*----------------------------------------------------------------*/

    #[inline]
    pub fn into_vec128(self) -> Vec128 {
        unsafe { _mm256_castsi256_si128(self.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero() -> Vec256 {
        unsafe { _mm256_setzero_si256().into() }
    }
    
    #[inline]
    pub fn splat8(value: u8) -> Vec256 {
        unsafe { _mm256_set1_epi8(value as i8).into() }
    }

    #[inline]
    pub fn splat16(value: u16) -> Vec256 {
        unsafe { _mm256_set1_epi16(value as i16).into() }
    }

    #[inline]
    pub fn splat32(value: u32) -> Vec256 {
        unsafe { _mm256_set1_epi32(value as i32).into() }
    }

    #[inline]
    pub fn splat64(value: u64) -> Vec256 {
        unsafe { _mm256_set1_epi64x(value as i64).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask8(mask: Vec256Mask8, vec: Vec256) -> Vec256 {
        unsafe { _mm256_maskz_mov_epi8(mask, vec.raw).into() }
    }

    #[inline]
    pub fn mask16(mask: Vec256Mask16, vec: Vec256) -> Vec256 {
        unsafe { _mm256_maskz_mov_epi16(mask, vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn add8(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_add_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn add16(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_add_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn add32(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_add_epi32(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn sub8(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_sub_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn sub16(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_sub_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn sub32(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_sub_epi32(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn min8(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_min_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn min16(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_min_epi16(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn max8(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_max_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn max16(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_max_epi16(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn clamp8(value: Vec256, min: Vec256, max: Vec256) -> Vec256 {
        Vec256::max8(Vec256::min8(value, max), min)
    }

    #[inline]
    pub fn clamp16(value: Vec256, min: Vec256, max: Vec256) -> Vec256 {
        Vec256::max16(Vec256::min16(value, max), min)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl16<const SHIFT:i32>(vec: Vec256) -> Vec256 {
        unsafe { _mm256_slli_epi16::<SHIFT>(vec.raw).into() }
    }

    #[inline]
    pub fn shr16<const SHIFT: i32>(vec: Vec256) -> Vec256 {
        unsafe { _mm256_srli_epi16::<SHIFT>(vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl16_mz(mask: Vec256Mask16, a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_maskz_sllv_epi16(mask, a.raw, b.raw).into() }
    }

    #[inline]
    pub fn shr16_mz(mask: Vec256Mask16, a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_maskz_srlv_epi16(mask, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress8(mask: Vec256Mask8, vec: Vec256) -> Vec256 {
        unsafe { _mm256_maskz_compress_epi8(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress16(mask: Vec256Mask16, vec: Vec256) -> Vec256 {
        unsafe { _mm256_maskz_compress_epi16(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress32(mask: Vec256Mask32, vec: Vec256) -> Vec256 {
        unsafe { _mm256_maskz_compress_epi32(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress64(mask: Vec256Mask64, vec: Vec256) -> Vec256 {
        unsafe { _mm256_maskz_compress_epi64(mask, vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress_store8<T>(dest: *mut T, mask: Vec256Mask8, vec: Vec256) {
        unsafe { _mm256_mask_compressstoreu_epi8(dest.cast(), mask, vec.raw) }
    }

    #[inline]
    pub fn compress_store16<T>(dest: *mut T, mask: Vec256Mask16, vec: Vec256) {
        unsafe { _mm256_mask_compressstoreu_epi16(dest.cast(), mask, vec.raw) }
    }

    #[inline]
    pub fn compress_store32<T>(dest: *mut T, mask: Vec256Mask32, vec: Vec256) {
        unsafe { _mm256_mask_compressstoreu_epi32(dest.cast(), mask, vec.raw) }
    }

    #[inline]
    pub fn compress_store64<T>(dest: *mut T, mask: Vec256Mask64, vec: Vec256) {
        unsafe { _mm256_mask_compressstoreu_epi64(dest.cast(), mask, vec.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn bitshuffle(a: Vec256, b: Vec256) -> Vec256Mask8 {
        unsafe { _mm256_bitshuffle_epi64_mask(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn mask_bitshuffle(mask: Vec256Mask8, a: Vec256, b: Vec256) -> Vec256Mask8 {
        unsafe { _mm256_mask_bitshuffle_epi64_mask(mask, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn gf2p8matmul8(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_gf2p8affine_epi64_epi8::<0>(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero8(self) -> Vec256Mask8 {
        unsafe { _mm256_cmpeq_epu8_mask(self.raw, Vec256::zero().raw) }
    }

    #[inline]
    pub fn zero16(self) -> Vec256Mask16 {
        unsafe { _mm256_cmpeq_epu16_mask(self.raw, Vec256::zero().raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn nonzero8(self) -> Vec256Mask8 {
        unsafe { _mm256_cmpneq_epu8_mask(self.raw, Vec256::zero().raw) }
    }

    #[inline]
    pub fn nonzero16(self) -> Vec256Mask16 {
        unsafe { _mm256_cmpneq_epu16_mask(self.raw, Vec256::zero().raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn msb8(self) -> Vec256Mask8 {
        unsafe { _mm256_movepi8_mask(self.raw) }
    }

    #[inline]
    pub fn msb16(self) -> Vec256Mask16 {
        unsafe { _mm256_movepi16_mask(self.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq8(a: Vec256, b: Vec256) -> Vec256Mask8 {
        unsafe { _mm256_cmpeq_epu8_mask(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn eq16(a: Vec256, b: Vec256) -> Vec256Mask16 {
        unsafe { _mm256_cmpeq_epu16_mask(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn neq8(a: Vec256, b: Vec256) -> Vec256Mask8 {
        unsafe { _mm256_cmpneq_epu8_mask(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn neq16(a: Vec256, b: Vec256) -> Vec256Mask16 {
        unsafe { _mm256_cmpneq_epu16_mask(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn test8(a: Vec256, b: Vec256) -> Vec256Mask8 {
        unsafe { _mm256_test_epi8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn test16(a: Vec256, b: Vec256) -> Vec256Mask16 {
        unsafe { _mm256_test_epi16_mask(a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn testn8(a: Vec256, b: Vec256) -> Vec256Mask8 {
        unsafe { _mm256_testn_epi8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn testn16(a: Vec256, b: Vec256) -> Vec256Mask16 {
        unsafe { _mm256_testn_epi16_mask(a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zext8to16(self) -> Vec512 {
        unsafe { _mm512_cvtepu8_epi16(self.raw).into() }
    }

    /*----------------------------------------------------------------*/

    pub const SIZE: usize = size_of::<Vec256>();
    pub const CHUNKS_8: usize = Self::SIZE / size_of::<u8>();
    pub const CHUNKS_16: usize = Self::SIZE / size_of::<u16>();
    pub const CHUNKS_32: usize = Self::SIZE / size_of::<u32>();
    pub const CHUNKS_64: usize = Self::SIZE / size_of::<u64>();
}

impl From<Vec128> for Vec256 {
    #[inline]
    fn from(vec: Vec128) -> Vec256 {
        unsafe { _mm256_castsi128_si256(vec.raw).into() }
    }
}

impl From<__m256i> for Vec256 {
    #[inline]
    fn from(raw: __m256i) -> Self {
        Self { raw }
    }
}

impl From<[u8; 32]> for Vec256 {
    #[inline]
    fn from(arr: [u8; 32]) -> Self {
        unsafe { _mm256_loadu_si256(arr.as_ptr().cast()).into() }
    }
}

impl From<[u16; 16]> for Vec256 {
    #[inline]
    fn from(arr: [u16; 16]) -> Self {
        unsafe { _mm256_loadu_si256(arr.as_ptr().cast()).into() }
    }
}

impl From<[u32; 8]> for Vec256 {
    #[inline]
    fn from(arr: [u32; 8]) -> Self {
        unsafe { _mm256_loadu_si256(arr.as_ptr().cast()).into() }
    }
}

impl From<[u64; 4]> for Vec256 {
    #[inline]
    fn from(arr: [u64; 4]) -> Self {
        unsafe { _mm256_loadu_si256(arr.as_ptr().cast()).into() }
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_vec256_ops {
    ($($trait:ident, $fn:ident, $intrinsic:ident;)*) => {$(
        impl $trait for Vec256 {
            type Output = Self;

            #[inline]
            fn $fn(self, other: Vec256) -> Vec256 {
                unsafe { $intrinsic(self.raw, other.raw).into() }
            }
        }
    )*}
}

macro_rules! impl_vec256_assign_ops {
    ($($trait:ident, $fn:ident, $intrinsic:ident;)*) => {$(
        impl $trait for Vec256 {
            #[inline]
            fn $fn(&mut self, other: Vec256) {
                self.raw = unsafe { $intrinsic(self.raw, other.raw) };
            }
        }
    )*}
}

impl_vec256_ops! {
    BitAnd, bitand, _mm256_and_si256;
    BitOr, bitor, _mm256_or_si256;
    BitXor, bitxor, _mm256_xor_si256;
}

impl_vec256_assign_ops! {
    BitAndAssign, bitand_assign, _mm256_and_si256;
    BitOrAssign, bitor_assign, _mm256_or_si256;
    BitXorAssign, bitxor_assign, _mm256_xor_si256;
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Vec512 {
    raw: __m512i
}

impl Vec512 {
    #[inline]
    pub fn load<T>(src: *const T) -> Vec512 {
        unsafe { _mm512_loadu_si512(src.cast()).into() }
    }

    #[inline]
    pub fn store<T>(dst: *mut T, src: Vec512) {
        unsafe { _mm512_storeu_si512(dst.cast(), src.raw); }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn into_u32(self) -> u32 {
        unsafe { _mm512_cvtsi512_si32(self.raw) as u32 }
    }

    #[inline]
    pub fn into_vec128(self) -> Vec128 {
        unsafe { _mm512_castsi512_si128(self.raw).into() }
    }

    #[inline]
    pub fn into_vec256(self) -> Vec256 {
        unsafe { _mm512_castsi512_si256(self.raw).into() }
    }
    
    #[inline]
    pub fn extract_vec256<const INDEX: i32>(self) -> Vec256 {
        unsafe { _mm512_extracti64x4_epi64::<INDEX>(self.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero() -> Vec512 {
        unsafe { _mm512_setzero_si512().into() }
    }

    #[inline]
    pub fn splat8(value: u8) -> Vec512 {
        unsafe { _mm512_set1_epi8(value as i8).into() }
    }

    #[inline]
    pub fn splat16(value: u16) -> Vec512 {
        unsafe { _mm512_set1_epi16(value as i16).into() }
    }

    #[inline]
    pub fn splat32(value: u32) -> Vec512 {
        unsafe { _mm512_set1_epi32(value as i32).into() }
    }

    #[inline]
    pub fn splat64(value: u64) -> Vec512 {
        unsafe { _mm512_set1_epi64(value as i64).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask_splat8(vec: Vec512, mask: Vec512Mask8, value: u8) -> Vec512 {
        unsafe { _mm512_mask_set1_epi8(vec.raw, mask, value as i8).into() }
    }

    #[inline]
    pub fn mask_splat16(vec: Vec512, mask: Vec512Mask16, value: u16) -> Vec512 {
        unsafe { _mm512_mask_set1_epi16(vec.raw, mask, value as i16).into() }
    }

    #[inline]
    pub fn mask_splat32(vec: Vec512, mask: Vec512Mask32, value: u32) -> Vec512 {
        unsafe { _mm512_mask_set1_epi32(vec.raw, mask, value as i32).into() }
    }

    #[inline]
    pub fn mask_splat64(vec: Vec512, mask: Vec512Mask64, value: u64) -> Vec512 {
        unsafe { _mm512_mask_set1_epi64(vec.raw, mask, value as i64).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask8(mask: Vec512Mask8, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_mov_epi8(mask, vec.raw).into() }
    }

    #[inline]
    pub fn mask16(mask: Vec512Mask16, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_mov_epi16(mask, vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn add8(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_add_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn add16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_add_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn add32(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_add_epi32(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn sub8(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_sub_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn sub16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_sub_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn sub32(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_sub_epi32(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn madd16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_madd_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn mullo16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_mullo_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn reduce_add32(self) -> i32 {
        unsafe { _mm512_reduce_add_epi32(self.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn min8(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_min_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn min16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_min_epi16(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn max8(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_max_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn max16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_max_epi16(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn clamp8(value: Vec512, min: Vec512, max: Vec512) -> Vec512 {
        Vec512::max8(Vec512::min8(value, max), min)
    }

    #[inline]
    pub fn clamp16(value: Vec512, min: Vec512, max: Vec512) -> Vec512 {
        Vec512::max16(Vec512::min16(value, max), min)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl16<const SHIFT: u32>(vec: Vec512) -> Vec512 {
        unsafe { _mm512_slli_epi16::<SHIFT>(vec.raw).into() }
    }

    #[inline]
    pub fn shr16<const SHIFT: u32>(vec: Vec512) -> Vec512 {
        unsafe { _mm512_srli_epi16::<SHIFT>(vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl16_mz(mask: Vec512Mask16, a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_sllv_epi16(mask, a.raw, b.raw).into() }
    }

    #[inline]
    pub fn shr16_mz(mask: Vec512Mask16, a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_srlv_epi16(mask, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress8(mask: Vec512Mask8, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_compress_epi8(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress16(mask: Vec512Mask16, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_compress_epi16(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress32(mask: Vec512Mask32, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_compress_epi32(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress64(mask: Vec512Mask64, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_compress_epi64(mask, vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress_store8<T>(dest: *mut T, mask: Vec512Mask8, vec: Vec512) {
        unsafe { _mm512_mask_compressstoreu_epi8(dest.cast(), mask, vec.raw) }
    }

    #[inline]
    pub fn compress_store16<T>(dest: *mut T, mask: Vec512Mask16, vec: Vec512) {
        unsafe { _mm512_mask_compressstoreu_epi16(dest.cast(), mask, vec.raw) }
    }

    #[inline]
    pub fn compress_store32<T>(dest: *mut T, mask: Vec512Mask32, vec: Vec512) {
        unsafe { _mm512_mask_compressstoreu_epi32(dest.cast(), mask, vec.raw) }
    }

    #[inline]
    pub fn compress_store64<T>(dest: *mut T, mask: Vec512Mask64, vec: Vec512) {
        unsafe { _mm512_mask_compressstoreu_epi64(dest.cast(), mask, vec.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn blend8(mask: Vec512Mask8, a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_mask_blend_epi8(mask, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn interleave16(a: u16, b: u16) -> u16 {
        unsafe { _mm512_kunpackb(b, a) }
    }

    #[inline]
    pub fn interleave32(a: u32, b: u32) -> u32 {
        unsafe { _mm512_kunpackw(b, a) }
    }

    #[inline]
    pub fn interleave64(a: u64, b: u64) -> u64 {
        unsafe { _mm512_kunpackd(b, a) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn permute8(index: Vec512, vec: Vec512) -> Vec512 {
        unsafe { _mm512_permutexvar_epi8(index.raw, vec.raw).into() }
    }

    #[inline]
    pub fn permute16(index: Vec512, vec: Vec512) -> Vec512 {
        unsafe { _mm512_permutexvar_epi16(index.raw, vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn permute8_mz(mask: Vec512Mask8, index: Vec512, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_permutexvar_epi8(mask, index.raw, vec.raw).into() }
    }

    #[inline]
    pub fn permute16_mz(mask: Vec512Mask16, index: Vec512, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_permutexvar_epi16(mask, index.raw, vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn permute2var8(index: Vec512, a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_permutex2var_epi8(index.raw, a.raw, b.raw).into() }
    }

    #[inline]
    pub fn permute2var16(index: Vec512, a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_permutex2var_epi16(index.raw, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn permute2var8_mz(mask: Vec512Mask8, index: Vec512, a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_permutex2var_epi8(mask, index.raw, a.raw, b.raw).into() }
    }

    #[inline]
    pub fn permute2var16_mz(mask: Vec512Mask16, index: Vec512, a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_permutex2var_epi16(mask, index.raw, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn permute8_128(index: Vec512, vec: Vec128) -> Vec512 {
        unsafe { _mm512_shuffle_epi8(_mm512_broadcast_i32x4(vec.raw), index.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn bitshuffle(a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_bitshuffle_epi64_mask(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn mask_bitshuffle(mask: Vec512Mask8, a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_mask_bitshuffle_epi64_mask(mask, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn lane_splat8to64(vec: Vec512) -> Vec512 {
        let vec = Vec512::gf2p8matmul8(Vec512::splat64(0x0102040810204080), vec);

        Vec512::gf2p8matmul8(Vec512::splat64(0xFFFFFFFFFFFFFFFF), vec)
    }

    #[inline]
    pub fn gf2p8matmul8(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_gf2p8affine_epi64_epi8::<0>(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero8(self) -> Vec512Mask8 {
        unsafe { _mm512_cmpeq_epu8_mask(self.raw, Vec512::zero().raw) }
    }

    #[inline]
    pub fn zero16(self) -> Vec512Mask16 {
        unsafe { _mm512_cmpeq_epu16_mask(self.raw, Vec512::zero().raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn nonzero8(self) -> Vec512Mask8 {
        unsafe { _mm512_cmpneq_epu8_mask(self.raw, Vec512::zero().raw) }
    }

    #[inline]
    pub fn nonzero16(self) -> Vec512Mask16 {
        unsafe { _mm512_cmpneq_epu16_mask(self.raw, Vec512::zero().raw) }
    }

    #[inline]
    pub fn nonzero32(self) -> Vec512Mask32 {
        unsafe { _mm512_cmpneq_epu32_mask(self.raw, Vec512::zero().raw) }
    }

    #[inline]
    pub fn nonzero64(self) -> Vec512Mask64 {
        unsafe { _mm512_cmpneq_epu64_mask(self.raw, Vec512::zero().raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn msb8(self) -> Vec512Mask8 {
        unsafe { _mm512_movepi8_mask(self.raw) }
    }

    #[inline]
    pub fn msb16(self) -> Vec512Mask16 {
        unsafe { _mm512_movepi16_mask(self.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq8(a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_cmpeq_epu8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn eq16(a: Vec512, b: Vec512) -> Vec512Mask16 {
        unsafe { _mm512_cmpeq_epu16_mask(a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask_eq8(mask: Vec512Mask8, a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_mask_cmpeq_epu8_mask(mask, a.raw, b.raw) }
    }

    #[inline]
    pub fn mask_eq16(mask: Vec512Mask16, a: Vec512, b: Vec512) -> Vec512Mask16 {
        unsafe { _mm512_mask_cmpeq_epu16_mask(mask, a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn neq8(a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_cmpneq_epu8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn neq16(a: Vec512, b: Vec512) -> Vec512Mask16 {
        unsafe { _mm512_cmpneq_epu16_mask(a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask_neq8(mask: Vec512Mask8, a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_mask_cmpneq_epu8_mask(mask, a.raw, b.raw) }
    }

    #[inline]
    pub fn mask_neq16(mask: Vec512Mask16, a: Vec512, b: Vec512) -> Vec512Mask16 {
        unsafe { _mm512_mask_cmpneq_epu16_mask(mask, a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn test8(a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_test_epi8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn test16(a: Vec512, b: Vec512) -> Vec512Mask16 {
        unsafe { _mm512_test_epi16_mask(a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn testn8(a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_testn_epi8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn testn16(a: Vec512, b: Vec512) -> Vec512Mask16 {
        unsafe { _mm512_testn_epi16_mask(a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    pub const SIZE: usize = size_of::<Vec512>();
    pub const CHUNKS_8: usize = Self::SIZE / size_of::<u8>();
    pub const CHUNKS_16: usize = Self::SIZE / size_of::<u16>();
    pub const CHUNKS_32: usize = Self::SIZE / size_of::<u32>();
    pub const CHUNKS_64: usize = Self::SIZE / size_of::<u64>();
}

impl From<Vec128> for Vec512 {
    #[inline]
    fn from(vec: Vec128) -> Vec512 {
        unsafe { _mm512_castsi128_si512(vec.raw).into() }
    }
}

impl From<Vec256> for Vec512 {
    #[inline]
    fn from(vec: Vec256) -> Vec512 {
        unsafe { _mm512_castsi256_si512(vec.raw).into() }
    }
}

impl From<__m512i> for Vec512 {
    #[inline]
    fn from(raw: __m512i) -> Self {
        Self { raw }
    }
}

impl From<[u8; 64]> for Vec512 {
    #[inline]
    fn from(arr: [u8; 64]) -> Self {
        unsafe { _mm512_loadu_si512(arr.as_ptr().cast()).into() }
    }
}

impl From<[u16; 32]> for Vec512 {
    #[inline]
    fn from(arr: [u16; 32]) -> Self {
        unsafe { _mm512_loadu_si512(arr.as_ptr().cast()).into() }
    }
}

impl From<[u32; 16]> for Vec512 {
    #[inline]
    fn from(arr: [u32; 16]) -> Self {
        unsafe { _mm512_loadu_si512(arr.as_ptr().cast()).into() }
    }
}

impl From<[u64; 8]> for Vec512 {
    #[inline]
    fn from(arr: [u64; 8]) -> Self {
        unsafe { _mm512_loadu_si512(arr.as_ptr().cast()).into() }
    }
}

/*----------------------------------------------------------------*/

macro_rules! impl_vec512_ops {
    ($($trait:ident, $fn:ident, $intrinsic:ident;)*) => {$(
        impl $trait for Vec512 {
            type Output = Self;

            #[inline]
            fn $fn(self, other: Vec512) -> Vec512 {
                unsafe { $intrinsic(self.raw, other.raw).into() }
            }
        }
    )*}
}

macro_rules! impl_vec512_assign_ops {
    ($($trait:ident, $fn:ident, $intrinsic:ident;)*) => {$(
        impl $trait for Vec512 {
            #[inline]
            fn $fn(&mut self, other: Vec512) {
                self.raw = unsafe { $intrinsic(self.raw, other.raw) };
            }
        }
    )*}
}

impl_vec512_ops! {
    BitAnd, bitand, _mm512_and_si512;
    BitOr, bitor, _mm512_or_si512;
    BitXor, bitxor, _mm512_xor_si512;
}

impl_vec512_assign_ops! {
    BitAndAssign, bitand_assign, _mm512_and_si512;
    BitOrAssign, bitor_assign, _mm512_or_si512;
    BitXorAssign, bitxor_assign, _mm512_xor_si512;
}