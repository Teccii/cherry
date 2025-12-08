use core::{arch::x86_64::*, ops::*};

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Mask8(pub __mmask8);

impl Mask8 {
    #[inline]
    pub fn from_bitmask(bitmask: u8) -> Mask8 {
        Mask8(bitmask)
    }

    #[inline]
    pub fn to_bitmask(self) -> u8 {
        self.0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn concat(self, other: Mask8) -> Mask16 {
        Mask16(self.0 as u16 | ((other.0 as u16) << 8))
    }
}

impl From<__mmask8> for Mask8 {
    #[inline]
    fn from(raw: __mmask8) -> Self {
        Mask8(raw)
    }
}

impl Not for Mask8 {
    type Output = Mask8;

    #[inline]
    fn not(self) -> Self::Output {
        Mask8(!self.0)
    }
}

impl BitAnd for Mask8 {
    type Output = Mask8;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Mask8(self.0 & rhs.0)
    }
}

impl BitOr for Mask8 {
    type Output = Mask8;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Mask8(self.0 | rhs.0)
    }
}

impl BitXor for Mask8 {
    type Output = Mask8;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Mask8(self.0 ^ rhs.0)
    }
}

impl BitAnd<__mmask8> for Mask8 {
    type Output = Mask8;

    #[inline]
    fn bitand(self, rhs: __mmask8) -> Self::Output {
        Mask8(self.0 & rhs)
    }
}

impl BitOr<__mmask8> for Mask8 {
    type Output = Mask8;

    #[inline]
    fn bitor(self, rhs: __mmask8) -> Self::Output {
        Mask8(self.0 | rhs)
    }
}

impl BitXor<__mmask8> for Mask8 {
    type Output = Mask8;

    #[inline]
    fn bitxor(self, rhs: __mmask8) -> Self::Output {
        Mask8(self.0 ^ rhs)
    }
}

impl BitAndAssign for Mask8 {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOrAssign for Mask8 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXorAssign for Mask8 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl BitAndAssign<__mmask8> for Mask8 {
    #[inline]
    fn bitand_assign(&mut self, rhs: __mmask8) {
        self.0 &= rhs;
    }
}

impl BitOrAssign<__mmask8> for Mask8 {
    #[inline]
    fn bitor_assign(&mut self, rhs: __mmask8) {
        self.0 |= rhs;
    }
}

impl BitXorAssign<__mmask8> for Mask8 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: __mmask8) {
        self.0 ^= rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Mask16(pub __mmask16);

impl Mask16 {
    #[inline]
    pub fn from_bitmask(bitmask: u16) -> Mask16 {
        Mask16(bitmask)
    }

    #[inline]
    pub fn to_bitmask(self) -> u16 {
        self.0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn concat(self, other: Mask16) -> Mask32 {
        Mask32(self.0 as u32 | ((other.0 as u32) << 16))
    }
}

impl From<__mmask16> for Mask16 {
    #[inline]
    fn from(raw: __mmask16) -> Self {
        Mask16(raw)
    }
}

impl Not for Mask16 {
    type Output = Mask16;

    #[inline]
    fn not(self) -> Self::Output {
        Mask16(!self.0)
    }
}

impl BitAnd for Mask16 {
    type Output = Mask16;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Mask16(self.0 & rhs.0)
    }
}

impl BitOr for Mask16 {
    type Output = Mask16;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Mask16(self.0 | rhs.0)
    }
}

impl BitXor for Mask16 {
    type Output = Mask16;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Mask16(self.0 ^ rhs.0)
    }
}

impl BitAnd<__mmask16> for Mask16 {
    type Output = Mask16;

    #[inline]
    fn bitand(self, rhs: __mmask16) -> Self::Output {
        Mask16(self.0 & rhs)
    }
}

impl BitOr<__mmask16> for Mask16 {
    type Output = Mask16;

    #[inline]
    fn bitor(self, rhs: __mmask16) -> Self::Output {
        Mask16(self.0 | rhs)
    }
}

impl BitXor<__mmask16> for Mask16 {
    type Output = Mask16;

    #[inline]
    fn bitxor(self, rhs: __mmask16) -> Self::Output {
        Mask16(self.0 ^ rhs)
    }
}

