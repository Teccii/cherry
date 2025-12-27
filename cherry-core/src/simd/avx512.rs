use core::{arch::x86_64::*, ops::*};

/*----------------------------------------------------------------*/

macro_rules! def_mask {
    ($mask:ident, $ty:ty) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub struct $mask($ty);

        impl $mask {
            #[inline]
            pub fn expand_inner(&mut self) { }

            #[inline]
            pub fn widen(self) -> $mask {
                self
            }

            #[inline]
            pub fn to_bitmask(self) -> $ty {
                self.0
            }
        }

        impl From<$ty> for $mask {
            #[inline]
            fn from(raw: $ty) -> Self {
                Self(raw)
            }
        }

        impl Not for $mask {
            type Output = Self;

            #[inline]
            fn not(self) -> Self {
                Self(!self.0)
            }
        }

        impl BitAnd for $mask {
            type Output = Self;

            #[inline]
            fn bitand(self, other: Self) -> Self {
                Self(self.0 & other.0)
            }
        }

        impl BitOr for $mask {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl BitXor for $mask {
            type Output = Self;

            #[inline]
            fn bitxor(self, other: Self) -> Self {
                Self(self.0 ^ other.0)
            }
        }

        impl BitAnd<$ty> for $mask {
            type Output = Self;

            #[inline]
            fn bitand(self, other: $ty) -> Self {
                Self(self.0 & other)
            }
        }

        impl BitOr<$ty> for $mask {
            type Output = Self;

            #[inline]
            fn bitor(self, other: $ty) -> Self {
                Self(self.0 | other)
            }
        }

        impl BitXor<$ty> for $mask {
            type Output = Self;

            #[inline]
            fn bitxor(self, other: $ty) -> Self {
                Self(self.0 ^ other)
            }
        }

        impl BitAndAssign for $mask {
            #[inline]
            fn bitand_assign(&mut self, other: Self) {
                self.0 &= other.0;
            }
        }

        impl BitOrAssign for $mask {
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0;
            }
        }

        impl BitXorAssign for $mask {
            #[inline]
            fn bitxor_assign(&mut self, other: Self) {
                self.0 ^= other.0;
            }
        }

        impl BitAndAssign<$ty> for $mask {
            #[inline]
            fn bitand_assign(&mut self, other: $ty) {
                self.0 &= other;
            }
        }

        impl BitOrAssign<$ty> for $mask {
            #[inline]
            fn bitor_assign(&mut self, other: $ty) {
                self.0 |= other;
            }
        }

        impl BitXorAssign<$ty> for $mask {
            #[inline]
            fn bitxor_assign(&mut self, other: $ty) {
                self.0 ^= other;
            }
        }
    };

    ($mask:ident, $ty:ty, $next_mask:ident, $next_ty:ty, $concat_shift:expr) => {
        def_mask!($mask, $ty);

        impl $mask {
            #[inline]
            pub fn concat(self, other: $mask) -> $next_mask {
                $next_mask(self.0 as $next_ty | ((other.0 as $next_ty) << $concat_shift))
            }
        }
    }
}

/*----------------------------------------------------------------*/

def_mask!(Mask8, u8, Mask16, u16, 8);
def_mask!(Mask16, u16, Mask32, u32, 16);
def_mask!(Mask32, u32, Mask64, u64, 32);
def_mask!(Mask64, u64);

/*----------------------------------------------------------------*/
pub type Mask8x16 = Mask16;
pub type Mask16x8 = Mask8;
pub type Mask32x4 = Mask8;
pub type Mask64x2 = Mask8;

pub type Mask8x32 = Mask32;
pub type Mask16x16 = Mask16;
pub type Mask32x8 = Mask8;
pub type Mask64x4 = Mask8;

pub type Mask8x64 = Mask64;
pub type Mask16x32 = Mask32;
pub type Mask32x16 = Mask16;
pub type Mask64x8 = Mask8;

pub type Mask16x64 = Mask64;

/*----------------------------------------------------------------*/

