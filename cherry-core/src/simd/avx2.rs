use core::{arch::x86_64::*, ops::*};

/*----------------------------------------------------------------*/

macro_rules! def_mask {
    ($mask:ident, $vec:ty, $bitmask:ty) => {
        #[derive(Debug, Copy, Clone)]
        pub enum $mask {
            Vec($vec),
            Bitmask($bitmask),
        }

        impl $mask {
            #[inline]
            pub fn expand_inner(&mut self) {
                *self = $mask::from(self.expand())
            }
        }

        impl From<$vec> for $mask {
            #[inline]
            fn from(vec: $vec) -> Self {
                $mask::Vec(vec)
            }
        }

        impl From<$bitmask> for $mask {
            #[inline]
            fn from(bitmask: $bitmask) -> Self {
                $mask::Bitmask(bitmask)
            }
        }

        impl Not for $mask {
            type Output = Self;

            #[inline]
            fn not(self) -> Self::Output {
                match self {
                    $mask::Vec(vec) => $mask::Vec(!vec),
                    $mask::Bitmask(bitmask) => $mask::Bitmask(!bitmask),
                }
            }
        }

        impl BitAnd for $mask {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    ($mask::Vec(a), $mask::Vec(b)) => $mask::Vec(a & b),
                    ($mask::Bitmask(a), $mask::Bitmask(b)) => $mask::Bitmask(a & b),
                    ($mask::Vec(_), $mask::Bitmask(bitmask)) =>
                        $mask::Bitmask(self.to_bitmask() & bitmask),
                    ($mask::Bitmask(bitmask), $mask::Vec(_)) =>
                        $mask::Bitmask(rhs.to_bitmask() & bitmask),
                }
            }
        }

        impl BitOr for $mask {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    ($mask::Vec(a), $mask::Vec(b)) => $mask::Vec(a | b),
                    ($mask::Bitmask(a), $mask::Bitmask(b)) => $mask::Bitmask(a | b),
                    ($mask::Vec(_), $mask::Bitmask(bitmask)) =>
                        $mask::Bitmask(self.to_bitmask() | bitmask),
                    ($mask::Bitmask(bitmask), $mask::Vec(_)) =>
                        $mask::Bitmask(rhs.to_bitmask() | bitmask),
                }
            }
        }

        impl BitXor for $mask {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    ($mask::Vec(a), $mask::Vec(b)) => $mask::Vec(a ^ b),
                    ($mask::Bitmask(a), $mask::Bitmask(b)) => $mask::Bitmask(a ^ b),
                    ($mask::Vec(_), $mask::Bitmask(bitmask)) =>
                        $mask::Bitmask(self.to_bitmask() ^ bitmask),
                    ($mask::Bitmask(bitmask), $mask::Vec(_)) =>
                        $mask::Bitmask(rhs.to_bitmask() ^ bitmask),
                }
            }
        }

        impl BitAnd<$bitmask> for $mask {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: $bitmask) -> Self::Output {
                $mask::Bitmask(self.to_bitmask() & rhs)
            }
        }

        impl BitOr<$bitmask> for $mask {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: $bitmask) -> Self::Output {
                $mask::Bitmask(self.to_bitmask() | rhs)
            }
        }

        impl BitXor<$bitmask> for $mask {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: $bitmask) -> Self::Output {
                $mask::Bitmask(self.to_bitmask() ^ rhs)
            }
        }

        impl BitAndAssign for $mask {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs;
            }
        }

        impl BitOrAssign for $mask {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                *self = *self | rhs;
            }
        }

        impl BitXorAssign for $mask {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = *self ^ rhs;
            }
        }

        impl BitAndAssign<$bitmask> for $mask {
            #[inline]
            fn bitand_assign(&mut self, rhs: $bitmask) {
                *self = *self & rhs;
            }
        }

        impl BitOrAssign<$bitmask> for $mask {
            #[inline]
            fn bitor_assign(&mut self, rhs: $bitmask) {
                *self = *self | rhs;
            }
        }

        impl BitXorAssign<$bitmask> for $mask {
            #[inline]
            fn bitxor_assign(&mut self, rhs: $bitmask) {
                *self = *self ^ rhs;
            }
        }
    };
}

/*----------------------------------------------------------------*/

def_mask!(Mask8x16, u8x16, u16);
impl Mask8x16 {
    #[inline]
    pub fn expand(self) -> u8x16 {
        match self {
            Mask8x16::Vec(vec) => vec,
            Mask8x16::Bitmask(bitmask) => unsafe {
                let shuffled = _mm_shuffle_epi8(
                    _mm_cvtsi32_si128(bitmask as i32),
                    _mm_set_epi64x(0x0101010101010101, 0),
                );
                let and_mask = _mm_set1_epi64x(0x8040201008040201u64 as i64);

                _mm_cmpeq_epi8(and_mask, _mm_and_si128(and_mask, shuffled)).into()
            },
        }
    }