impl BitAndAssign for Mask16 {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOrAssign for Mask16 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXorAssign for Mask16 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl BitAndAssign<__mmask16> for Mask16 {
    #[inline]
    fn bitand_assign(&mut self, rhs: __mmask16) {
        self.0 &= rhs;
    }
}

impl BitOrAssign<__mmask16> for Mask16 {
    #[inline]
    fn bitor_assign(&mut self, rhs: __mmask16) {
        self.0 |= rhs;
    }
}

impl BitXorAssign<__mmask16> for Mask16 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: __mmask16) {
        self.0 ^= rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Mask32(pub __mmask32);

impl Mask32 {
    #[inline]
    pub fn from_bitmask(bitmask: u32) -> Mask32 {
        Mask32(bitmask)
    }

    #[inline]
    pub fn to_bitmask(self) -> u32 {
        self.0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn concat(self, other: Mask32) -> Mask64 {
        Mask64(self.0 as u64 | ((other.0 as u64) << 32))
    }
}

impl From<__mmask32> for Mask32 {
    #[inline]
    fn from(raw: __mmask32) -> Self {
        Mask32(raw)
    }
}

impl Not for Mask32 {
    type Output = Mask32;

    #[inline]
    fn not(self) -> Self::Output {
        Mask32(!self.0)
    }
}

impl BitAnd for Mask32 {
    type Output = Mask32;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Mask32(self.0 & rhs.0)
    }
}

impl BitOr for Mask32 {
    type Output = Mask32;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Mask32(self.0 | rhs.0)
    }
}

impl BitXor for Mask32 {
    type Output = Mask32;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Mask32(self.0 ^ rhs.0)
    }
}

impl BitAnd<__mmask32> for Mask32 {
    type Output = Mask32;

    #[inline]
    fn bitand(self, rhs: __mmask32) -> Self::Output {
        Mask32(self.0 & rhs)
    }
}

impl BitOr<__mmask32> for Mask32 {
    type Output = Mask32;

    #[inline]
    fn bitor(self, rhs: __mmask32) -> Self::Output {
        Mask32(self.0 | rhs)
    }
}

impl BitXor<__mmask32> for Mask32 {
    type Output = Mask32;

    #[inline]
    fn bitxor(self, rhs: __mmask32) -> Self::Output {
        Mask32(self.0 ^ rhs)
    }
}

impl BitAndAssign for Mask32 {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOrAssign for Mask32 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXorAssign for Mask32 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl BitAndAssign<__mmask32> for Mask32 {
    #[inline]
    fn bitand_assign(&mut self, rhs: __mmask32) {
        self.0 &= rhs;
    }
}

impl BitOrAssign<__mmask32> for Mask32 {
    #[inline]
    fn bitor_assign(&mut self, rhs: __mmask32) {
        self.0 |= rhs;
    }
}

impl BitXorAssign<__mmask32> for Mask32 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: __mmask32) {
        self.0 ^= rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Mask64(pub __mmask64);

impl Mask64 {
    #[inline]
    pub fn from_bitmask(bitmask: u64) -> Mask64 {
        Mask64(bitmask)
    }

    #[inline]
    pub fn to_bitmask(self) -> u64 {
        self.0
    }
}

impl From<__mmask64> for Mask64 {
    #[inline]
    fn from(raw: __mmask64) -> Self {
        Mask64(raw)
    }
}

impl Not for Mask64 {
    type Output = Mask64;

    #[inline]
    fn not(self) -> Self::Output {
        Mask64(!self.0)
    }
}

impl BitAnd for Mask64 {
    type Output = Mask64;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Mask64(self.0 & rhs.0)
    }
}

impl BitOr for Mask64 {
    type Output = Mask64;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Mask64(self.0 | rhs.0)
    }
}

impl BitXor for Mask64 {
    type Output = Mask64;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Mask64(self.0 ^ rhs.0)
    }
}

impl BitAnd<__mmask64> for Mask64 {
    type Output = Mask64;

    #[inline]
    fn bitand(self, rhs: __mmask64) -> Self::Output {
        Mask64(self.0 & rhs)
    }
}

impl BitOr<__mmask64> for Mask64 {
    type Output = Mask64;

    #[inline]
    fn bitor(self, rhs: __mmask64) -> Self::Output {
        Mask64(self.0 | rhs)
    }
}

impl BitXor<__mmask64> for Mask64 {
    type Output = Mask64;

    #[inline]
    fn bitxor(self, rhs: __mmask64) -> Self::Output {
        Mask64(self.0 ^ rhs)
    }
}

impl BitAndAssign for Mask64 {
    #[inline]
    fn bitand_assign(&mut self, rhs: Mask64) {
        self.0 &= rhs.0;
    }
}

