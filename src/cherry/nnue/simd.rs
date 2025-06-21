macro_rules! simd_wrapper {
    (
        $name:ident,
        $target:expr,
        $import:path,
        $vec:ty,
        $zero:ident,
        $splat:ident,
        $load:ident,
        $store:ident,
        $add:ident,
        $sub:ident,
        $max:ident,
        $min:ident
    ) => {
        #[cfg(target_feature = $target)]
        mod $name {
            use $import::*;
            
            pub const I16_CHUNK: usize = size_of::<$vec>() / size_of::<i16>();
            
            /*----------------------------------------------------------------*/
            
            #[inline(always)]
            pub unsafe fn zero_i16() -> $vec {
                unsafe { $zero() }
            }
            
            #[inline(always)]
            pub unsafe fn splat_i16(value: i16) -> $vec {
                unsafe { $splat(value) }
            }
            
            /*----------------------------------------------------------------*/

            #[inline(always)]
            pub unsafe fn load_i16(src: *const i16) -> $vec {
                unsafe { $load(src.cast()) }
            }
        
            #[inline(always)]
            pub unsafe fn store_i16(dst: *mut i16, vec: $vec) {
                unsafe { $store(dst.cast(), vec); }
            }
        
            #[inline(always)]
            pub unsafe fn add_i16(lhs: $vec, rhs: $vec) -> $vec {
                unsafe { $add(lhs, rhs) }
            }
        
            #[inline(always)]
            pub unsafe fn sub_i16(lhs: $vec, rhs: $vec) -> $vec {
                unsafe { $sub(lhs, rhs) }
            }
            
            /*----------------------------------------------------------------*/
            
            #[inline(always)]
            pub unsafe fn max_i16(lhs: $vec, rhs: $vec) -> $vec {
                unsafe { $max(lhs, rhs) }
            }
            
            #[inline(always)]
            pub unsafe fn min_i16(lhs: $vec, rhs: $vec) -> $vec {
                unsafe { $min(lhs, rhs) }
            }
        
            #[inline(always)]
            pub unsafe fn clamp_i16(value: $vec, low: $vec, high: $vec) -> $vec {
                unsafe { $min($max(value, low), high) }
            }
        }
    }
}

simd_wrapper! {
    avx512,
    "avx512f",
    std::arch::x86_64,
    __m512i,
    _mm512_setzero_si512,
    _mm512_set1_epi16,
    _mm512_load_si512,
    _mm512_store_si512,
    _mm512_add_epi16,
    _mm512_sub_epi16,
    _mm512_max_epi16,
    _mm512_min_epi16
}

simd_wrapper! {
    avx2,
    "avx2",
    std::arch::x86_64,
    __m256i,
    _mm256_setzero_si256,
    _mm256_set1_epi16,
    _mm256_load_si256,
    _mm256_store_si256,
    _mm256_add_epi16,
    _mm256_sub_epi16,
    _mm256_max_epi16,
    _mm256_min_epi16
}

simd_wrapper! {
    sse2,
    "sse2",
    std::arch::x86_64,
    __m128i,
    _mm_setzero_si128,
    _mm_set1_epi16,
    _mm_load_si128,
    _mm_store_si128,
    _mm_add_epi16,
    _mm_sub_epi16,
    _mm_max_epi16,
    _mm_min_epi16
}

#[cfg(target_feature = "avx512f")]
pub use avx512::*;

#[cfg(all(
    target_feature = "avx2",
    not(target_feature = "avx512f")
))]
pub use avx2::*;

#[cfg(all(
    target_feature = "sse2",
    not(target_feature = "avx2"),
    not(target_feature = "avx512f")
))]
pub use sse2::*;