macro_rules! def_vec {
    (
        $vec:ident, $raw_vec:ty, $elem_ty:ty, $arr_ty:ty;
        $load:ident,
        $store:ident,
        $splat:ident,
        $bitand:ident,
        $bitor:ident,
        $bitxor:ident
    ) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $vec($raw_vec);

        impl $vec {
            #[inline]
            pub unsafe fn load<T>(src: *const T) -> $vec {
                unsafe { $load(src.cast()).into() }
            }

            #[inline]
            pub unsafe fn store<T>(self, dest: *mut T) {
                unsafe { $store(dest.cast(), self.0) }
            }

            #[inline]
            pub fn splat(value: $elem_ty) -> $vec {
                unsafe { $splat(value as _).into() }
            }
        }

        impl From<$raw_vec> for $vec {
            #[inline]
            fn from(raw: $raw_vec) -> Self {
                Self(raw)
            }
        }

        impl From<$arr_ty> for $vec {
            #[inline]
            fn from(arr: $arr_ty) -> Self {
                unsafe { $vec::load(arr.as_ptr()) }
            }
        }

        impl BitAnd for $vec {
            type Output = Self;

            #[inline]
            fn bitand(self, other: Self) -> Self {
                unsafe { $bitand(self.0, other.0).into() }
            }
        }

        impl BitOr for $vec {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self {
                unsafe { $bitor(self.0, other.0).into() }
            }
        }

        impl BitXor for $vec {
            type Output = Self;

            #[inline]
            fn bitxor(self, other: Self) -> Self {
                unsafe { $bitxor(self.0, other.0).into() }
            }
        }

        impl BitAndAssign for $vec {
            #[inline]
            fn bitand_assign(&mut self, other: Self) {
                *self = *self & other;
            }
        }

        impl BitOrAssign for $vec {
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                *self = *self | other;
            }
        }

        impl BitXorAssign for $vec {
            #[inline]
            fn bitxor_assign(&mut self, other: Self) {
                *self = *self ^ other;
            }
        }
    }
}

macro_rules! impl_conv {
    ($vec:ty, $($conv_fn:ident => $other_ty:ident;)*) => {
        impl $vec {$(
            #[inline]
            pub fn $conv_fn(self) -> $other_ty {
                $other_ty(self.0)
            }
        )*}
    }
}

macro_rules! impl_cmp {
    (
        $vec:ident, $mask_ty:ty;
        $eq:ident,
        $neq:ident,
        $test:ident,
        $testn:ident,
        $msb:ident
    ) => {
        impl $vec {
            #[inline]
            pub fn eq(a: $vec, b: $vec) -> $mask_ty {
                unsafe { $eq(a.0, b.0).into() }
            }

            #[inline]
            pub fn neq(a: $vec, b: $vec) -> $mask_ty {
                unsafe { $neq(a.0, b.0).into() }
            }

            #[inline]
            pub fn test(a: $vec, b: $vec) -> $mask_ty {
                unsafe { $test(a.0, b.0).into() }
            }

            #[inline]
            pub fn testn(a: $vec, b: $vec) -> $mask_ty {
                unsafe { $testn(a.0, b.0).into() }
            }

            /*----------------------------------------------------------------*/

            #[inline]
            pub fn zero(self) -> $mask_ty {
                $vec::eq(self, $vec::splat(0))
            }

            #[inline]
            pub fn nonzero(self) -> $mask_ty {
                $vec::neq(self, $vec::splat(0))
            }

            #[inline]
            pub fn msb(self) -> $mask_ty {
                unsafe { $msb(self.0).into() }
            }
        }
    }
}

macro_rules! impl_select {
    (
        $vec:ident, $mask_ty:ty;
        $mask:ident,
        $blend:ident,
        $compress:ident
    ) => {
        impl $vec {
            #[inline]
            pub fn mask(self, mask: $mask_ty) -> $vec {
                unsafe { $mask(mask.0, self.0).into() }
            }

            #[inline]
            pub fn blend(a: $vec, b: $vec, mask: $mask_ty) -> $vec {
                unsafe { $blend(mask.0, a.0, b.0).into() }
            }

            #[inline]
            pub fn compress(self, mask: $mask_ty) -> $vec {
                unsafe { $compress(mask.0, self.0).into() }
            }

            #[inline]
            pub unsafe fn compress_store<T>(self, mask: $mask_ty, dest: *mut T) {
                unsafe { self.compress(mask).store(dest) }
            }
        }
    };
    (
        $vec:ident, $mask_ty:ty;
        $mask:ident,
        $blend:ident,
        $compress:ident,
        $permute:ident,
        $shuffle:ident
    ) => {
        impl_select! {
            $vec, $mask_ty;
            $mask,
            $blend,
            $compress
        }

        impl $vec {
            #[inline]
            pub fn permute(self, index: $vec) -> $vec {
                unsafe { $permute(index.0, self.0).into() }
            }

            #[inline]
            pub fn shuffle(self, index: $vec) -> $vec {
                unsafe { $shuffle(self.0, index.0).into() }
            }
        }
    }
}