impl BitOrAssign for Mask64 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Mask64) {
        self.0 |= rhs.0;
    }
}

impl BitXorAssign for Mask64 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Mask64) {
        self.0 ^= rhs.0;
    }
}

impl BitAndAssign<__mmask64> for Mask64 {
    #[inline]
    fn bitand_assign(&mut self, rhs: __mmask64) {
        self.0 &= rhs;
    }
}

impl BitOrAssign<__mmask64> for Mask64 {
    #[inline]
    fn bitor_assign(&mut self, rhs: __mmask64) {
        self.0 |= rhs;
    }
}

impl BitXorAssign<__mmask64> for Mask64 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: __mmask64) {
        self.0 ^= rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u32x4(pub __m128i);

impl u32x4 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u32x4 {
        unsafe { _mm_loadu_si128(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            _mm_storeu_si128(dest.cast(), self.0);
        }
    }

    #[inline]
    pub fn splat(value: u32) -> u32x4 {
        unsafe { _mm_set1_epi32(value as i32).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn to_u8x16(self) -> u8x16 {
        u8x16(self.0)
    }

    #[inline]
    pub fn to_u16x8(self) -> u16x8 {
        u16x8(self.0)
    }
}

impl From<__m128i> for u32x4 {
    #[inline]
    fn from(raw: __m128i) -> Self {
        u32x4(raw)
    }
}

impl From<[u32; 4]> for u32x4 {
    #[inline]
    fn from(arr: [u32; 4]) -> Self {
        unsafe { u32x4::load(arr.as_ptr()) }
    }
}

impl BitAnd for u32x4 {
    type Output = u32x4;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm_and_si128(self.0, rhs.0).into() }
    }
}

impl BitOr for u32x4 {
    type Output = u32x4;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm_or_si128(self.0, rhs.0).into() }
    }
}

impl BitXor for u32x4 {
    type Output = u32x4;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { _mm_xor_si128(self.0, rhs.0).into() }
    }
}

impl BitAndAssign for u32x4 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u32x4) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u32x4 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u32x4) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u32x4 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u32x4) {
        *self = *self ^ rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u16x8(pub __m128i);

impl u16x8 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u16x8 {
        unsafe { _mm_loadu_si128(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            _mm_storeu_si128(dest.cast(), self.0);
        }
    }

    #[inline]
    pub fn splat(value: u16) -> u16x8 {
        unsafe { _mm_set1_epi16(value as i16).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn to_u8x16(self) -> u8x16 {
        u8x16(self.0)
    }

    #[inline]
    pub fn to_u32x4(self) -> u32x4 {
        u32x4(self.0)
    }
}

impl From<__m128i> for u16x8 {
    #[inline]
    fn from(raw: __m128i) -> Self {
        u16x8(raw)
    }
}

impl From<[u16; 8]> for u16x8 {
    #[inline]
    fn from(arr: [u16; 8]) -> Self {
        unsafe { u16x8::load(arr.as_ptr()) }
    }
}

impl BitAnd for u16x8 {
    type Output = u16x8;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm_and_si128(self.0, rhs.0).into() }
    }
}

impl BitOr for u16x8 {
    type Output = u16x8;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm_or_si128(self.0, rhs.0).into() }
    }
}

impl BitXor for u16x8 {
    type Output = u16x8;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { _mm_xor_si128(self.0, rhs.0).into() }
    }
}

impl BitAndAssign for u16x8 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u16x8) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u16x8 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u16x8) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u16x8 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u16x8) {
        *self = *self ^ rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u8x16(pub __m128i);

impl u8x16 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u8x16 {
        unsafe { _mm_loadu_si128(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            _mm_storeu_si128(dest.cast(), self.0);
        }
    }

    #[inline]
    pub fn splat(value: u8) -> u8x16 {
        unsafe { _mm_set1_epi8(value as i8).into() }
    }

    /*----------------------------------------------------------------*/

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

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask(self, mask: Mask16) -> u8x16 {
        unsafe { _mm_maskz_mov_epi8(mask.to_bitmask(), self.0).into() }
    }

    #[inline]
    pub fn blend(a: u8x16, b: u8x16, mask: Mask16) -> u8x16 {
        unsafe { _mm_mask_blend_epi8(mask.to_bitmask(), a.0, b.0).into() }
    }

    #[inline]
    pub fn compress(self, mask: Mask16) -> u8x16 {
        unsafe { _mm_maskz_compress_epi8(mask.to_bitmask(), self.0).into() }
    }