    #[inline]
    pub fn widen(self) -> Mask16x16 {
        match self {
            Mask8x16::Vec(vec) => {
                let vec = vec.zero_ext();
                (vec | (vec.shl::<8>())).into()
            }
            Mask8x16::Bitmask(bitmask) => Mask16x16::from(bitmask),
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u16 {
        match self {
            Mask8x16::Vec(vec) => unsafe { _mm_movemask_epi8(vec.0) as u16 },
            Mask8x16::Bitmask(bitmask) => bitmask,
        }
    }
}

def_mask!(Mask16x8, u16x8, u8);
impl Mask16x8 {
    #[inline]
    pub fn expand(self) -> u16x8 {
        match self {
            Mask16x8::Vec(vec) => vec,
            Mask16x8::Bitmask(bitmask) => unsafe {
                let mask_vec = _mm_set1_epi8(bitmask as i8);
                let and_mask = _mm_setr_epi16(0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80);

                _mm_cmpeq_epi16(and_mask, _mm_and_si128(mask_vec, and_mask)).into()
            },
        }
    }

    #[inline]
    pub fn widen(self) -> Mask32x8 {
        match self {
            Mask16x8::Vec(vec) => unsafe {
                let vec = vec.zero_ext();
                (vec | (_mm256_slli_epi32::<16>(vec.0).into())).into()
            },
            Mask16x8::Bitmask(bitmask) => Mask32x8::from(bitmask),
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u8 {
        match self {
            Mask16x8::Vec(vec) => unsafe {
                _pext_u32(_mm_movemask_epi8(vec.0) as u32, 0xAAAAAAAAu32) as u8
            },
            Mask16x8::Bitmask(bitmask) => bitmask,
        }
    }
}

def_mask!(Mask32x4, u32x4, u8);
impl Mask32x4 {
    #[inline]
    pub fn expand(self) -> u32x4 {
        match self {
            Mask32x4::Vec(vec) => vec,
            Mask32x4::Bitmask(bitmask) => unsafe {
                let mask_vec = _mm_set1_epi8(bitmask as i8);
                let and_mask = _mm_setr_epi32(0x01, 0x02, 0x04, 0x08);

                _mm_cmpeq_epi32(and_mask, _mm_and_si128(and_mask, mask_vec)).into()
            },
        }
    }

    #[inline]
    pub fn widen(self) -> Mask64x4 {
        match self {
            Mask32x4::Vec(vec) => unsafe {
                let vec = vec.zero_ext();
                (vec | (_mm256_slli_epi64::<32>(vec.0).into())).into()
            },
            Mask32x4::Bitmask(bitmask) => Mask64x4::from(bitmask),
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u8 {
        match self {
            Mask32x4::Vec(vec) => unsafe {
                _pext_u32(_mm_movemask_epi8(vec.0) as u32, 0x88888888u32) as u8
            },
            Mask32x4::Bitmask(bitmask) => bitmask,
        }
    }
}

def_mask!(Mask64x2, u64x2, u8);
impl Mask64x2 {
    #[inline]
    pub fn expand(self) -> u64x2 {
        match self {
            Mask64x2::Vec(vec) => vec,
            Mask64x2::Bitmask(bitmask) => unsafe {
                let mask_vec = _mm_set1_epi8(bitmask as i8);
                let and_mask = _mm_set_epi64x(0x02, 0x01);

                _mm_cmpeq_epi64(and_mask, _mm_and_si128(mask_vec, and_mask)).into()
            },
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u8 {
        match self {
            Mask64x2::Vec(vec) => unsafe {
                _pext_u32(_mm_movemask_epi8(vec.0) as u32, 0x80808080u32) as u8
            },
            Mask64x2::Bitmask(bitmask) => bitmask,
        }
    }
}

/*----------------------------------------------------------------*/

def_mask!(Mask8x32, u8x32, u32);
impl Mask8x32 {
    #[inline]
    pub fn expand(self) -> u8x32 {
        match self {
            Mask8x32::Vec(vec) => vec,
            Mask8x32::Bitmask(bitmask) => unsafe {
                let shuffled = _mm256_shuffle_epi8(
                    _mm256_set1_epi32(bitmask as i32),
                    _mm256_setr_epi64x(
                        0x0000000000000000,
                        0x0101010101010101,
                        0x0202020202020202,
                        0x0303030303030303,
                    ),
                );
                let and_mask = _mm256_set1_epi64x(0x8040201008040201u64 as i64);

                _mm256_cmpeq_epi8(and_mask, _mm256_and_si256(and_mask, shuffled)).into()
            },
        }
    }

    #[inline]
    pub fn widen(self) -> Mask16x32 {
        match self {
            Mask8x32::Vec(vec) => {
                let vec = vec.zero_ext();
                (vec | vec.shl::<8>()).into()
            }
            Mask8x32::Bitmask(bitmask) => Mask16x32::from(bitmask),
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u32 {
        match self {
            Mask8x32::Vec(vec) => unsafe { _mm256_movemask_epi8(vec.0) as u32 },
            Mask8x32::Bitmask(bitmask) => bitmask,
        }
    }
}

def_mask!(Mask16x16, u16x16, u16);
impl Mask16x16 {
    #[inline]
    pub fn expand(self) -> u16x16 {
        match self {
            Mask16x16::Vec(vec) => vec,
            Mask16x16::Bitmask(bitmask) => unsafe {
                let vec = _mm256_set1_epi16(bitmask as i16);
                let and_mask = _mm256_setr_epi16(
                    0x0001,
                    0x0002,
                    0x0004,
                    0x0008,
                    0x0010,
                    0x0020,
                    0x0040,
                    0x0080,
                    0x0100,
                    0x0200,
                    0x0400,
                    0x0800,
                    0x1000,
                    0x2000,
                    0x4000,
                    0x8000u16 as i16,
                );

                _mm256_cmpeq_epi16(and_mask, _mm256_and_si256(vec, and_mask)).into()
            },
        }
    }

    #[inline]
    pub fn widen(self) -> Mask32x16 {
        match self {
            Mask16x16::Vec(vec) => unsafe {
                let vec = vec.zero_ext();
                let shifted_lo = _mm256_slli_epi32::<16>(vec.0[0].0).into();
                let shifted_hi = _mm256_slli_epi32::<16>(vec.0[1].0).into();

                (vec | u32x16([shifted_lo, shifted_hi])).into()
            },
            Mask16x16::Bitmask(bitmask) => Mask32x16::from(bitmask),
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u16 {
        match self {
            Mask16x16::Vec(vec) => unsafe {
                _pext_u32(_mm256_movemask_epi8(vec.0) as u32, 0xAAAAAAAAu32) as u16
            },
            Mask16x16::Bitmask(bitmask) => bitmask,
        }
    }
}

def_mask!(Mask32x8, u32x8, u8);
impl Mask32x8 {
    #[inline]
    pub fn expand(self) -> u32x8 {
        match self {
            Mask32x8::Vec(vec) => vec,
            Mask32x8::Bitmask(bitmask) => unsafe {
                let mask_vec = _mm256_set1_epi8(bitmask as i8);
                let and_mask = _mm256_setr_epi32(0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80);

                _mm256_cmpeq_epi32(and_mask, _mm256_and_si256(and_mask, mask_vec)).into()
            },
        }
    }

    #[inline]
    pub fn widen(self) -> Mask64x8 {
        match self {
            Mask32x8::Vec(vec) => unsafe {
                let vec = vec.zero_ext();
                let shifted_lo = _mm256_slli_epi64::<32>(vec.0[0].0).into();
                let shifted_hi = _mm256_slli_epi64::<32>(vec.0[1].0).into();

                (vec | u64x8([shifted_lo, shifted_hi])).into()
            },
            Mask32x8::Bitmask(bitmask) => Mask64x8::from(bitmask),
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u8 {
        match self {
            Mask32x8::Vec(vec) => unsafe {
                _pext_u32(_mm256_movemask_epi8(vec.0) as u32, 0x88888888u32) as u8
            },
            Mask32x8::Bitmask(bitmask) => bitmask,
        }
    }
}

def_mask!(Mask64x4, u64x4, u8);
impl Mask64x4 {
    #[inline]
    pub fn expand(self) -> u64x4 {
        match self {
            Mask64x4::Vec(vec) => vec,
            Mask64x4::Bitmask(bitmask) => unsafe {
                let mask_vec = _mm256_set1_epi8(bitmask as i8);
                let and_mask = _mm256_setr_epi64x(0x01, 0x02, 0x04, 0x08);

                _mm256_cmpeq_epi64(and_mask, _mm256_and_si256(mask_vec, and_mask)).into()
            },
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u8 {
        match self {
            Mask64x4::Vec(vec) => unsafe {
                _pext_u32(_mm256_movemask_epi8(vec.0) as u32, 0x80808080u32) as u8
            },
            Mask64x4::Bitmask(bitmask) => bitmask,
        }
    }
}

/*----------------------------------------------------------------*/

def_mask!(Mask8x64, u8x64, u64);
impl Mask8x64 {
    #[inline]
    pub fn expand(self) -> u8x64 {
        match self {
            Mask8x64::Vec(vec) => vec,
            Mask8x64::Bitmask(bitmask) => unsafe {
                let shuffled0 = _mm256_shuffle_epi8(
                    _mm256_set1_epi32(bitmask as i32),
                    _mm256_setr_epi64x(
                        0x0000000000000000,
                        0x0101010101010101,
                        0x0202020202020202,
                        0x0303030303030303,
                    ),
                );
                let shuffled1 = _mm256_shuffle_epi8(
                    _mm256_set1_epi32((bitmask >> 32) as i32),
                    _mm256_setr_epi64x(
                        0x0000000000000000,
                        0x0101010101010101,
                        0x0202020202020202,
                        0x0303030303030303,
                    ),
                );

                let and_mask = _mm256_set1_epi64x(0x8040201008040201u64 as i64);
                let lo = _mm256_cmpeq_epi8(and_mask, _mm256_and_si256(and_mask, shuffled0));
                let hi = _mm256_cmpeq_epi8(and_mask, _mm256_and_si256(and_mask, shuffled1));

                u8x64([u8x32(lo), u8x32(hi)])
            },
        }
    }

    #[inline]
    pub fn widen(self) -> Mask16x64 {
        match self {
            Mask8x64::Vec(vec) => {
                let vec = vec.zero_ext();
                (vec | (vec.shl::<8>())).into()
            }
            Mask8x64::Bitmask(bitmask) => Mask16x64::from(bitmask),
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u64 {
        match self {
            Mask8x64::Vec(vec) => unsafe {
                let lo = _mm256_movemask_epi8(vec.0[0].0) as u32 as u64;
                let hi = _mm256_movemask_epi8(vec.0[1].0) as u32 as u64;
                lo | (hi << 32)
            },
            Mask8x64::Bitmask(bitmask) => bitmask,
        }
    }
}

def_mask!(Mask16x32, u16x32, u32);
impl Mask16x32 {
    #[inline]
    pub fn expand(self) -> u16x32 {
        match self {
            Mask16x32::Vec(vec) => vec,
            Mask16x32::Bitmask(bitmask) => {
                let lo = Mask16x16::from(bitmask as u16).expand();
                let hi = Mask16x16::from((bitmask >> 16) as u16).expand();
                u16x32([lo, hi])
            }
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u32 {
        match self {
            Mask16x32::Vec(vec) => unsafe {
                let lo = _mm256_movemask_epi8(vec.0[0].0) as u32 as u64;
                let hi = _mm256_movemask_epi8(vec.0[1].0) as u32 as u64;

                _pext_u64(lo | (hi << 32), 0xAAAAAAAAAAAAAAAAu64) as u32
            },
            Mask16x32::Bitmask(bitmask) => bitmask,
        }
    }
}

def_mask!(Mask32x16, u32x16, u16);
impl Mask32x16 {
    #[inline]
    pub fn expand(self) -> u32x16 {
        match self {
            Mask32x16::Vec(vec) => vec,
            Mask32x16::Bitmask(bitmask) => {
                let lo = Mask32x8::from(bitmask as u8).expand();
                let hi = Mask32x8::from((bitmask >> 8) as u8).expand();
                u32x16([lo, hi])
            }
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u16 {
        match self {
            Mask32x16::Vec(vec) => unsafe {
                let lo = _mm256_movemask_epi8(vec.0[0].0) as u32 as u64;
                let hi = _mm256_movemask_epi8(vec.0[1].0) as u32 as u64;

                _pext_u64(lo | (hi << 32), 0x8888888888888888u64) as u16
            },
            Mask32x16::Bitmask(bitmask) => bitmask,
        }
    }
}

def_mask!(Mask64x8, u64x8, u8);
impl Mask64x8 {
    #[inline]
    pub fn expand(self) -> u64x8 {
        match self {
            Mask64x8::Vec(vec) => vec,
            Mask64x8::Bitmask(bitmask) => unsafe {
                let mask_vec = _mm256_set1_epi8(bitmask as i8);
                let and_mask0 = _mm256_setr_epi64x(0x01, 0x02, 0x04, 0x08);
                let and_mask1 = _mm256_setr_epi64x(0x10, 0x20, 0x40, 0x80);

                let lo =
                    _mm256_cmpeq_epi64(and_mask0, _mm256_and_si256(mask_vec, and_mask0)).into();
                let hi =
                    _mm256_cmpeq_epi64(and_mask1, _mm256_and_si256(mask_vec, and_mask1)).into();
                u64x8([lo, hi])
            },
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u8 {
        match self {
            Mask64x8::Vec(vec) => unsafe {
                let lo = _mm256_movemask_epi8(vec.0[0].0) as u32 as u64;
                let hi = _mm256_movemask_epi8(vec.0[1].0) as u32 as u64;
                _pext_u64(lo | (hi << 32), 0x8080808080808080u64) as u8
            },
            Mask64x8::Bitmask(bitmask) => bitmask,
        }
    }
}

/*----------------------------------------------------------------*/

def_mask!(Mask16x64, u16x64, u64);
impl Mask16x64 {
    #[inline]
    pub fn expand(self) -> u16x64 {
        match self {
            Mask16x64::Vec(vec) => vec,
            Mask16x64::Bitmask(bitmask) => {
                let lo = Mask16x32::from(bitmask as u32).expand();
                let hi = Mask16x32::from((bitmask >> 32) as u32).expand();
                u16x64([lo, hi])
            }
        }
    }

    #[inline]
    pub fn to_bitmask(self) -> u64 {
        match self {
            Mask16x64::Vec(vec) => unsafe {
                let lo0 = _mm256_movemask_epi8(vec.0[0].0[0].0) as u32 as u64;
                let hi0 = _mm256_movemask_epi8(vec.0[0].0[1].0) as u32 as u64;
                let mask_lo = _pext_u64(lo0 | (hi0 << 32), 0xAAAAAAAAAAAAAAAAu64);

                let lo1 = _mm256_movemask_epi8(vec.0[1].0[0].0) as u32 as u64;
                let hi1 = _mm256_movemask_epi8(vec.0[1].0[1].0) as u32 as u64;
                let mask_hi = _pext_u64(lo1 | (hi1 << 32), 0xAAAAAAAAAAAAAAAAu64);

                mask_lo | (mask_hi << 32)
            },
            Mask16x64::Bitmask(bitmask) => bitmask,
        }
    }
}

/*----------------------------------------------------------------*/

macro_rules! def_vec {
    (
        $vec:ident, $raw_vec:ty, $elem:ident, $arr:ty;
        $load:ident,
        $store:ident,
        $splat:ident,
        $andnot:ident,
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
            pub fn splat(value: $elem) -> $vec {
                unsafe { $splat(value as _).into() }
            }

            #[inline]
            pub fn andnot(self, other: $vec) -> $vec {
                unsafe { $andnot(self.0, other.0).into() }
            }
        }

        impl From<$raw_vec> for $vec {
            #[inline]
            fn from(raw: $raw_vec) -> Self {
                Self(raw)
            }
        }

        impl From<$arr> for $vec {
            #[inline]
            fn from(arr: $arr) -> Self {
                unsafe { $vec::load(arr.as_ptr()) }
            }
        }

        impl Not for $vec {
            type Output = Self;

            #[inline]
            fn not(self) -> Self::Output {
                self ^ $vec::splat($elem::MAX)
            }
        }

        impl BitAnd for $vec {
            type Output = Self;

            #[inline]
            fn bitand(self, other: Self) -> Self::Output {
                unsafe { $bitand(self.0, other.0).into() }
            }
        }

        impl BitOr for $vec {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self::Output {
                unsafe { $bitor(self.0, other.0).into() }
            }
        }

        impl BitXor for $vec {
            type Output = Self;

            #[inline]
            fn bitxor(self, other: Self) -> Self::Output {
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
    };
}

macro_rules! impl_conv {
    ($vec:ty, $($fn:ident => $output:ident;)*) => {
        impl $vec {$(
            #[inline]
            pub fn $fn(self) -> $output {
                $output(self.0)
            }
        )*}
    }
}

macro_rules! impl_cmp {
    ($vec:ident, $mask:ty, $elem:ident, $eq:ident, $gt:ident) => {
        impl $vec {
            #[inline]
            pub fn eq(a: $vec, b: $vec) -> $mask {
                let mask_vec: $vec = unsafe { $eq(a.0, b.0).into() };
                mask_vec.into()
            }

            #[inline]
            pub fn neq(a: $vec, b: $vec) -> $mask {
                let mask_vec: $vec = unsafe { $eq(a.0, b.0).into() };
                (!mask_vec).into()
            }

            #[inline]
            pub fn test(a: $vec, b: $vec) -> $mask {
                (a & b).nonzero()
            }

            #[inline]
            pub fn testn(a: $vec, b: $vec) -> $mask {
                (a & b).zero()
            }

            /*----------------------------------------------------------------*/

            #[inline]
            pub fn zero(self) -> $mask {
                $vec::eq(self, $vec::splat(0))
            }

            #[inline]
            pub fn nonzero(self) -> $mask {
                $vec::neq(self, $vec::splat(0))
            }

            #[inline]
            pub fn msb(self) -> $mask {
                let mask_vec: $vec = unsafe { $gt($vec::splat(0).0, self.0).into() };
                mask_vec.into()
            }
        }
    };
}

macro_rules! impl_select {
    ($vec:ident, $mask:ty, $arr:ty) => {
        impl $vec {
            #[inline]
            pub fn mask(self, mask: $mask) -> $vec {
                self & mask.expand()
            }

            #[inline]
            pub fn blend(a: $vec, b: $vec, mask: $mask) -> $vec {
                let mask_vec = mask.expand();

                (mask_vec & b) | mask_vec.andnot(a)
            }

            #[inline]
            pub fn compress(self, mask: $mask) -> $vec {
                let mut values = <$arr>::default();
                unsafe {
                    self.store(values.as_mut_ptr());
                }

                let mut mask = mask.to_bitmask();
                let mut temp = <$arr>::default();
                let mut cursor = 0;

                while mask != 0 {
                    temp[cursor] = values[mask.trailing_zeros() as usize];
                    mask &= mask.wrapping_sub(1);
                    cursor += 1;
                }

                $vec::from(temp)
            }

            #[inline]
            pub unsafe fn compress_store<T>(self, mask: $mask, dest: *mut T) {
                unsafe { self.compress(mask).store(dest) }
            }
        }
    };
}

/*----------------------------------------------------------------*/

def_vec! {
    u8x16, __m128i, u8, [u8; 16];
    _mm_loadu_si128,
    _mm_storeu_si128,
    _mm_set1_epi8,
    _mm_andnot_si128,
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
    u8x16,
    Mask8x16,
    u8,
    _mm_cmpeq_epi8,
    _mm_cmpgt_epi8
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
        self.broadcast32().broadcast64()
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
    pub fn mask(self, mask: Mask8x16) -> u8x16 {
        self & mask.expand()
    }

    #[inline]
    pub fn blend(a: u8x16, b: u8x16, mask: Mask8x16) -> u8x16 {
        unsafe { _mm_blendv_epi8(a.0, b.0, mask.expand().0).into() }
    }

    #[inline]
    pub fn compress(self, mask: Mask8x16) -> u8x16 {
        let mut values = [0u8; 16];
        unsafe {
            self.store(values.as_mut_ptr());
        }

        let mut mask = mask.to_bitmask();
        let mut temp = [0u8; 16];
        let mut cursor = 0;

        while mask != 0 {
            temp[cursor] = values[mask.trailing_zeros() as usize];
            mask &= mask.wrapping_sub(1);
            cursor += 1;
        }

        u8x16::from(temp)
    }

    #[inline]
    pub unsafe fn compress_store<T>(self, mask: Mask8x16, dest: *mut T) {
        unsafe { self.compress(mask).store(dest) }
    }

    #[inline]
    pub fn shuffle(self, index: u8x16) -> u8x16 {
        unsafe { _mm_shuffle_epi8(self.0, index.0).into() }
    }
}

def_vec! {
    u16x8, __m128i, u16, [u16; 8];
    _mm_loadu_si128,
    _mm_storeu_si128,
    _mm_set1_epi16,
    _mm_andnot_si128,
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
    u16x8,
    Mask16x8,
    u16,
    _mm_cmpeq_epi16,
    _mm_cmpgt_epi16
}
impl_select! {
    u16x8,
    Mask16x8,
    [u16; 8]
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
        self.broadcast16().broadcast32()
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
    _mm_andnot_si128,
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
impl_cmp! {
    u32x4,
    Mask32x4,
    u32,
    _mm_cmpeq_epi32,
    _mm_cmpgt_epi32
}
impl_select! {
    u32x4,
    Mask32x4,
    [u32; 4]
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
        self.broadcast8().broadcast16()
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
    _mm_andnot_si128,
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
impl_cmp! {
    u64x2,
    Mask64x2,
    u64,
    _mm_cmpeq_epi64,
    _mm_cmpgt_epi64
}
impl_select! {
    u64x2,
    Mask64x2,
    [u64; 2]
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
        self.broadcast4().broadcast8()
    }
}

/*----------------------------------------------------------------*/

def_vec! {
    u8x32, __m256i, u8, [u8; 32];
    _mm256_loadu_si256,
    _mm256_storeu_si256,
    _mm256_set1_epi8,
    _mm256_andnot_si256,
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
    u8x32,
    Mask8x32,
    u8,
    _mm256_cmpeq_epi8,
    _mm256_cmpgt_epi8
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
        u8x64([self; 2])
    }

    #[inline]
    pub fn zero_ext(self) -> u16x32 {
        let lo = self.extract16::<0>().zero_ext();
        let hi = self.extract16::<1>().zero_ext();
        u16x32([lo, hi])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask(self, mask: Mask8x32) -> u8x32 {
        self & mask.expand()
    }

    #[inline]
    pub fn blend(a: u8x32, b: u8x32, mask: Mask8x32) -> u8x32 {
        unsafe { _mm256_blendv_epi8(a.0, b.0, mask.expand().0).into() }
    }

    #[inline]
    pub fn compress(self, mask: Mask8x32) -> u8x32 {
        let mut values = [0u8; 32];
        unsafe {
            self.store(values.as_mut_ptr());
        }

        let mut mask = mask.to_bitmask();
        let mut temp = [0u8; 32];
        let mut cursor = 0;

        while mask != 0 {
            temp[cursor] = values[mask.trailing_zeros() as usize];
            mask &= mask.wrapping_sub(1);
            cursor += 1;
        }

        u8x32::from(temp)
    }

    #[inline]
    pub unsafe fn compress_store<T>(self, mask: Mask8x32, dest: *mut T) {
        unsafe { self.compress(mask).store(dest) }
    }

    #[inline]
    pub fn permute(self, index: u8x32) -> u8x32 {
        let mask = Mask8x32::from(index.to_u16x16().shl::<3>().to_u8x32());
        let index = index & u8x32::splat(15);
        let lo: u8x32 = unsafe { _mm256_permute2x128_si256::<0x00>(self.0, self.0).into() };
        let hi: u8x32 = unsafe { _mm256_permute2x128_si256::<0x11>(self.0, self.0).into() };

        u8x32::blend(lo.shuffle(index), hi.shuffle(index), mask)
    }

    #[inline]
    pub fn shuffle(self, index: u8x32) -> u8x32 {
        unsafe { _mm256_shuffle_epi8(self.0, index.0).into() }
    }
}

def_vec! {
    u16x16, __m256i, u16, [u16; 16];
    _mm256_loadu_si256,
    _mm256_storeu_si256,
    _mm256_set1_epi16,
    _mm256_andnot_si256,
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
    u16x16,
    Mask16x16,
    u16,
    _mm256_cmpeq_epi16,
    _mm256_cmpgt_epi16
}
impl_select! {
    u16x16,
    Mask16x16,
    [u16; 16]
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
        u16x32([self; 2])
    }

    #[inline]
    pub fn zero_ext(self) -> u32x16 {
        let lo = self.extract8::<0>().zero_ext();
        let hi = self.extract8::<1>().zero_ext();
        u32x16([lo, hi])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl<const SHIFT: i32>(self) -> u16x16 {
        unsafe { _mm256_slli_epi16::<SHIFT>(self.0).into() }
    }

    #[inline]
    pub fn shlv(self, shift: u16x16) -> u16x16 {
        unsafe {
            let zero = _mm256_setzero_si256();
            let vec_lo = _mm256_unpacklo_epi16(zero, self.0);
            let vec_hi = _mm256_unpackhi_epi16(zero, self.0);
            let shift_lo = _mm256_unpacklo_epi16(shift.0, zero);
            let shift_hi = _mm256_unpackhi_epi16(shift.0, zero);
            let result_lo = _mm256_srli_epi32(_mm256_sllv_epi32(vec_lo, shift_lo), 16);
            let result_hi = _mm256_srli_epi32(_mm256_sllv_epi32(vec_hi, shift_hi), 16);

            _mm256_packus_epi32(result_lo, result_hi).into()
        }
    }

    #[inline]
    pub fn shr<const SHIFT: i32>(self) -> u16x16 {
        unsafe { _mm256_srli_epi16::<SHIFT>(self.0).into() }
    }

    #[inline]
    pub fn shrv(self, shift: u16x16) -> u16x16 {
        unsafe {
            let zero = _mm256_setzero_si256();
            let vec_lo = _mm256_unpacklo_epi16(self.0, zero);
            let vec_hi = _mm256_unpackhi_epi16(self.0, zero);
            let shift_lo = _mm256_unpacklo_epi16(shift.0, zero);
            let shift_hi = _mm256_unpackhi_epi16(shift.0, zero);
            let result_lo = _mm256_srlv_epi32(vec_lo, shift_lo);
            let result_hi = _mm256_srlv_epi32(vec_hi, shift_hi);

            _mm256_packus_epi32(result_lo, result_hi).into()
        }
    }
}

def_vec! {
    u32x8, __m256i, u32, [u32; 8];
    _mm256_loadu_si256,
    _mm256_storeu_si256,
    _mm256_set1_epi32,
    _mm256_andnot_si256,
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
    u32x8,
    Mask32x8,
    u32,
    _mm256_cmpeq_epi32,
    _mm256_cmpgt_epi32
}
impl_select! {
    u32x8,
    Mask32x8,
    [u32; 8]
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
        u32x16([self; 2])
    }

    #[inline]
    pub fn zero_ext(self) -> u64x8 {
        let lo = self.extract4::<0>().zero_ext();
        let hi = self.extract4::<1>().zero_ext();
        u64x8([lo, hi])
    }
}

def_vec! {
    u64x4, __m256i, u64, [u64; 4];
    _mm256_loadu_si256,
    _mm256_storeu_si256,
    _mm256_set1_epi64x,
    _mm256_andnot_si256,
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
impl_cmp! {
    u64x4,
    Mask64x4,
    u64,
    _mm256_cmpeq_epi64,
    _mm256_cmpgt_epi64
}
impl_select! {
    u64x4,
    Mask64x4,
    [u64; 4]
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
        u64x8([self; 2])
    }
}

/*----------------------------------------------------------------*/

macro_rules! def_big_vec {
    ($vec:ident, $half_vec:ident, $half_vec_width:expr, $elem:ident, $arr:ty) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $vec([$half_vec; 2]);

        impl $vec {
            #[inline]
            pub unsafe fn load<T>(src: *const T) -> $vec {
                let lo = unsafe { $half_vec::load(src) };
                let hi = unsafe { $half_vec::load(src.byte_add($half_vec_width)) };

                $vec([lo, hi])
            }

            #[inline]
            pub unsafe fn store<T>(self, dest: *mut T) {
                unsafe {
                    self.0[0].store(dest);
                    self.0[1].store(dest.byte_add($half_vec_width));
                }
            }

            #[inline]
            pub fn splat(value: $elem) -> $vec {
                let half = $half_vec::splat(value as _);
                $vec([half; 2])
            }

            #[inline]
            pub fn andnot(self, other: $vec) -> $vec {
                let lo = self.0[0].andnot(other.0[0]);
                let hi = self.0[1].andnot(other.0[1]);
                $vec([lo, hi])
            }
        }

        impl From<$arr> for $vec {
            #[inline]
            fn from(arr: $arr) -> Self {
                unsafe { $vec::load(arr.as_ptr()) }
            }
        }

        impl Not for $vec {
            type Output = Self;

            #[inline]
            fn not(self) -> Self::Output {
                self ^ $vec::splat($elem::MAX)
            }
        }

        impl BitAnd for $vec {
            type Output = Self;

            #[inline]
            fn bitand(self, other: Self) -> Self::Output {
                let lo = self.0[0] & other.0[0];
                let hi = self.0[1] & other.0[1];
                $vec([lo, hi])
            }
        }

        impl BitOr for $vec {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self::Output {
                let lo = self.0[0] | other.0[0];
                let hi = self.0[1] | other.0[1];
                $vec([lo, hi])
            }
        }

        impl BitXor for $vec {
            type Output = Self;

            #[inline]
            fn bitxor(self, other: Self) -> Self::Output {
                let lo = self.0[0] ^ other.0[0];
                let hi = self.0[1] ^ other.0[1];
                $vec([lo, hi])
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
    };
}
macro_rules! impl_big_conv {
    ($vec:ty, $($conv_fn:ident | $half_conv_fn:ident => $other_ty:ident;)*) => {
        impl $vec {$(
            #[inline]
            pub fn $conv_fn(self) -> $other_ty {
                let lo = self.0[0].$half_conv_fn();
                let hi = self.0[1].$half_conv_fn();

                $other_ty([lo, hi])
            }
        )*}
    }
}
macro_rules! impl_big_cmp {
    ($vec:ident, $half_vec:ident, $mask:ident) => {
        impl $vec {
            #[inline]
            pub fn eq(a: $vec, b: $vec) -> $mask {
                let lo = $half_vec::eq(a.0[0], b.0[0]).expand();
                let hi = $half_vec::eq(a.0[1], b.0[1]).expand();
                $mask::from($vec([lo, hi]))
            }

            #[inline]
            pub fn neq(a: $vec, b: $vec) -> $mask {
                let lo = $half_vec::neq(a.0[0], b.0[0]).expand();
                let hi = $half_vec::neq(a.0[1], b.0[1]).expand();
                $mask::from($vec([lo, hi]))
            }

            #[inline]
            pub fn test(a: $vec, b: $vec) -> $mask {
                let lo = $half_vec::test(a.0[0], b.0[0]).expand();
                let hi = $half_vec::test(a.0[1], b.0[1]).expand();
                $mask::from($vec([lo, hi]))
            }

            #[inline]
            pub fn testn(a: $vec, b: $vec) -> $mask {
                let lo = $half_vec::testn(a.0[0], b.0[0]).expand();
                let hi = $half_vec::testn(a.0[1], b.0[1]).expand();
                $mask::from($vec([lo, hi]))
            }

            /*----------------------------------------------------------------*/

            #[inline]
            pub fn zero(self) -> $mask {
                $vec::eq(self, $vec::splat(0))
            }

            #[inline]
            pub fn nonzero(self) -> $mask {
                $vec::neq(self, $vec::splat(0))
            }

            /*----------------------------------------------------------------*/

            #[inline]
            pub fn msb(self) -> $mask {
                let lo = self.0[0].msb().expand();
                let hi = self.0[1].msb().expand();
                $mask::from($vec([lo, hi]))
            }
        }
    };
}

/*----------------------------------------------------------------*/

def_big_vec!(u8x64, u8x32, 32, u8, [u8; 64]);
impl_big_conv! {
    u8x64,
    to_u16x32 | to_u16x16 => u16x32;
    to_u32x16 | to_u32x8 => u32x16;
    to_u64x8 | to_u64x4 => u64x8;
}
impl_big_cmp! {
    u8x64,
    u8x32,
    Mask8x64
}
impl u8x64 {
    #[inline]
    pub fn extract16<const INDEX: usize>(self) -> u8x16 {
        match INDEX {
            0 => self.0[0].extract16::<0>(),
            1 => self.0[0].extract16::<1>(),
            2 => self.0[1].extract16::<0>(),
            3 => self.0[1].extract16::<1>(),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extract32<const INDEX: usize>(self) -> u8x32 {
        self.0[INDEX]
    }

    #[inline]
    pub fn zero_ext(self) -> u16x64 {
        let lo = self.0[0].zero_ext();
        let hi = self.0[1].zero_ext();
        u16x64([lo, hi])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn flip_rays(self) -> u8x64 {
        u8x64([self.0[1], self.0[0]])
    }

    #[inline]
    pub fn extend_rays(self) -> u8x64 {
        unsafe {
            let zero = u8x32::splat(0);
            let lo: u8x32 = _mm256_sad_epu8(self.0[0].0, zero.0).into();
            let hi: u8x32 = _mm256_sad_epu8(self.0[1].0, zero.0).into();
            let index = u8x32::from([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
                0x08, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x18, 0x18, 0x18, 0x18,
                0x18, 0x18, 0x18, 0x18,
            ]);

            u8x64([lo.shuffle(index), hi.shuffle(index)])
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask(self, mask: Mask8x64) -> u8x64 {
        self & mask.expand()
    }

    #[inline]
    pub fn blend(a: u8x64, b: u8x64, mask: Mask8x64) -> u8x64 {
        let mask = mask.expand();
        let lo = u8x32::blend(a.0[0], b.0[0], Mask8x32::from(mask.0[0]));
        let hi = u8x32::blend(a.0[1], b.0[1], Mask8x32::from(mask.0[1]));
        u8x64([lo, hi])
    }

    #[inline]
    pub fn compress(self, mask: Mask8x64) -> u8x64 {
        let mut values = [0u8; 64];
        unsafe {
            self.store(values.as_mut_ptr());
        }

        let mut mask = mask.to_bitmask();
        let mut temp = [0u8; 64];
        let mut cursor = 0;

        while mask != 0 {
            temp[cursor] = values[mask.trailing_zeros() as usize];
            mask &= mask.wrapping_sub(1);
            cursor += 1;
        }

        u8x64::from(temp)
    }

    #[inline]
    pub unsafe fn compress_store<T>(self, mask: Mask8x64, dest: *mut T) {
        unsafe { self.compress(mask).store(dest) }
    }

    #[inline]
    pub fn permute(self, index: u8x64) -> u8x64 {
        let mask_lo = Mask8x32::from(index.0[0].to_u16x16().shl::<2>().to_u8x32());
        let mask_hi = Mask8x32::from(index.0[1].to_u16x16().shl::<2>().to_u8x32());
        let lo = u8x32::blend(
            self.0[0].permute(index.0[0]),
            self.0[1].permute(index.0[0]),
            mask_lo,
        );
        let hi = u8x32::blend(
            self.0[0].permute(index.0[1]),
            self.0[1].permute(index.0[1]),
            mask_hi,
        );
        u8x64([lo, hi])
    }

    #[inline]
    pub fn shuffle(self, index: u8x64) -> u8x64 {
        let lo = self.0[0].shuffle(index.0[0]);
        let hi = self.0[1].shuffle(index.0[1]);
        u8x64([lo, hi])
    }
}

def_big_vec!(u16x32, u16x16, 32, u16, [u16; 32]);
impl_big_conv! {
    u16x32,
    to_u8x64 | to_u8x32 => u8x64;
    to_u32x16 | to_u32x8 => u32x16;
    to_u64x8 | to_u64x4 => u64x8;
}
impl_big_cmp! {
    u16x32,
    u16x16,
    Mask16x32
}
impl_select! {
    u16x32,
    Mask16x32,
    [u16; 32]
}
impl u16x32 {
    #[inline]
    pub fn extract8<const INDEX: i32>(self) -> u16x8 {
        match INDEX {
            0 => self.0[0].extract8::<0>(),
            1 => self.0[0].extract8::<1>(),
            2 => self.0[1].extract8::<0>(),
            3 => self.0[1].extract8::<1>(),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extract16<const INDEX: usize>(self) -> u16x16 {
        self.0[INDEX]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl<const SHIFT: i32>(self) -> u16x32 {
        let lo = self.0[0].shl::<SHIFT>();
        let hi = self.0[1].shl::<SHIFT>();
        u16x32([lo, hi])
    }

    #[inline]
    pub fn shlv(self, shift: u16x32) -> u16x32 {
        let lo = self.0[0].shlv(shift.0[0]);
        let hi = self.0[1].shlv(shift.0[1]);
        u16x32([lo, hi])
    }

    #[inline]
    pub fn shr<const SHIFT: i32>(self) -> u16x32 {
        let lo = self.0[0].shr::<SHIFT>();
        let hi = self.0[1].shr::<SHIFT>();
        u16x32([lo, hi])
    }

    #[inline]
    pub fn shrv(self, shift: u16x32) -> u16x32 {
        let lo = self.0[0].shrv(shift.0[0]);
        let hi = self.0[1].shrv(shift.0[1]);
        u16x32([lo, hi])
    }
}

def_big_vec!(u32x16, u32x8, 32, u32, [u32; 16]);
impl_big_conv! {
    u32x16,
    to_u8x64 | to_u8x32 => u8x64;
    to_u16x32 | to_u16x16 => u16x32;
    to_u64x8 | to_u64x4 => u64x8;
}
impl_big_cmp! {
    u32x16,
    u32x8,
    Mask32x16
}
impl_select! {
    u32x16,
    Mask32x16,
    [u32; 16]
}
impl u32x16 {
    #[inline]
    pub fn extract4<const INDEX: usize>(self) -> u32x4 {
        match INDEX {
            0 => self.0[0].extract4::<0>(),
            1 => self.0[0].extract4::<1>(),
            2 => self.0[1].extract4::<0>(),
            3 => self.0[1].extract4::<1>(),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extract8<const INDEX: usize>(self) -> u32x8 {
        self.0[INDEX]
    }
}

def_big_vec!(u64x8, u64x4, 32, u64, [u64; 8]);
impl_big_conv! {
    u64x8,
    to_u8x64 | to_u8x32 => u8x64;
    to_u16x32 | to_u16x16 => u16x32;
    to_u32x16 | to_u32x8 => u32x16;
}
impl_big_cmp! {
    u64x8,
    u64x4,
    Mask64x8
}
impl_select! {
    u64x8,
    Mask64x8,
    [u64; 8]
}
impl u64x8 {
    #[inline]
    pub fn extract2<const INDEX: usize>(self) -> u64x2 {
        match INDEX {
            0 => self.0[0].extract2::<0>(),
            1 => self.0[0].extract2::<1>(),
            2 => self.0[1].extract2::<0>(),
            3 => self.0[1].extract2::<1>(),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extract4<const INDEX: usize>(self) -> u64x4 {
        self.0[INDEX]
    }
}

def_big_vec!(u16x64, u16x32, 64, u16, [u16; 64]);
impl_big_cmp! {
    u16x64,
    u16x32,
    Mask16x64
}
impl u16x64 {
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
    pub fn shl<const SHIFT: i32>(self) -> u16x64 {
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
    pub fn shr<const SHIFT: i32>(self) -> u16x64 {
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
        self & mask.expand()
    }

    #[inline]
    pub fn blend(a: u16x64, b: u16x64, mask: Mask16x64) -> u16x64 {
        let mask_vec = mask.expand();
        (mask_vec & b) | mask_vec.andnot(a)
    }

    #[inline]
    pub fn compress(self, mask: Mask16x64) -> u16x64 {
        let mut values = [0u16; 64];
        unsafe {
            self.store(values.as_mut_ptr());
        }

        let mut mask = mask.to_bitmask();
        let mut temp = [0u16; 64];
        let mut cursor = 0;

        while mask != 0 {
            temp[cursor] = values[mask.trailing_zeros() as usize];
            mask &= mask.wrapping_sub(1);
            cursor += 1;
        }

        u16x64::from(temp)
    }

    #[inline]
    pub unsafe fn compress_store<T>(self, mask: Mask16x64, dest: *mut T) {
        unsafe { self.compress(mask).store(dest) }
    }
}

/*----------------------------------------------------------------*/

def_big_vec!(i16x32, u16x16, 32, i16, [i16; 32]);
impl i16x32 {
    #[inline]
    pub fn clamp(self, min: i16x32, max: i16x32) -> i16x32 {
        unsafe {
            let lo = _mm256_min_epi16(_mm256_max_epi16(self.0[0].0, min.0[0].0), max.0[0].0).into();
            let hi = _mm256_min_epi16(_mm256_max_epi16(self.0[1].0, min.0[1].0), max.0[1].0).into();
            i16x32([lo, hi])
        }
    }

    #[inline]
    pub fn madd(self, rhs: i16x32) -> i32x16 {
        unsafe {
            let lo = _mm256_madd_epi16(self.0[0].0, rhs.0[0].0).into();
            let hi = _mm256_madd_epi16(self.0[1].0, rhs.0[1].0).into();
            i32x16([lo, hi])
        }
    }
}
impl Add for i16x32 {
    type Output = i16x32;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe {
            let lo = _mm256_add_epi16(self.0[0].0, rhs.0[0].0).into();
            let hi = _mm256_add_epi16(self.0[1].0, rhs.0[1].0).into();
            i16x32([lo, hi])
        }
    }
}
impl Sub for i16x32 {
    type Output = i16x32;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe {
            let lo = _mm256_sub_epi16(self.0[0].0, rhs.0[0].0).into();
            let hi = _mm256_sub_epi16(self.0[1].0, rhs.0[1].0).into();
            i16x32([lo, hi])
        }
    }
}
impl Mul for i16x32 {
    type Output = i16x32;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            let lo = _mm256_mullo_epi16(self.0[0].0, rhs.0[0].0).into();
            let hi = _mm256_mullo_epi16(self.0[1].0, rhs.0[1].0).into();
            i16x32([lo, hi])
        }
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

def_big_vec!(i32x16, u32x8, 32, i32, [i32; 16]);
impl i32x16 {
    #[inline]
    pub fn reduce_sum(self) -> i32 {
        let mut temp = [0i32; 16];
        unsafe {
            self.store(temp.as_mut_ptr());
        }
        temp.iter().sum()
    }
}
impl Add for i32x16 {
    type Output = i32x16;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe {
            let lo = _mm256_add_epi32(self.0[0].0, rhs.0[0].0).into();
            let hi = _mm256_add_epi32(self.0[1].0, rhs.0[1].0).into();
            i32x16([lo, hi])
        }
    }
}
impl Sub for i32x16 {
    type Output = i32x16;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe {
            let lo = _mm256_sub_epi32(self.0[0].0, rhs.0[0].0).into();
            let hi = _mm256_sub_epi32(self.0[1].0, rhs.0[1].0).into();
            i32x16([lo, hi])
        }
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