/*----------------------------------------------------------------*/

def_vec! {
    u8x16, __m128i, u8, [u8; 16];
    _mm_loadu_si128,
    _mm_storeu_si128,
    _mm_set1_epi8,
    _mm_and_si128,
    _mm_or_si128,
    _mm_xor_si128
}
impl_conv! {
    u8x16,
    to_u16x8 => u16x8;
    to_u32x4 => u32x4;
    to_u64x2 => u64x2;
}
impl_cmp! {
    u8x16, Mask8x16;
    _mm_cmpeq_epu8_mask,
    _mm_cmpneq_epu8_mask,
    _mm_test_epi8_mask,
    _mm_testn_epi8_mask,
    _mm_movepi8_mask
}
impl_select! {
    u8x16, Mask8x16;
    _mm_maskz_mov_epi8,
    _mm_mask_blend_epi8,
    _mm_maskz_compress_epi8,
    _mm_permutexvar_epi8,
    _mm_shuffle_epi8
}
impl u8x16 {
    #[inline]
    pub fn extract<const INDEX: i32>(self) -> u8 {
        unsafe { _mm_extract_epi8::<INDEX>(self.0) as u8 }
    }

    #[inline]
    pub fn broadcast32(self) -> u8x32 {
        unsafe { _mm256_broadcastsi128_si256(self.0).into() }
    }

    #[inline]
    pub fn broadcast64(self) -> u8x64 {
        unsafe { _mm512_broadcast_i64x2(self.0).into() }
    }