    #[inline]
    pub fn permute(self, index: u8x16) -> u8x16 {
        unsafe { _mm_permutexvar_epi8(index.0, self.0).into() }
    }

    #[inline]
    pub fn shuffle(self, index: u8x16) -> u8x16 {
        unsafe { _mm_shuffle_epi8(self.0, index.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq(a: u8x16, b: u8x16) -> Mask16 {
        unsafe { _mm_cmpeq_epu8_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn neq(a: u8x16, b: u8x16) -> Mask16 {
        unsafe { _mm_cmpneq_epu8_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn test(a: u8x16, b: u8x16) -> Mask16 {
        unsafe { _mm_test_epi8_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn testn(a: u8x16, b: u8x16) -> Mask16 {
        unsafe { _mm_testn_epi8_mask(a.0, b.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero(self) -> Mask16 {
        u8x16::eq(self, u8x16::splat(0))
    }

    #[inline]
    pub fn nonzero(self) -> Mask16 {
        u8x16::neq(self, u8x16::splat(0))
    }

    #[inline]
    pub fn msb(self) -> Mask16 {
        unsafe { _mm_movepi8_mask(self.0).into() }
    }
}

impl From<__m128i> for u8x16 {
    #[inline]
    fn from(raw: __m128i) -> Self {
        u8x16(raw)
    }
}

impl From<[u8; 16]> for u8x16 {
    #[inline]
    fn from(arr: [u8; 16]) -> Self {
        unsafe { u8x16::load(arr.as_ptr()) }
    }
}

impl BitAnd for u8x16 {
    type Output = u8x16;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm_and_si128(self.0, rhs.0).into() }
    }
}

impl BitOr for u8x16 {
    type Output = u8x16;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm_or_si128(self.0, rhs.0).into() }
    }
}

impl BitXor for u8x16 {
    type Output = u8x16;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { _mm_xor_si128(self.0, rhs.0).into() }
    }
}

impl BitAndAssign for u8x16 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u8x16) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u8x16 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u8x16) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u8x16 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u8x16) {
        *self = *self ^ rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u16x16(pub __m256i);

impl u16x16 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u16x16 {
        unsafe { _mm256_loadu_si256(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            _mm256_storeu_si256(dest.cast(), self.0);
        }
    }

    #[inline]
    pub fn splat(value: u16) -> u16x16 {
        unsafe { _mm256_set1_epi16(value as i16).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub unsafe fn compress_store<T>(self, mask: Mask16, dest: *mut T) {
        unsafe { _mm256_mask_compressstoreu_epi16(dest.cast(), mask.to_bitmask(), self.0) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq(a: u16x16, b: u16x16) -> Mask16 {
        unsafe { _mm256_cmpeq_epu16_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn neq(a: u16x16, b: u16x16) -> Mask16 {
        unsafe { _mm256_cmpneq_epu16_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn test(a: u16x16, b: u16x16) -> Mask16 {
        unsafe { _mm256_test_epi16_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn testn(a: u16x16, b: u16x16) -> Mask16 {
        unsafe { _mm256_testn_epi16_mask(a.0, b.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero(self) -> Mask16 {
        u16x16::eq(self, u16x16::splat(0))
    }

    #[inline]
    pub fn nonzero(self) -> Mask16 {
        u16x16::neq(self, u16x16::splat(0))
    }

    #[inline]
    pub fn msb(self) -> Mask16 {
        unsafe { _mm256_movepi16_mask(self.0).into() }
    }
}

impl From<__m256i> for u16x16 {
    #[inline]
    fn from(raw: __m256i) -> Self {
        u16x16(raw)
    }
}

impl From<[u16; 16]> for u16x16 {
    #[inline]
    fn from(arr: [u16; 16]) -> Self {
        unsafe { u16x16::load(arr.as_ptr()) }
    }
}

impl BitAnd for u16x16 {
    type Output = u16x16;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_and_si256(self.0, rhs.0).into() }
    }
}

impl BitOr for u16x16 {
    type Output = u16x16;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_or_si256(self.0, rhs.0).into() }
    }
}

impl BitXor for u16x16 {
    type Output = u16x16;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_xor_si256(self.0, rhs.0).into() }
    }
}

impl BitAndAssign for u16x16 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u16x16) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u16x16 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u16x16) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u16x16 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u16x16) {
        *self = *self ^ rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u8x32(pub __m256i);

impl u8x32 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u8x32 {
        unsafe { _mm256_loadu_si256(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            _mm256_storeu_si256(dest.cast(), self.0);
        }
    }

    #[inline]
    pub fn splat(value: u8) -> u8x32 {
        unsafe { _mm256_set1_epi8(value as i8).into() }
    }

    /*----------------------------------------------------------------*/

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

    #[inline]
    pub fn mask(self, mask: Mask32) -> u8x32 {
        unsafe { _mm256_maskz_mov_epi8(mask.to_bitmask(), self.0).into() }
    }

    #[inline]
    pub fn blend(a: u8x32, b: u8x32, mask: Mask32) -> u8x32 {
        unsafe { _mm256_mask_blend_epi8(mask.to_bitmask(), a.0, b.0).into() }
    }

    #[inline]
    pub fn compress(self, mask: Mask32) -> u8x32 {
        unsafe { _mm256_maskz_compress_epi8(mask.to_bitmask(), self.0).into() }
    }

    #[inline]
    pub fn permute(self, index: u8x32) -> u8x32 {
        unsafe { _mm256_permutexvar_epi8(index.0, self.0).into() }
    }

    #[inline]
    pub fn shuffle(self, index: u8x32) -> u8x32 {
        unsafe { _mm256_shuffle_epi8(self.0, index.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq(a: u8x32, b: u8x32) -> Mask32 {
        unsafe { _mm256_cmpeq_epu8_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn neq(a: u8x32, b: u8x32) -> Mask32 {
        unsafe { _mm256_cmpneq_epu8_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn test(a: u8x32, b: u8x32) -> Mask32 {
        unsafe { _mm256_test_epi8_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn testn(a: u8x32, b: u8x32) -> Mask32 {
        unsafe { _mm256_testn_epi8_mask(a.0, b.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero(self) -> Mask32 {
        u8x32::eq(self, u8x32::splat(0))
    }

    #[inline]
    pub fn nonzero(self) -> Mask32 {
        u8x32::neq(self, u8x32::splat(0))
    }

    #[inline]
    pub fn msb(self) -> Mask32 {
        unsafe { _mm256_movepi8_mask(self.0).into() }
    }
}

impl From<__m256i> for u8x32 {
    #[inline]
    fn from(raw: __m256i) -> Self {
        u8x32(raw)
    }
}

impl From<[u8; 32]> for u8x32 {
    #[inline]
    fn from(arr: [u8; 32]) -> Self {
        unsafe { u8x32::load(arr.as_ptr()) }
    }
}

impl BitAnd for u8x32 {
    type Output = u8x32;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_and_si256(self.0, rhs.0).into() }
    }
}

impl BitOr for u8x32 {
    type Output = u8x32;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_or_si256(self.0, rhs.0).into() }
    }
}

impl BitXor for u8x32 {
    type Output = u8x32;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_xor_si256(self.0, rhs.0).into() }
    }
}

impl BitAndAssign for u8x32 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u8x32) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u8x32 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u8x32) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u8x32 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u8x32) {
        *self = *self ^ rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u64x8(pub __m512i);

impl u64x8 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u64x8 {
        unsafe { _mm512_loadu_si512(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            _mm512_storeu_si512(dest.cast(), self.0);
        }
    }

    #[inline]
    pub fn splat(value: u64) -> u64x8 {
        unsafe { _mm512_set1_epi64(value as i64).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn to_u8x64(self) -> u8x64 {
        u8x64(self.0)
    }

    #[inline]
    pub fn to_u16x32(self) -> u16x32 {
        u16x32(self.0)
    }

    #[inline]
    pub fn to_u32x16(self) -> u32x16 {
        u32x16(self.0)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub unsafe fn compress_store<T>(self, mask: Mask8, dest: *mut T) {
        unsafe { _mm512_mask_compressstoreu_epi64(dest.cast(), mask.to_bitmask(), self.0) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq(a: u64x8, b: u64x8) -> Mask8 {
        unsafe { _mm512_cmpeq_epu64_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn neq(a: u64x8, b: u64x8) -> Mask8 {
        unsafe { _mm512_cmpneq_epu64_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn test(a: u64x8, b: u64x8) -> Mask8 {
        unsafe { _mm512_test_epi64_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn testn(a: u64x8, b: u64x8) -> Mask8 {
        unsafe { _mm512_testn_epi64_mask(a.0, b.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero(self) -> Mask8 {
        u64x8::eq(self, u64x8::splat(0))
    }

    #[inline]
    pub fn nonzero(self) -> Mask8 {
        u64x8::neq(self, u64x8::splat(0))
    }

    #[inline]
    pub fn msb(self) -> Mask8 {
        unsafe { _mm512_movepi64_mask(self.0).into() }
    }
}

impl From<__m512i> for u64x8 {
    #[inline]
    fn from(raw: __m512i) -> Self {
        u64x8(raw)
    }
}

impl From<[u64; 8]> for u64x8 {
    #[inline]
    fn from(arr: [u64; 8]) -> Self {
        unsafe { u64x8::load(arr.as_ptr()) }
    }
}

impl BitAnd for u64x8 {
    type Output = u64x8;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_and_si512(self.0, rhs.0).into() }
    }
}

impl BitOr for u64x8 {
    type Output = u64x8;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_or_si512(self.0, rhs.0).into() }
    }
}

impl BitXor for u64x8 {
    type Output = u64x8;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_xor_si512(self.0, rhs.0).into() }
    }
}

impl BitAndAssign for u64x8 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u64x8) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u64x8 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u64x8) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u64x8 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u64x8) {
        *self = *self ^ rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u32x16(pub __m512i);

impl u32x16 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u32x16 {
        unsafe { _mm512_loadu_si512(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            _mm512_storeu_si512(dest.cast(), self.0);
        }
    }

    #[inline]
    pub fn splat(value: u32) -> u32x16 {
        unsafe { _mm512_set1_epi32(value as i32).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn to_u8x64(self) -> u8x64 {
        u8x64(self.0)
    }

    #[inline]
    pub fn to_u16x32(self) -> u16x32 {
        u16x32(self.0)
    }

    #[inline]
    pub fn to_u64x8(self) -> u64x8 {
        u64x8(self.0)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq(a: u32x16, b: u32x16) -> Mask16 {
        unsafe { _mm512_cmpeq_epu32_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn neq(a: u32x16, b: u32x16) -> Mask16 {
        unsafe { _mm512_cmpneq_epu32_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn test(a: u32x16, b: u32x16) -> Mask16 {
        unsafe { _mm512_test_epi32_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn testn(a: u32x16, b: u32x16) -> Mask16 {
        unsafe { _mm512_testn_epi32_mask(a.0, b.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero(self) -> Mask16 {
        u32x16::eq(self, u32x16::splat(0))
    }

    #[inline]
    pub fn nonzero(self) -> Mask16 {
        u32x16::neq(self, u32x16::splat(0))
    }

    #[inline]
    pub fn msb(self) -> Mask16 {
        unsafe { _mm512_movepi32_mask(self.0).into() }
    }
}

impl From<__m512i> for u32x16 {
    #[inline]
    fn from(raw: __m512i) -> Self {
        u32x16(raw)
    }
}

impl From<[u32; 16]> for u32x16 {
    #[inline]
    fn from(arr: [u32; 16]) -> Self {
        unsafe { u32x16::load(arr.as_ptr()) }
    }
}

impl BitAnd for u32x16 {
    type Output = u32x16;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_and_si512(self.0, rhs.0).into() }
    }
}

impl BitOr for u32x16 {
    type Output = u32x16;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_or_si512(self.0, rhs.0).into() }
    }
}

impl BitXor for u32x16 {
    type Output = u32x16;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_xor_si512(self.0, rhs.0).into() }
    }
}

impl BitAndAssign for u32x16 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u32x16) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u32x16 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u32x16) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u32x16 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u32x16) {
        *self = *self ^ rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u16x32(pub __m512i);

impl u16x32 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u16x32 {
        unsafe { _mm512_loadu_si512(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            _mm512_storeu_si512(dest.cast(), self.0);
        }
    }

    #[inline]
    pub fn splat(value: u16) -> u16x32 {
        unsafe { _mm512_set1_epi16(value as i16).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn to_u8x64(self) -> u8x64 {
        u8x64(self.0)
    }

    #[inline]
    pub fn to_u32x16(self) -> u32x16 {
        u32x16(self.0)
    }

    #[inline]
    pub fn to_u64x8(self) -> u64x8 {
        u64x8(self.0)
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

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask(self, mask: Mask32) -> u16x32 {
        unsafe { _mm512_maskz_mov_epi16(mask.to_bitmask(), self.0).into() }
    }

    #[inline]
    pub unsafe fn compress_store<T>(self, mask: Mask32, dest: *mut T) {
        unsafe { _mm512_mask_compressstoreu_epi16(dest.cast(), mask.to_bitmask(), self.0) }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq(a: u16x32, b: u16x32) -> Mask32 {
        unsafe { _mm512_cmpeq_epu16_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn neq(a: u16x32, b: u16x32) -> Mask32 {
        unsafe { _mm512_cmpneq_epu16_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn test(a: u16x32, b: u16x32) -> Mask32 {
        unsafe { _mm512_test_epi16_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn testn(a: u16x32, b: u16x32) -> Mask32 {
        unsafe { _mm512_testn_epi16_mask(a.0, b.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero(self) -> Mask32 {
        u16x32::eq(self, u16x32::splat(0))
    }

    #[inline]
    pub fn nonzero(self) -> Mask32 {
        u16x32::neq(self, u16x32::splat(0))
    }

    #[inline]
    pub fn msb(self) -> Mask32 {
        unsafe { _mm512_movepi16_mask(self.0).into() }
    }
}

impl From<__m512i> for u16x32 {
    #[inline]
    fn from(raw: __m512i) -> Self {
        u16x32(raw)
    }
}

impl From<[u16; 32]> for u16x32 {
    #[inline]
    fn from(arr: [u16; 32]) -> Self {
        unsafe { u16x32::load(arr.as_ptr()) }
    }
}

impl BitAnd for u16x32 {
    type Output = u16x32;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_and_si512(self.0, rhs.0).into() }
    }
}

impl BitOr for u16x32 {
    type Output = u16x32;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_or_si512(self.0, rhs.0).into() }
    }
}

impl BitXor for u16x32 {
    type Output = u16x32;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_xor_si512(self.0, rhs.0).into() }
    }
}

impl BitAndAssign for u16x32 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u16x32) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u16x32 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u16x32) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u16x32 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u16x32) {
        *self = *self ^ rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u8x64(pub __m512i);

impl u8x64 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u8x64 {
        unsafe { _mm512_loadu_si512(src.cast()).into() }
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            _mm512_storeu_si512(dest.cast(), self.0);
        }
    }

    #[inline]
    pub fn splat(value: u8) -> u8x64 {
        unsafe { _mm512_set1_epi8(value as i8).into() }
    }

    /*----------------------------------------------------------------*/

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
        u16x64(
            self.extract32::<0>().zero_ext(),
            self.extract32::<1>().zero_ext(),
        )
    }

    #[inline]
    pub fn to_u16x32(self) -> u16x32 {
        u16x32(self.0)
    }

    #[inline]
    pub fn to_u32x16(self) -> u32x16 {
        u32x16(self.0)
    }

    #[inline]
    pub fn to_u64x8(self) -> u64x8 {
        u64x8(self.0)
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

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask(self, mask: Mask64) -> u8x64 {
        unsafe { _mm512_maskz_mov_epi8(mask.to_bitmask(), self.0).into() }
    }

    #[inline]
    pub fn blend(a: u8x64, b: u8x64, mask: Mask64) -> u8x64 {
        unsafe { _mm512_mask_blend_epi8(mask.to_bitmask(), a.0, b.0).into() }
    }

    #[inline]
    pub fn compress(self, mask: Mask64) -> u8x64 {
        unsafe { _mm512_maskz_compress_epi8(mask.to_bitmask(), self.0).into() }
    }

    #[inline]
    pub fn permute(self, index: u8x64) -> u8x64 {
        unsafe { _mm512_permutexvar_epi8(index.0, self.0).into() }
    }

    #[inline]
    pub fn shuffle(self, index: u8x64) -> u8x64 {
        unsafe { _mm512_shuffle_epi8(self.0, index.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq(a: u8x64, b: u8x64) -> Mask64 {
        unsafe { _mm512_cmpeq_epu8_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn neq(a: u8x64, b: u8x64) -> Mask64 {
        unsafe { _mm512_cmpneq_epu8_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn test(a: u8x64, b: u8x64) -> Mask64 {
        unsafe { _mm512_test_epi8_mask(a.0, b.0).into() }
    }

    #[inline]
    pub fn testn(a: u8x64, b: u8x64) -> Mask64 {
        unsafe { _mm512_testn_epi8_mask(a.0, b.0).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero(self) -> Mask64 {
        u8x64::eq(self, u8x64::splat(0))
    }

    #[inline]
    pub fn nonzero(self) -> Mask64 {
        u8x64::neq(self, u8x64::splat(0))
    }

    #[inline]
    pub fn msb(self) -> Mask64 {
        unsafe { _mm512_movepi8_mask(self.0).into() }
    }
}

impl From<__m512i> for u8x64 {
    #[inline]
    fn from(raw: __m512i) -> Self {
        u8x64(raw)
    }
}

impl From<[u8; 64]> for u8x64 {
    #[inline]
    fn from(arr: [u8; 64]) -> Self {
        unsafe { u8x64::load(arr.as_ptr()) }
    }
}

impl BitAnd for u8x64 {
    type Output = u8x64;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_and_si512(self.0, rhs.0).into() }
    }
}

impl BitOr for u8x64 {
    type Output = u8x64;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_or_si512(self.0, rhs.0).into() }
    }
}

impl BitXor for u8x64 {
    type Output = u8x64;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { _mm512_xor_si512(self.0, rhs.0).into() }
    }
}

impl BitAndAssign for u8x64 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u8x64) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u8x64 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u8x64) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u8x64 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u8x64) {
        *self = *self ^ rhs;
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct u16x64(pub u16x32, pub u16x32);

impl u16x64 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> u16x64 {
        let lo = unsafe { u16x32::load(src) };
        let hi = unsafe { u16x32::load(src.byte_add(64)) };

        u16x64(lo, hi)
    }

    #[inline]
    pub unsafe fn store<T>(self, dest: *mut T) {
        unsafe {
            self.0.store(dest);
            self.1.store(dest.byte_add(64));
        }
    }

    #[inline]
    pub fn splat(value: u16) -> u16x64 {
        u16x64(u16x32::splat(value), u16x32::splat(value))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl<const SHIFT: u32>(self) -> u16x64 {
        u16x64(self.0.shl::<SHIFT>(), self.1.shl::<SHIFT>())
    }

    #[inline]
    pub fn shlv(self, shift: u16x64) -> u16x64 {
        u16x64(self.0.shlv(shift.0), self.1.shlv(shift.1))
    }

    #[inline]
    pub fn shr<const SHIFT: u32>(self) -> u16x64 {
        u16x64(self.0.shr::<SHIFT>(), self.1.shr::<SHIFT>())
    }

    #[inline]
    pub fn shrv(self, shift: u16x64) -> u16x64 {
        u16x64(self.0.shrv(shift.0), self.1.shrv(shift.1))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask(self, mask: Mask64) -> u16x64 {
        let mask = mask.to_bitmask();
        let lo_mask = Mask32(mask as u32);
        let hi_mask = Mask32((mask >> 32) as u32);

        u16x64(self.0.mask(lo_mask), self.1.mask(hi_mask))
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eq(a: u16x64, b: u16x64) -> Mask64 {
        let lo = u16x32::eq(a.0, b.0);
        let hi = u16x32::eq(a.1, b.1);
        lo.concat(hi)
    }

    #[inline]
    pub fn neq(a: u16x64, b: u16x64) -> Mask64 {
        let lo = u16x32::neq(a.0, b.0);
        let hi = u16x32::neq(a.1, b.1);
        lo.concat(hi)
    }

    #[inline]
    pub fn test(a: u16x64, b: u16x64) -> Mask64 {
        let lo = u16x32::test(a.0, b.0);
        let hi = u16x32::test(a.1, b.1);
        lo.concat(hi)
    }

    #[inline]
    pub fn testn(a: u16x64, b: u16x64) -> Mask64 {
        let lo = u16x32::testn(a.0, b.0);
        let hi = u16x32::testn(a.1, b.1);
        lo.concat(hi)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero(self) -> Mask64 {
        u16x64::eq(self, u16x64::splat(0))
    }

    #[inline]
    pub fn nonzero(self) -> Mask64 {
        u16x64::neq(self, u16x64::splat(0))
    }

    #[inline]
    pub fn msb(self) -> Mask64 {
        let lo = self.0.msb();
        let hi = self.1.msb();
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
    type Output = u16x64;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        u16x64(self.0 & rhs.0, self.1 & rhs.1)
    }
}

impl BitOr for u16x64 {
    type Output = u16x64;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        u16x64(self.0 | rhs.0, self.1 | rhs.1)
    }
}

impl BitXor for u16x64 {
    type Output = u16x64;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        u16x64(self.0 ^ rhs.0, self.1 ^ rhs.1)
    }
}

impl BitAndAssign for u16x64 {
    #[inline]
    fn bitand_assign(&mut self, rhs: u16x64) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for u16x64 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u16x64) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for u16x64 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u16x64) {
        *self = *self ^ rhs;
    }
}
