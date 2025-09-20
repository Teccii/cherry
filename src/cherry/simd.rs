#[cfg(target_feature = "avx512f")]
mod simd {
    use ::core::arch::x86_64::*;

    pub const I16_CHUNK: usize = size_of::<__m512i>() / size_of::<i16>();

    #[inline]
    pub fn zero() -> __m512i {
        unsafe { _mm512_setzero_si512() }
    }

    #[inline]
    pub fn splat_i16(value: i16) -> __m512i {
        unsafe { _mm512_set1_epi16(value) }
    }

    #[inline]
    pub fn load_i16(src: *const i16) -> __m512i {
        unsafe { _mm512_load_si512(src.cast()) }
    }

    #[inline]
    pub fn store_i16(dst: *mut i16, src: __m512i) {
        unsafe { _mm512_store_si512(dst.cast(), src) }
    }

    #[inline]
    pub fn add_i16(a: __m512i, b: __m512i) -> __m512i {
        unsafe { _mm512_add_epi16(a, b) }
    }

    #[inline]
    pub fn add_i32(a: __m512i, b: __m512i) -> __m512i {
        unsafe { _mm512_add_epi32(a, b) }
    }

    #[inline]
    pub fn sub_i16(a: __m512i, b: __m512i) -> __m512i {
        unsafe { _mm512_sub_epi16(a, b) }
    }

    #[inline]
    pub fn mullo_i16(a: __m512i, b: __m512i) -> __m512i {
        unsafe { _mm512_mullo_epi16(a, b) }
    }

    #[inline]
    pub fn madd_i16(a: __m512i, b: __m512i) -> __m512i {
        unsafe { _mm512_madd_epi16(a, b) }
    }

    #[inline]
    pub fn min_i16(a: __m512i, b: __m512i) -> __m512i {
        unsafe { _mm512_min_epi16(a, b) }
    }

    #[inline]
    pub fn max_i16(a: __m512i, b: __m512i) -> __m512i {
        unsafe { _mm512_max_epi16(a, b) }
    }

    #[inline]
    pub fn clamp_i16(value: __m512i, min: __m512i, max: __m512i) -> __m512i {
        min_i16(max_i16(value, min), max)
    }

    #[inline]
    pub fn reduce_add_i32(vec: __m512i) -> i32 {
        unsafe { _mm512_reduce_add_epi32(vec) }
    }
}

#[cfg(all(
    target_feature = "avx2",
    not(target_feature = "avx512f"))
)]
mod simd {
    use ::core::arch::x86_64::*;

    pub const I16_CHUNK: usize = size_of::<__m256i>() / size_of::<i16>();
    pub const I32_CHUNK: usize = size_of::<__m256i>() / size_of::<i32>();

    #[inline]
    pub fn zero() -> __m256i {
        unsafe { _mm256_setzero_si256() }
    }

    #[inline]
    pub fn splat_i16(value: i16) -> __m256i {
        unsafe { _mm256_set1_epi16(value) }
    }

    #[inline]
    pub fn load_i16(src: *const i16) -> __m256i {
        unsafe { _mm256_load_si256(src.cast()) }
    }

    #[inline]
    pub fn store_i16(dst: *mut i16, src: __m256i) {
        unsafe { _mm256_store_si256(dst.cast(), src) }
    }

    #[inline]
    pub fn add_i16(a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_add_epi16(a, b) }
    }

    #[inline]
    pub fn add_i32(a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_add_epi32(a, b) }
    }

    #[inline]
    pub fn sub_i16(a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_sub_epi16(a, b) }
    }

    #[inline]
    pub fn mullo_i16(a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_mullo_epi16(a, b) }
    }

    #[inline]
    pub fn madd_i16(a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_madd_epi16(a, b) }
    }

    #[inline]
    pub fn min_i16(a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_min_epi16(a, b) }
    }

    #[inline]
    pub fn max_i16(a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_max_epi16(a, b) }
    }

    #[inline]
    pub fn clamp_i16(value: __m256i, min: __m256i, max: __m256i) -> __m256i {
        min_i16(max_i16(value, min), max)
    }

    #[inline]
    pub fn reduce_add_i32(vec: __m256i) -> i32 {
        let mut temp = [0i32; I32_CHUNK];
        unsafe { _mm256_storeu_si256(temp.as_mut_ptr().cast(), vec); }

        temp.iter().sum()
    }
}

#[cfg(all(
    not(target_feature = "avx2"),
    not(target_feature = "avx512f")
))]
mod simd {
    use ::core::arch::x86_64::*;

    pub const I16_CHUNK: usize = size_of::<__m128i>() / size_of::<i16>();
    pub const I32_CHUNK: usize = size_of::<__m128i>() / size_of::<i32>();

    #[inline]
    pub fn zero() -> __m128i {
        unsafe { _mm_setzero_si128() }
    }

    #[inline]
    pub fn splat_i16(value: i16) -> __m128i {
        unsafe { _mm_set1_epi16(value) }
    }

    #[inline]
    pub fn load_i16(src: *const i16) -> __m128i {
        unsafe { _mm_load_si128(src.cast()) }
    }

    #[inline]
    pub fn store_i16(dst: *mut i16, src: __m128i) {
        unsafe { _mm_store_si128(dst.cast(), src) }
    }

    #[inline]
    pub fn add_i16(a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_add_epi16(a, b) }
    }

    #[inline]
    pub fn add_i32(a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_add_epi32(a, b) }
    }

    #[inline]
    pub fn sub_i16(a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_sub_epi16(a, b) }
    }

    #[inline]
    pub fn mullo_i16(a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_mullo_epi16(a, b) }
    }

    #[inline]
    pub fn madd_i16(a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_madd_epi16(a, b) }
    }

    #[inline]
    pub fn min_i16(a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_min_epi16(a, b) }
    }

    #[inline]
    pub fn max_i16(a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_max_epi16(a, b) }
    }

    #[inline]
    pub fn clamp_i16(value: __m128i, min: __m128i, max: __m128i) -> __m128i {
        min_i16(max_i16(value, min), max)
    }

    #[inline]
    pub fn reduce_add_i32(vec: __m128i) -> i32 {
        let mut temp = [0i32; I32_CHUNK];
        unsafe { _mm_storeu_si128(temp.as_mut_ptr().cast(), vec); }

        temp.iter().sum()
    }
}

pub use simd::*;