    #[inline]
    pub fn zero_ext(self) -> u16x16 {
        unsafe { _mm256_cvtepu8_epi16(self.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn findset(self, needles: u8x16, needle_count: usize) -> u16 {
        unsafe {
            _mm_extract_epi16::<0>(_mm_cmpestrm::<0>(
                needles.0,
                needle_count as i32,
                self.0,
                16,
            )) as u16
        }
    }
}

def_vec! {
    u16x8, __m128i, u16, [u16; 8];
    _mm_loadu_si128,
    _mm_storeu_si128,
    _mm_set1_epi16,
    _mm_and_si128,
    _mm_or_si128,
    _mm_xor_si128
}
impl_conv! {
    u16x8,
    to_u8x16 => u8x16;
    to_u32x4 => u32x4;
    to_u64x2 => u64x2;
}
impl_cmp! {
    u16x8, Mask16x8;
    _mm_cmpeq_epu16_mask,
    _mm_cmpneq_epu16_mask,
    _mm_test_epi16_mask,
    _mm_testn_epi16_mask,
    _mm_movepi16_mask
}
impl_select! {
    u16x8, Mask16x8;
    _mm_maskz_mov_epi16,
    _mm_mask_blend_epi16,
    _mm_maskz_compress_epi16
}
impl u16x8 {
    #[inline]
    pub fn extract<const INDEX: i32>(self) -> u16 {
        unsafe { _mm_extract_epi16::<INDEX>(self.0) as u16 }
    }

    #[inline]
    pub fn broadcast16(self) -> u16x16 {
        unsafe { _mm256_broadcastsi128_si256(self.0).into() }
    }

    #[inline]
    pub fn broadcast32(self) -> u16x32 {
        unsafe { _mm512_broadcast_i64x2(self.0).into() }
    }

    #[inline]
    pub fn zero_ext(self) -> u32x8 {
        unsafe { _mm256_cvtepu16_epi32(self.0).into() }
    }
}

def_vec! {
    u32x4, __m128i, u32, [u32; 4];
    _mm_loadu_si128,
    _mm_storeu_si128,
    _mm_set1_epi32,
    _mm_and_si128,
    _mm_or_si128,
    _mm_xor_si128
}
impl_conv! {
    u32x4,
    to_u8x16 => u8x16;
    to_u16x8 => u16x8;
    to_u64x2 => u64x2;
}
impl_select! {
    u32x4, Mask32x4;
    _mm_maskz_mov_epi32,
    _mm_mask_blend_epi32,
    _mm_maskz_compress_epi32
}
impl u32x4 {
    #[inline]
    pub fn extract<const INDEX: i32>(self) -> u32 {
        unsafe { _mm_extract_epi32::<INDEX>(self.0) as u32 }
    }

    #[inline]
    pub fn broadcast8(self) -> u32x8 {
        unsafe { _mm256_broadcastsi128_si256(self.0).into() }
    }

    #[inline]
    pub fn broadcast16(self) -> u32x16 {
        unsafe { _mm512_broadcast_i64x2(self.0).into() }
    }

    #[inline]
    pub fn zero_ext(self) -> u64x4 {
        unsafe { _mm256_cvtepu32_epi64(self.0).into() }
    }
}

def_vec! {
    u64x2, __m128i, u64, [u64; 2];
    _mm_loadu_si128,
    _mm_storeu_si128,
    _mm_set1_epi64x,
    _mm_and_si128,
    _mm_or_si128,
    _mm_xor_si128
}
impl_conv! {
    u64x2,
    to_u8x16 => u8x16;
    to_u16x8 => u16x8;
    to_u32x4 => u32x4;
}
impl_select! {
    u64x2, Mask64x2;
    _mm_maskz_mov_epi64,
    _mm_mask_blend_epi64,
    _mm_maskz_compress_epi64
}
impl u64x2 {
    #[inline]
    pub fn extract<const INDEX: i32>(self) -> u64 {
        unsafe { _mm_extract_epi64::<INDEX>(self.0) as u64 }
    }

    #[inline]
    pub fn broadcast4(self) -> u64x4 {
        unsafe { _mm256_broadcastsi128_si256(self.0).into() }
    }

    #[inline]
    pub fn broadcast8(self) -> u64x8 {
        unsafe { _mm512_broadcast_i64x2(self.0).into() }
    }
}


/*----------------------------------------------------------------*/

def_vec! {
    u8x32, __m256i, u8, [u8; 32];
    _mm256_loadu_si256,
    _mm256_storeu_si256,
    _mm256_set1_epi8,
    _mm256_and_si256,
    _mm256_or_si256,
    _mm256_xor_si256
}
impl_conv! {
    u8x32,
    to_u16x16 => u16x16;
    to_u32x8 => u32x8;
    to_u64x4 => u64x4;
}
impl_cmp! {
    u8x32, Mask8x32;
    _mm256_cmpeq_epu8_mask,
    _mm256_cmpneq_epu8_mask,
    _mm256_test_epi8_mask,
    _mm256_testn_epi8_mask,
    _mm256_movepi8_mask
}
impl_select! {
    u8x32, Mask8x32;
    _mm256_maskz_mov_epi8,
    _mm256_mask_blend_epi8,
    _mm256_maskz_compress_epi8,
    _mm256_permutexvar_epi8,
    _mm256_shuffle_epi8
}
impl u8x32 {
    #[inline]
    pub fn extract<const INDEX: i32>(self) -> u8 {
        unsafe { _mm256_extract_epi8::<INDEX>(self.0) as u8 }
    }

    #[inline]
    pub fn extract16<const INDEX: i32>(self) -> u8x16 {
        unsafe { _mm256_extracti128_si256::<INDEX>(self.0).into() }
    }

    #[inline]
    pub fn broadcast64(self) -> u8x64 {
        unsafe { _mm512_broadcast_i64x4(self.0).into() }
    }

    #[inline]
    pub fn zero_ext(self) -> u16x32 {
        unsafe { _mm512_cvtepu8_epi16(self.0).into() }
    }
}

def_vec! {
    u16x16, __m256i, u16, [u16; 16];
    _mm256_loadu_si256,
    _mm256_storeu_si256,
    _mm256_set1_epi16,
    _mm256_and_si256,
    _mm256_or_si256,
    _mm256_xor_si256
}
impl_conv! {
    u16x16,
    to_u8x32 => u8x32;
    to_u32x8 => u32x8;
    to_u64x4 => u64x4;
}
impl_cmp! {
    u16x16, Mask16x16;
    _mm256_cmpeq_epu16_mask,
    _mm256_cmpneq_epu16_mask,
    _mm256_test_epi16_mask,
    _mm256_testn_epi16_mask,
    _mm256_movepi16_mask
}
impl_select! {
    u16x16, Mask16x16;
    _mm256_maskz_mov_epi16,
    _mm256_mask_blend_epi16,
    _mm256_maskz_compress_epi16
}
impl u16x16 {
    #[inline]
    pub fn extract<const INDEX: i32>(self) -> u16 {
        unsafe { _mm256_extract_epi16::<INDEX>(self.0) as u16 }
    }

    #[inline]
    pub fn extract8<const INDEX: i32>(self) -> u16x8 {
        unsafe { _mm256_extracti128_si256::<INDEX>(self.0).into() }
    }

    #[inline]
    pub fn broadcast32(self) -> u16x32 {
        unsafe { _mm512_broadcast_i64x4(self.0).into() }
    }

    #[inline]
    pub fn zero_ext(self) -> u32x16 {
        unsafe { _mm512_cvtepu16_epi32(self.0).into() }
    }
}

def_vec! {
    u32x8, __m256i, u32, [u32; 8];
    _mm256_loadu_si256,
    _mm256_storeu_si256,
    _mm256_set1_epi32,
    _mm256_and_si256,
    _mm256_or_si256,
    _mm256_xor_si256
}
impl_conv! {
    u32x8,
    to_u8x32 => u8x32;
    to_u16x16 => u16x16;
    to_u64x4 => u64x4;
}
impl_cmp! {
    u32x8, Mask32x8;
    _mm256_cmpeq_epu32_mask,
    _mm256_cmpneq_epu32_mask,
    _mm256_test_epi32_mask,
    _mm256_testn_epi32_mask,
    _mm256_movepi32_mask
}
impl_select! {
    u32x8, Mask32x8;
    _mm256_maskz_mov_epi32,
    _mm256_mask_blend_epi32,
    _mm256_maskz_compress_epi32
}
impl u32x8 {
    #[inline]
    pub fn extract<const INDEX: i32>(self) -> u32 {
        unsafe { _mm256_extract_epi32::<INDEX>(self.0) as u32 }
    }

    #[inline]
    pub fn extract4<const INDEX: i32>(self) -> u32x4 {
        unsafe { _mm256_extracti128_si256::<INDEX>(self.0).into() }
    }

    #[inline]
    pub fn broadcast16(self) -> u32x16 {
        unsafe { _mm512_broadcast_i64x4(self.0).into() }
    }

    #[inline]
    pub fn zero_ext(self) -> u64x8 {
        unsafe { _mm512_cvtepu32_epi64(self.0).into() }
    }
}

def_vec! {
    u64x4, __m256i, u64, [u64; 4];
    _mm256_loadu_si256,
    _mm256_storeu_si256,
    _mm256_set1_epi64x,
    _mm256_and_si256,
    _mm256_or_si256,
    _mm256_xor_si256
}
impl_conv! {
    u64x4,
    to_u8x32 => u8x32;
    to_u16x16 => u16x16;
    to_u32x8 => u32x8;
}
impl_select! {
    u64x4, Mask64x4;
    _mm256_maskz_mov_epi64,
    _mm256_mask_blend_epi64,
    _mm256_maskz_compress_epi64
}
impl u64x4 {
    #[inline]
    pub fn extract<const INDEX: i32>(self) -> u64 {
        unsafe { _mm256_extract_epi64::<INDEX>(self.0) as u64 }
    }

    #[inline]
    pub fn extract2<const INDEX: i32>(self) -> u64x2 {
        unsafe { _mm256_extracti128_si256::<INDEX>(self.0).into() }
    }

    #[inline]
    pub fn broadcast8(self) -> u64x8 {
        unsafe { _mm512_broadcast_i64x4(self.0).into() }
    }
}

/*----------------------------------------------------------------*/

def_vec! {
    u8x64, __m512i, u8, [u8; 64];
    _mm512_loadu_si512,
    _mm512_storeu_si512,
    _mm512_set1_epi8,
    _mm512_and_si512,
    _mm512_or_si512,
    _mm512_xor_si512
}
impl_conv! {
    u8x64,
    to_u16x32 => u16x32;
    to_u32x16 => u32x16;
    to_u64x8 => u64x8;
}
impl_cmp! {
    u8x64, Mask8x64;
    _mm512_cmpeq_epu8_mask,
    _mm512_cmpneq_epu8_mask,
    _mm512_test_epi8_mask,
    _mm512_testn_epi8_mask,
    _mm512_movepi8_mask
}
impl_select! {
    u8x64, Mask8x64;
    _mm512_maskz_mov_epi8,
    _mm512_mask_blend_epi8,
    _mm512_maskz_compress_epi8,
    _mm512_permutexvar_epi8,
    _mm512_shuffle_epi8
}
impl u8x64 {
    #[inline]
    pub fn extract16<const INDEX: i32>(self) -> u8x16 {
        unsafe { _mm512_extracti64x2_epi64::<INDEX>(self.0).into() }
    }

    #[inline]
    pub fn extract32<const INDEX: i32>(self) -> u8x32 {
        unsafe { _mm512_extracti64x4_epi64::<INDEX>(self.0).into() }
    }

    #[inline]
    pub fn zero_ext(self) -> u16x64 {
        let lo = self.extract32::<0>().zero_ext();
        let hi = self.extract32::<1>().zero_ext();
        u16x64([lo, hi])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn flip_rays(self) -> u8x64 {
        unsafe { _mm512_shuffle_i32x4::<0x4E>(self.0, self.0).into() }
    }

    #[inline]
    pub fn extend_rays(self) -> u8x64 {
        let temp = u8x64::gfni_affine(u64x8::splat(0x0102040810204080).to_u8x64(), self);
        u8x64::gfni_affine(u64x8::splat(0xFFFFFFFFFFFFFFFF).to_u8x64(), temp)
    }

    #[inline]
    pub fn gfni_affine(a: u8x64, b: u8x64) -> u8x64 {
        unsafe { _mm512_gf2p8affine_epi64_epi8::<0>(a.0, b.0).into() }
    }
}

def_vec! {
    u16x32, __m512i, u16, [u16; 32];
    _mm512_loadu_si512,
    _mm512_storeu_si512,
    _mm512_set1_epi16,
    _mm512_and_si512,
    _mm512_or_si512,
    _mm512_xor_si512
}
impl_conv! {
    u16x32,
    to_u8x64 => u8x64;
    to_u32x16 => u32x16;
    to_u64x8 => u64x8;
}
impl_cmp! {
    u16x32, Mask16x32;
    _mm512_cmpeq_epu16_mask,
    _mm512_cmpneq_epu16_mask,
    _mm512_test_epi16_mask,
    _mm512_testn_epi16_mask,
    _mm512_movepi16_mask
}
impl_select! {
    u16x32, Mask16x32;
    _mm512_maskz_mov_epi16,
    _mm512_mask_blend_epi16,
    _mm512_maskz_compress_epi16
}
impl u16x32 {
    #[inline]
    pub fn extract8<const INDEX: i32>(self) -> u16x8 {
        unsafe { _mm512_extracti64x2_epi64::<INDEX>(self.0).into() }
    }

    #[inline]
    pub fn extract16<const INDEX: i32>(self) -> u16x16 {
        unsafe { _mm512_extracti64x4_epi64::<INDEX>(self.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl<const SHIFT: u32>(self) -> u16x32 {
        unsafe { _mm512_slli_epi16::<SHIFT>(self.0).into() }
    }

    #[inline]
    pub fn shlv(self, shift: u16x32) -> u16x32 {
        unsafe { _mm512_sllv_epi16(self.0, shift.0).into() }
    }

    #[inline]
    pub fn shr<const SHIFT: u32>(self) -> u16x32 {
        unsafe { _mm512_srli_epi16::<SHIFT>(self.0).into() }
    }

    #[inline]
    pub fn shrv(self, shift: u16x32) -> u16x32 {
        unsafe { _mm512_srlv_epi16(self.0, shift.0).into() }
    }
}

def_vec! {
    u32x16, __m512i, u32, [u32; 16];
    _mm512_loadu_si512,
    _mm512_storeu_si512,
    _mm512_set1_epi32,
    _mm512_and_si512,
    _mm512_or_si512,
    _mm512_xor_si512
}
impl_conv! {
    u32x16,
    to_u8x64 => u8x64;
    to_u16x32 => u16x32;
    to_u64x8 => u64x8;
}
impl_cmp! {
    u32x16, Mask32x16;
    _mm512_cmpeq_epu32_mask,
    _mm512_cmpneq_epu32_mask,
    _mm512_test_epi32_mask,
    _mm512_testn_epi32_mask,
    _mm512_movepi32_mask
}
impl_select! {
    u32x16, Mask32x16;
    _mm512_maskz_mov_epi32,
    _mm512_mask_blend_epi32,
    _mm512_maskz_compress_epi32
}

def_vec! {
    u64x8, __m512i, u64, [u64; 8];
    _mm512_loadu_si512,
    _mm512_storeu_si512,
    _mm512_set1_epi64,
    _mm512_and_si512,
    _mm512_or_si512,
    _mm512_xor_si512
}
impl_conv! {
    u64x8,
    to_u8x64 => u8x64;
    to_u16x32 => u16x32;
    to_u32x16 => u32x16;
}
impl_cmp! {
    u64x8, Mask64x8;
    _mm512_cmpeq_epu64_mask,
    _mm512_cmpneq_epu64_mask,
    _mm512_test_epi64_mask,
    _mm512_testn_epi64_mask,
    _mm512_movepi64_mask
}
impl_select! {
    u64x8, Mask64x8;
    _mm512_maskz_mov_epi64,
    _mm512_mask_blend_epi64,
    _mm512_maskz_compress_epi64
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u16x64([u16x32; 2]);
impl u16x64 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u16x64 {
        unsafe {
            let lo = u16x32::load(src);
            let hi = u16x32::load(src.byte_add(64));
            u16x64([lo, hi])
        }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            self.0[0].store(dest);
            self.0[1].store(dest.byte_add(64));
        }
    }

    #[inline]
    pub fn splat(value: u16) -> u16x64 {
        let half = u16x32::splat(value);
        u16x64([half; 2])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn extract16<const INDEX: usize>(self) -> u16x16 {
        match INDEX {
            0 => self.0[0].extract16::<0>(),
            1 => self.0[0].extract16::<1>(),
            2 => self.0[1].extract16::<0>(),
            3 => self.0[1].extract16::<1>(),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extract32<const INDEX: usize>(self) -> u16x32 {
        self.0[INDEX]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl<const SHIFT: u32>(self) -> u16x64 {
        let lo = self.0[0].shl::<SHIFT>();
        let hi = self.0[1].shl::<SHIFT>();
        u16x64([lo, hi])
    }

    #[inline]
    pub fn shlv(self, shift: u16x64) -> u16x64 {
        let lo = self.0[0].shlv(shift.0[0]);
        let hi = self.0[1].shlv(shift.0[1]);
        u16x64([lo, hi])
    }

    #[inline]
    pub fn shr<const SHIFT: u32>(self) -> u16x64 {
        let lo = self.0[0].shr::<SHIFT>();
        let hi = self.0[1].shr::<SHIFT>();
        u16x64([lo, hi])
    }

    #[inline]
    pub fn shrv(self, shift: u16x64) -> u16x64 {
        let lo = self.0[0].shrv(shift.0[0]);
        let hi = self.0[1].shrv(shift.0[1]);
        u16x64([lo, hi])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask(self, mask: Mask16x64) -> u16x64 {
        let lo = self.0[0].mask(Mask16x32::from(mask.0 as u32));
        let hi = self.0[1].mask(Mask16x32::from((mask.0 >> 32) as u32));
        u16x64([lo, hi])
    }

    #[inline]
    pub fn blend(a: u16x64, b: u16x64, mask: Mask16x64) -> u16x64 {
        let lo = u16x32::blend(a.0[0], b.0[0], Mask16x32::from(mask.0 as u32));
        let hi = u16x32::blend(a.0[1], b.0[1], Mask16x32::from((mask.0 >> 32) as u32));
        u16x64([lo, hi])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq(a: u16x64, b: u16x64) -> Mask16x64 {
        let lo = u16x32::eq(a.0[0], b.0[0]);
        let hi = u16x32::eq(a.0[1], b.0[1]);
        lo.concat(hi)
    }

    #[inline]
    pub fn neq(a: u16x64, b: u16x64) -> Mask16x64 {
        let lo = u16x32::neq(a.0[0], b.0[0]);
        let hi = u16x32::neq(a.0[1], b.0[1]);
        lo.concat(hi)
    }

    #[inline]
    pub fn test(a: u16x64, b: u16x64) -> Mask16x64 {
        let lo = u16x32::test(a.0[0], b.0[0]);
        let hi = u16x32::test(a.0[1], b.0[1]);
        lo.concat(hi)
    }

    #[inline]
    pub fn testn(a: u16x64, b: u16x64) -> Mask16x64 {
        let lo = u16x32::testn(a.0[0], b.0[0]);
        let hi = u16x32::testn(a.0[1], b.0[1]);
        lo.concat(hi)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero(self) -> Mask16x64 {
        u16x64::eq(self, u16x64::splat(0))
    }

    #[inline]
    pub fn nonzero(self) -> Mask16x64 {
        u16x64::neq(self, u16x64::splat(0))
    }

    #[inline]
    pub fn msb(self) -> Mask16x64 {
        let lo = self.0[0].msb();
        let hi = self.0[1].msb();
        lo.concat(hi)
    }
}

impl From<[u16; 64]> for u16x64 {
    #[inline]
    fn from(arr: [u16; 64]) -> Self {
        unsafe { u16x64::load(arr.as_ptr()) }
    }
}
impl BitAnd for u16x64 {
    type Output = Self;

    #[inline]
    fn bitand(self, other: Self) -> Self {
        let lo = self.0[0] & other.0[0];
        let hi = self.0[1] & other.0[1];
        u16x64([lo, hi])
    }
}
impl BitOr for u16x64 {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        let lo = self.0[0] | other.0[0];
        let hi = self.0[1] | other.0[1];
        u16x64([lo, hi])
    }
}
impl BitXor for u16x64 {
    type Output = Self;

    #[inline]
    fn bitxor(self, other: Self) -> Self {
        let lo = self.0[0] ^ other.0[0];
        let hi = self.0[1] ^ other.0[1];
        u16x64([lo, hi])
    }
}
impl BitAndAssign for u16x64 {
    #[inline]
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
    }
}
impl BitOrAssign for u16x64 {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}
impl BitXorAssign for u16x64 {
    #[inline]
    fn bitxor_assign(&mut self, other: Self) {
        *self = *self ^ other;
    }
}

/*----------------------------------------------------------------*/

def_vec! {
    i16x32, __m512i, i16, [i16; 32];
    _mm512_loadu_si512,
    _mm512_storeu_si512,
    _mm512_set1_epi16,
    _mm512_and_si512,
    _mm512_or_si512,
    _mm512_xor_si512
}
impl i16x32 {
    #[inline]
    pub fn clamp(self, min: i16x32, max: i16x32) -> i16x32 {
        unsafe { _mm512_min_epi16(_mm512_max_epi16(self.0, min.0), max.0).into() }
    }

    #[inline]
    pub fn madd(self, rhs: i16x32) -> i32x16 {
        unsafe { _mm512_madd_epi16(self.0, rhs.0).into() }
    }
}
impl Add for i16x32 {
    type Output = i16x32;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_add_epi16(self.0, rhs.0).into() }
    }
}
impl Sub for i16x32 {
    type Output = i16x32;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_sub_epi16(self.0, rhs.0).into() }
    }
}
impl Mul for i16x32 {
    type Output = i16x32;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_mullo_epi16(self.0, rhs.0).into() }
    }
}
impl AddAssign for i16x32 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl SubAssign for i16x32 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

/*----------------------------------------------------------------*/

def_vec! {
    i32x16, __m512i, i32, [i32; 16];
    _mm512_loadu_si512,
    _mm512_storeu_si512,
    _mm512_set1_epi32,
    _mm512_and_si512,
    _mm512_or_si512,
    _mm512_xor_si512
}
impl i32x16 {
    #[inline]
    pub fn reduce_sum(self) -> i32 {
        unsafe { _mm512_reduce_add_epi32(self.0) }
    }
}
impl Add for i32x16 {
    type Output = i32x16;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_add_epi32(self.0, rhs.0).into() }
    }
}
impl Sub for i32x16 {
    type Output = i32x16;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_sub_epi32(self.0, rhs.0).into() }
    }
}
impl AddAssign for i32x16 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl SubAssign for i32x16 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}