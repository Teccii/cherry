use core::{arch::x86_64::*, ops::*};

use super::common::*;

/*----------------------------------------------------------------*/

#[inline]
pub fn interleave64(a: u32, b: u32) -> u64 {
    unsafe { _mm512_kunpackd(b as u64, a as u64) }
}

/*----------------------------------------------------------------*/

pub type NativeVec = Vec512;

/*----------------------------------------------------------------*/

impl Vec128 {
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
    pub fn compress_store16<T>(dest: *mut T, mask: Vec128Mask16, vec: Vec128) {
        unsafe { _mm_mask_compressstoreu_epi16(dest.cast(), mask, vec.raw) }
    }

    /*----------------------------------------------------------------*/

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
    pub fn msb8(self) -> Vec128Mask8 {
        unsafe { _mm_movepi8_mask(self.raw) }
    }

    #[inline]
    pub fn nonzero8(self) -> Vec128Mask8 {
        unsafe { _mm_cmpneq_epu8_mask(self.raw, Vec128::zero().raw) }
    }

    #[inline]
    pub fn eq8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        unsafe { _mm_cmpeq_epu8_mask(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn neq8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        unsafe { _mm_cmpneq_epu8_mask(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn testn8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        unsafe { _mm_testn_epi8_mask(a.raw, b.raw) }
    }
}

/*----------------------------------------------------------------*/

impl Vec256 {
    #[inline]
    pub fn mask16(mask: Vec256Mask16, vec: Vec256) -> Vec256 {
        unsafe { _mm256_maskz_mov_epi16(mask, vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress_store16<T>(dest: *mut T, mask: Vec256Mask16, vec: Vec256) {
        unsafe { _mm256_mask_compressstoreu_epi16(dest.cast(), mask, vec.raw) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zext8to16(self) -> Vec512 {
        unsafe { _mm512_cvtepu8_epi16(self.raw).into() }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Vec512 {
    pub raw: __m512i,
}

impl Vec512 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> Vec512 {
        unsafe { _mm512_loadu_si512(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(dst: *mut T, src: Vec512) {
        unsafe {
            _mm512_storeu_si512(dst.cast(), src.raw);
        }
    }

    /*----------------------------------------------------------------*/

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
    pub fn splat64(value: u64) -> Vec512 {
        unsafe { _mm512_set1_epi64(value as i64).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask_splat8(vec: Vec512, mask: Vec512Mask8, value: u8) -> Vec512 {
        unsafe { _mm512_mask_set1_epi8(vec.raw, mask, value as i8).into() }
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
    pub fn add32(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_add_epi32(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn add16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_add_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn sub16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_sub_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn min16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_min_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn max16(a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_max_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn clamp16(value: Vec512, min: Vec512, max: Vec512) -> Vec512 {
        Vec512::max16(Vec512::min16(value, max), min)
    }

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
    pub fn shr16<const SHIFT: u32>(vec: Vec512) -> Vec512 {
        unsafe { _mm512_srli_epi16::<SHIFT>(vec.raw).into() }
    }

    #[inline]
    pub fn shlv16_mz(mask: Vec512Mask16, a: Vec512, b: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_sllv_epi16(mask, a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress8(mask: Vec512Mask8, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_compress_epi8(mask, vec.raw).into() }
    }

    #[inline]
    pub fn compress_store16<T>(dest: *mut T, mask: Vec512Mask16, vec: Vec512) {
        unsafe { _mm512_mask_compressstoreu_epi16(dest.cast(), mask, vec.raw) }
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

    #[inline]
    pub fn permute8(index: Vec512, vec: Vec512) -> Vec512 {
        unsafe { _mm512_permutexvar_epi8(index.raw, vec.raw).into() }
    }

    #[inline]
    pub fn permute8_mz(mask: Vec512Mask8, index: Vec512, vec: Vec512) -> Vec512 {
        unsafe { _mm512_maskz_permutexvar_epi8(mask, index.raw, vec.raw).into() }
    }

    #[inline]
    pub fn permute8_128(index: Vec512, vec: Vec128) -> Vec512 {
        unsafe { _mm512_shuffle_epi8(_mm512_broadcast_i32x4(vec.raw), index.raw).into() }
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

    #[inline]
    pub fn nonzero8(self) -> Vec512Mask8 {
        unsafe { _mm512_cmpneq_epu8_mask(self.raw, Vec512::zero().raw) }
    }

    #[inline]
    pub fn nonzero16(self) -> Vec512Mask16 {
        unsafe { _mm512_cmpneq_epu16_mask(self.raw, Vec512::zero().raw) }
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

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq8(a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_cmpeq_epu8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn testn8(a: Vec512, b: Vec512) -> Vec512Mask8 {
        unsafe { _mm512_testn_epi8_mask(a.raw, b.raw) }
    }

    #[inline]
    pub fn test16(a: Vec512, b: Vec512) -> Vec512Mask16 {
        unsafe { _mm512_test_epi16_mask(a.raw, b.raw) }
    }

    /*----------------------------------------------------------------*/

    pub const SIZE: usize = size_of::<Vec512>();
    pub const CHUNKS_8: usize = Self::SIZE / size_of::<u8>();
    pub const CHUNKS_16: usize = Self::SIZE / size_of::<u16>();
    pub const CHUNKS_32: usize = Self::SIZE / size_of::<u32>();
    pub const CHUNKS_64: usize = Self::SIZE / size_of::<u16>();
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
        unsafe { Vec512::load(arr.as_ptr()) }
    }
}

impl From<[u16; 32]> for Vec512 {
    #[inline]
    fn from(arr: [u16; 32]) -> Self {
        unsafe { Vec512::load(arr.as_ptr()) }
    }
}

impl From<[u32; 16]> for Vec512 {
    #[inline]
    fn from(arr: [u32; 16]) -> Self {
        unsafe { Vec512::load(arr.as_ptr()) }
    }
}

impl From<[u64; 8]> for Vec512 {
    #[inline]
    fn from(arr: [u64; 8]) -> Self {
        unsafe { Vec512::load(arr.as_ptr()) }
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
