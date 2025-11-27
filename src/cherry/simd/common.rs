use core::{arch::x86_64::*, ops::*};

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

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Vec128 {
    pub raw: __m128i,
}

impl Vec128 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> Vec128 {
        unsafe { _mm_loadu_si128(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(dst: *mut T, src: Vec128) {
        unsafe {
            _mm_storeu_si128(dst.cast(), src.raw);
        }
    }

    /*----------------------------------------------------------------*/

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

    #[inline]
    pub fn add64(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_add_epi64(a.raw, b.raw).into() }
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

    #[inline]
    pub fn sub64(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_sub_epi64(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl16<const SHIFT: i32>(vec: Vec128) -> Vec128 {
        unsafe { _mm_slli_epi16::<SHIFT>(vec.raw).into() }
    }

    #[inline]
    pub fn shr16<const SHIFT: i32>(vec: Vec128) -> Vec128 {
        unsafe { _mm_srli_epi16::<SHIFT>(vec.raw).into() }
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
        unsafe { Vec128::load(arr.as_ptr()) }
    }
}

impl From<[u16; 8]> for Vec128 {
    #[inline]
    fn from(arr: [u16; 8]) -> Self {
        unsafe { Vec128::load(arr.as_ptr()) }
    }
}

impl From<[u32; 4]> for Vec128 {
    #[inline]
    fn from(arr: [u32; 4]) -> Self {
        unsafe { Vec128::load(arr.as_ptr()) }
    }
}

impl From<[u64; 2]> for Vec128 {
    #[inline]
    fn from(arr: [u64; 2]) -> Self {
        unsafe { Vec128::load(arr.as_ptr()) }
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
    pub raw: __m256i,
}

impl Vec256 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> Vec256 {
        unsafe { _mm256_loadu_si256(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(dst: *mut T, src: Vec256) {
        unsafe {
            _mm256_storeu_si256(dst.cast(), src.raw);
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn into_u32(self) -> u32 {
        unsafe { _mm256_cvtsi256_si32(self.raw) as u32 }
    }

    #[inline]
    pub fn into_vec128(self) -> Vec128 {
        unsafe { _mm256_castsi256_si128(self.raw).into() }
    }

    #[inline]
    pub fn extract_vec128<const INDEX: i32>(self) -> Vec128 {
        unsafe { _mm256_extracti128_si256::<INDEX>(self.raw).into() }
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

    #[inline]
    pub fn add64(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_add_epi64(a.raw, b.raw).into() }
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

    #[inline]
    pub fn sub64(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_sub_epi64(a.raw, b.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl16<const SHIFT: i32>(vec: Vec256) -> Vec256 {
        unsafe { _mm256_slli_epi16::<SHIFT>(vec.raw).into() }
    }

    #[inline]
    pub fn shr16<const SHIFT: i32>(vec: Vec256) -> Vec256 {
        unsafe { _mm256_srli_epi16::<SHIFT>(vec.raw).into() }
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
        unsafe { Vec256::load(arr.as_ptr()) }
    }
}

impl From<[u16; 16]> for Vec256 {
    #[inline]
    fn from(arr: [u16; 16]) -> Self {
        unsafe { Vec256::load(arr.as_ptr()) }
    }
}

impl From<[u32; 8]> for Vec256 {
    #[inline]
    fn from(arr: [u32; 8]) -> Self {
        unsafe { Vec256::load(arr.as_ptr()) }
    }
}

impl From<[u64; 4]> for Vec256 {
    #[inline]
    fn from(arr: [u64; 4]) -> Self {
        unsafe { Vec256::load(arr.as_ptr()) }
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
