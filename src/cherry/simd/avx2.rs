use core::{arch::x86_64::*, ops::*};
use std::ptr;
use super::common::*;

/*----------------------------------------------------------------*/

#[inline]
pub fn interleave64(a: u32, b: u32) -> u64 {
    ((b as u64) << 32) | a as u64
}

/*----------------------------------------------------------------*/

pub type NativeVec = Vec256;

/*----------------------------------------------------------------*/

impl Vec128 {
    #[inline]
    pub fn mask8(mask: Vec128Mask8, vec: Vec128) -> Vec128 {
        vec & Vec128::expand_mask8(mask)
    }

    #[inline]
    pub fn mask16(mask: Vec128Mask16, vec: Vec128) -> Vec128 {
        vec & Vec128::expand_mask16(mask)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress_store16<T>(dest: *mut T, mut mask: Vec128Mask16, vec: Vec128) {
        let mut values = [0u16; Self::CHUNKS_16];
        unsafe { Vec128::store(values.as_mut_ptr(), vec); }

        let mut temp = [0u16; Self::CHUNKS_16];
        let mut cursor = 0;

        while mask != 0 {
            temp[cursor] = values[mask.trailing_zeros() as usize];
            mask &= mask.wrapping_sub(1);
            cursor += 1;
        }

        unsafe { ptr::copy_nonoverlapping(temp.as_ptr().cast::<i8>(), dest.cast(), cursor * size_of::<u16>()); }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shuffle8(a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_shuffle_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn blend8(mask: Vec128, a: Vec128, b: Vec128) -> Vec128 {
        unsafe { _mm_blendv_epi8(a.raw, b.raw, mask.raw).into() }
    }

    #[inline]
    pub fn bitshuffle(a: Vec128, b: Vec128) -> Vec128Mask8 {
        #[inline]
        fn shl8(a: Vec128, b: Vec128) -> Vec128 {
            let b = Vec128::shl16::<5>(b);
            let shl = Vec128::splat8(0xf0u8) & Vec128::shl16::<4>(a);
            let a = Vec128::blend8(b, a, shl);

            let b = Vec128::add8(b, b);
            let shl = Vec128::splat8(0xfcu8) & Vec128::shl16::<2>(a);
            let a = Vec128::blend8(b, a, shl);

            let b = Vec128::add8(b, b);
            let shl = Vec128::splat8(0xfeu8) & Vec128::shl16::<1>(a);
            let a = Vec128::blend8(b, a, shl);

            a
        }

        let idx = b & Vec128::splat8(0x3f);
        let byte_idx = Vec128::add8(
            Vec128::shr16::<3>(idx) & Vec128::splat8(7),
            unsafe { _mm_set_epi64x(0x0808080808080808, 0).into() }
        );
        let shuffled = Vec128::shuffle8(a, byte_idx);
        let shift = Vec128::splat8(7) ^ (idx & Vec128::splat8(7));

        shl8(shuffled, shift).msb8()
    }

    #[inline]
    pub fn mask_bitshuffle(mask: Vec128Mask8, a: Vec128, b: Vec128) -> Vec128Mask8 {
        mask & Vec128::bitshuffle(a, b)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn msb8(self) -> Vec128Mask8 {
        unsafe { _mm_movemask_epi8(self.raw) as Vec128Mask8 }
    }

    #[inline]
    pub fn nonzero8(self) -> Vec128Mask8 {
        Vec128::neq8(self, Vec128::zero())
    }

    #[inline]
    pub fn eq8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        Vec128::from(unsafe { _mm_cmpeq_epi8(a.raw, b.raw) }).msb8()
    }

    #[inline]
    pub fn neq8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        !Vec128::eq8(a, b)
    }

    #[inline]
    pub fn testn8(a: Vec128, b: Vec128) -> Vec128Mask8 {
        Vec128::eq8(a & b, Vec128::zero())
    }
    
    /*----------------------------------------------------------------*/

    #[inline]
    pub fn expand_mask8(mask: Vec128Mask8) -> Vec128 {
        unsafe {
            let vec = _mm_cvtsi32_si128(mask as i32);
            let shuffled = _mm_shuffle_epi8(vec, _mm_set_epi64x(0x0101010101010101, 0));
            let and_mask = _mm_set1_epi64x(0x8040201008040201u64 as i64);

            _mm_cmpeq_epi8(and_mask, _mm_and_si128(and_mask, shuffled)).into()
        }
    }

    #[inline]
    pub fn expand_mask16(mask: Vec128Mask16) -> Vec128 {
        unsafe {
            let vec = _mm_set1_epi16(mask as i16);
            let and_mask = _mm_setr_epi16(0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80);

            _mm_cmpeq_epi16(and_mask, _mm_and_si128(vec, and_mask)).into()
        }
    }
}

/*----------------------------------------------------------------*/

impl Vec256 {
    #[inline]
    pub fn min16(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_min_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn max16(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_max_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn clamp16(value: Vec256, min: Vec256, max: Vec256) -> Vec256 {
        Vec256::max16(Vec256::min16(value, max), min)
    }

    #[inline]
    pub fn madd16(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_madd_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn mullo16(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_mullo_epi16(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn reduce_add32(self) -> i32 {
        let mut temp = [0i32; Self::CHUNKS_32];
        unsafe { Vec256::store(temp.as_mut_ptr(), self); }

        temp.iter().sum()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shlv16(vec: Vec256, shift: Vec256) -> Vec256 {
        unsafe {
            let zero = _mm256_setzero_si256();
            let vec_lo = _mm256_unpacklo_epi16(zero, vec.raw);
            let vec_hi = _mm256_unpackhi_epi16(zero, vec.raw);
            let shift_lo = _mm256_unpacklo_epi16(shift.raw, zero);
            let shift_hi = _mm256_unpackhi_epi16(shift.raw, zero);
            let result_lo = _mm256_srli_epi32(_mm256_sllv_epi32(vec_lo, shift_lo), 16);
            let result_hi = _mm256_srli_epi32(_mm256_sllv_epi32(vec_hi, shift_hi), 16);

            _mm256_packus_epi32(result_lo, result_hi).into()
        }
    }

    #[inline]
    pub fn shl64<const SHIFT: i32>(vec: Vec256) -> Vec256 {
        unsafe { _mm256_slli_epi64::<SHIFT>(vec.raw).into() }
    }

    #[inline]
    pub fn shr64<const SHIFT: i32>(vec: Vec256) -> Vec256 {
        unsafe { _mm256_srli_epi64::<SHIFT>(vec.raw).into() }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress_store16<T>(dest: *mut T, mut mask: Vec256Mask16, vec: Vec256) {
        let mut values = [0u16; Self::CHUNKS_16];
        unsafe { Vec256::store(values.as_mut_ptr(), vec); }

        let mut temp = [0u16; Self::CHUNKS_16];
        let mut cursor = 0;

        while mask != 0 {
            temp[cursor] = values[mask.trailing_zeros() as usize];
            mask &= mask.wrapping_sub(1);
            cursor += 1;
        }

        unsafe { ptr::copy_nonoverlapping(temp.as_ptr().cast::<i8>(), dest.cast(), cursor * size_of::<u16>()); }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shuffle8(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_shuffle_epi8(a.raw, b.raw).into() }
    }

    #[inline]
    pub fn blend8(mask: Vec256, a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_blendv_epi8(a.raw, b.raw, mask.raw).into() }
    }

    #[inline]
    pub fn permute8(index: Vec256, vec: Vec256) -> Vec256 {
        let mask = Vec256::shl16::<3>(index);
        let index = index & Vec256::splat8(15);
        let lo = unsafe { _mm256_permute2x128_si256::<0x00>(vec.raw, vec.raw).into() };
        let hi = unsafe { _mm256_permute2x128_si256::<0x11>(vec.raw, vec.raw).into() };

        Vec256::blend8(
            mask,
            Vec256::shuffle8(lo, index),
            Vec256::shuffle8(hi, index)
        )
    }

    #[inline]
    pub fn bitshuffle(a: Vec256, b: Vec256) -> Vec256Mask8 {
        #[inline]
        fn shlv8_overflowing(vec: __m256i, shift: __m256i) -> __m256i {
            unsafe {
                let vec = _mm256_blendv_epi8(vec, _mm256_slli_epi16(vec, 4), _mm256_slli_epi16(shift, 5));
                let vec = _mm256_blendv_epi8(vec, _mm256_slli_epi16(vec, 2), _mm256_slli_epi16(shift, 6));
                let vec = _mm256_blendv_epi8(vec, _mm256_slli_epi16(vec, 1), _mm256_slli_epi16(shift, 7));

                vec
            }
        }

        unsafe {
            let byte_idx = _mm256_add_epi8(
                _mm256_and_si256(_mm256_srli_epi16(b.raw, 3), _mm256_set1_epi8(7)),
                _mm256_setr_epi64x(
                    0x0000000000000000, 0x0808080808080808,
                    0x0000000000000000, 0x0808080808080808
                )
            );
            let shuffled = _mm256_shuffle_epi8(a.raw, byte_idx);
            let shift = _mm256_andnot_si256(b.raw, _mm256_set1_epi8(7));
            let shifted = shlv8_overflowing(shuffled, shift);

            _mm256_movemask_epi8(shifted) as __mmask32
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn lane_splat8to64(vec: Vec256) -> Vec256 {
        const fn xor_indices(xor: u8) -> [u8; 16] {
            let mut arr = [0; 16];
            let mut i = 0;
            while i < 16 {
                arr[i] = (i as u8) ^ xor;
                i += 1;
            }
            arr
        }

        unsafe {
            let idx1 = _mm256_broadcastsi128_si256(_mm_loadu_si128(const { xor_indices(1) }.as_ptr().cast()));
            let idx2 = _mm256_broadcastsi128_si256(_mm_loadu_si128(const { xor_indices(2) }.as_ptr().cast()));

            let vec = _mm256_xor_si256(vec.raw, _mm256_shuffle_epi8(vec.raw, idx1));
            let vec = _mm256_xor_si256(vec, _mm256_shuffle_epi8(vec, idx2));

            _mm256_xor_si256(vec, _mm256_shuffle_epi32(vec, 0xb1)).into()
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn msb8(self) -> Vec256Mask8 {
        unsafe { _mm256_movemask_epi8(self.raw) as Vec256Mask8 }
    }

    #[inline]
    pub fn eq8_vm(a: Vec256, b: Vec256) -> Vec256 {
        unsafe { _mm256_cmpeq_epi8(a.raw, b.raw).into() }
    }
    
    #[inline]
    pub fn eq8(a: Vec256, b: Vec256) -> Vec256Mask8 {
        Vec256::eq8_vm(a, b).msb8()
    }

    #[inline]
    pub fn neq8(a: Vec256, b: Vec256) -> Vec256Mask8 {
        !Vec256::eq8(a, b)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn expand_mask16(mask: Vec256Mask16) -> Vec256 {
        unsafe {
            let vec = _mm256_set1_epi16(mask as i16);
            let and_mask = _mm256_setr_epi16(
                0x0001, 0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0040, 0x0080,
                0x0100, 0x0200, 0x0400, 0x0800, 0x1000, 0x2000, 0x4000, 0x8000u16 as i16
            );

            _mm256_cmpeq_epi16(and_mask, _mm256_and_si256(vec, and_mask)).into()
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zext8to16(self) -> Vec512 {
        let lo = self.into_vec128().zext8to16();
        let hi = self.extract_vec128::<1>().zext8to16();

        Vec512::from([lo, hi])
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Vec512 {
    pub raw: [Vec256; 2]
}

impl Vec512 {
    #[inline]
    pub unsafe fn load<T>(src: *const T) -> Vec512 {
        Vec512::from([
            unsafe { Vec256::load(src) },
            unsafe { Vec256::load(src.byte_add(32)) }
        ])
    }

    #[inline]
    pub unsafe fn store<T>(dst: *mut T, src: Vec512) {
        unsafe {
            Vec256::store(dst, src.raw[0]);
            Vec256::store(dst.byte_add(32), src.raw[1]);
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn into_u32(self) -> u32 {
        self.raw[0].into_u32()
    }

    #[inline]
    pub fn into_vec128(self) -> Vec128 {
        self.raw[0].into_vec128()
    }

    #[inline]
    pub fn into_vec256(self) -> Vec256 {
        self.raw[0]
    }

    #[inline]
    pub fn extract_vec256<const INDEX: usize>(self) -> Vec256 {
        self.raw[INDEX]
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn zero() -> Vec512 {
        Vec512::from([Vec256::zero(), Vec256::zero()])
    }

    #[inline]
    pub fn splat8(value: u8) -> Vec512 {
        Vec512::from([Vec256::splat8(value), Vec256::splat8(value)])
    }

    #[inline]
    pub fn splat16(value: u16) -> Vec512 {
        Vec512::from([Vec256::splat16(value), Vec256::splat16(value)])
    }

    #[inline]
    pub fn splat32(value: u32) -> Vec512 {
        Vec512::from([Vec256::splat32(value), Vec256::splat32(value)])
    }

    #[inline]
    pub fn splat64(value: u64) -> Vec512 {
        Vec512::from([Vec256::splat64(value), Vec256::splat64(value)])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask_splat8(vec: Vec512, mask: Vec512Mask8, value: u8) -> Vec512 {
        Vec512::blend8(mask, vec, Vec512::splat8(value))
    }
    
    /*----------------------------------------------------------------*/

    #[inline]
    pub fn mask8(mask: Vec512Mask8, vec: Vec512) -> Vec512 {
        vec & Vec512::expand_mask8(mask)
    }

    #[inline]
    pub fn mask16(mask: Vec512Mask16, vec: Vec512) -> Vec512 {
        vec & Vec512::expand_mask16(mask)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn add8(a: Vec512, b: Vec512) -> Vec512 {
        Vec512::from([Vec256::add8(a.raw[0], b.raw[0]), Vec256::add8(a.raw[1], b.raw[1])])
    }

    #[inline]
    pub fn add16(a: Vec512, b: Vec512) -> Vec512 {
        Vec512::from([Vec256::add16(a.raw[0], b.raw[0]), Vec256::add16(a.raw[1], b.raw[1])])
    }

    #[inline]
    pub fn add32(a: Vec512, b: Vec512) -> Vec512 {
        Vec512::from([Vec256::add32(a.raw[0], b.raw[0]), Vec256::add32(a.raw[1], b.raw[1])])
    }

    #[inline]
    pub fn add64(a: Vec512, b: Vec512) -> Vec512 {
        Vec512::from([Vec256::add64(a.raw[0], b.raw[0]), Vec256::add64(a.raw[1], b.raw[1])])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn sub8(a: Vec512, b: Vec512) -> Vec512 {
        Vec512::from([Vec256::sub8(a.raw[0], b.raw[0]), Vec256::sub8(a.raw[1], b.raw[1])])
    }

    #[inline]
    pub fn sub16(a: Vec512, b: Vec512) -> Vec512 {
        Vec512::from([Vec256::sub16(a.raw[0], b.raw[0]), Vec256::sub16(a.raw[1], b.raw[1])])
    }

    #[inline]
    pub fn sub32(a: Vec512, b: Vec512) -> Vec512 {
        Vec512::from([Vec256::sub32(a.raw[0], b.raw[0]), Vec256::sub32(a.raw[1], b.raw[1])])
    }

    #[inline]
    pub fn sub64(a: Vec512, b: Vec512) -> Vec512 {
        Vec512::from([Vec256::sub64(a.raw[0], b.raw[0]), Vec256::sub64(a.raw[1], b.raw[1])])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn shl16<const SHIFT: i32>(vec: Vec512) -> Vec512 {
        Vec512::from([Vec256::shl16::<SHIFT>(vec.raw[0]), Vec256::shl16::<SHIFT>(vec.raw[1])])
    }

    #[inline]
    pub fn shr16<const SHIFT: i32>(vec: Vec512) -> Vec512 {
        Vec512::from([Vec256::shr16::<SHIFT>(vec.raw[0]), Vec256::shr16::<SHIFT>(vec.raw[1])])
    }

    #[inline]
    pub fn shlv16(vec: Vec512, shift: Vec512) -> Vec512 {
        Vec512::from([Vec256::shlv16(vec.raw[0], shift.raw[0]), Vec256::shlv16(vec.raw[1], shift.raw[1])])
    }

    #[inline]
    pub fn shlv16_mz(mask: Vec512Mask16, vec: Vec512, shift: Vec512) -> Vec512 {
        Vec512::mask16(mask, Vec512::shlv16(vec, shift))
    }

    #[inline]
    pub fn shl64<const SHIFT: i32>(vec: Vec512) -> Vec512 {
        Vec512::from([Vec256::shl64::<SHIFT>(vec.raw[0]), Vec256::shl64::<SHIFT>(vec.raw[1])])
    }

    #[inline]
    pub fn shr64<const SHIFT: i32>(vec: Vec512) -> Vec512 {
        Vec512::from([Vec256::shr64::<SHIFT>(vec.raw[0]), Vec256::shr64::<SHIFT>(vec.raw[1])])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn compress8(mut mask: Vec512Mask8, vec: Vec512) -> Vec512 {
        let mut values = [0u8; Self::CHUNKS_8];
        unsafe { Vec512::store(values.as_mut_ptr(), vec); }

        let mut temp = [0u8; Self::CHUNKS_8];
        let mut cursor = 0;

        while mask != 0 {
            temp[cursor] = values[mask.trailing_zeros() as usize];
            mask &= mask.wrapping_sub(1);
            cursor += 1;
        }

        Vec512::from(temp)
    }

    #[inline]
    pub fn compress_store16<T>(dest: *mut T, mut mask: Vec512Mask16, vec: Vec512) {
        let mut values = [0u16; Self::CHUNKS_16];
        unsafe { Vec512::store(values.as_mut_ptr(), vec); }

        let mut temp = [0u16; Self::CHUNKS_16];
        let mut cursor = 0;

        while mask != 0 {
            temp[cursor] = values[mask.trailing_zeros() as usize];
            mask &= mask.wrapping_sub(1);
            cursor += 1;
        }

        unsafe { ptr::copy_nonoverlapping(temp.as_ptr().cast::<i8>(), dest.cast(), cursor * size_of::<u16>()); }
    }

    #[inline]
    pub fn compress_store64<T>(dest: *mut T, mut mask: Vec512Mask64, vec: Vec512) {
        let mut values = [0u64; Self::CHUNKS_64];
        unsafe { Vec512::store(values.as_mut_ptr(), vec); }

        let mut temp = [0u64; Self::CHUNKS_64];
        let mut cursor = 0;

        while mask != 0 {
            temp[cursor] = values[mask.trailing_zeros() as usize];
            mask &= mask.wrapping_sub(1);
            cursor += 1;
        }

        unsafe { ptr::copy_nonoverlapping(temp.as_ptr().cast::<i8>(), dest.cast(), cursor * size_of::<u64>()); }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn blend8(mask: Vec512Mask8, a: Vec512, b: Vec512) -> Vec512 {
        let mask = Vec512::expand_mask8(mask);
        
        Vec512::from([
            Vec256::blend8(mask.raw[0], a.raw[0], b.raw[0]),
            Vec256::blend8(mask.raw[1], a.raw[1], b.raw[1])
        ])
    }
    
    #[inline]
    pub fn permute8(index: Vec512, vec: Vec512) -> Vec512 {
        let mask_lo = Vec256::shl16::<2>(index.raw[0]);
        let mask_hi = Vec256::shl16::<2>(index.raw[1]);

        let lo = Vec256::blend8(
            mask_lo,
            Vec256::permute8(index.raw[0], vec.raw[0]),
            Vec256::permute8(index.raw[0], vec.raw[1]),
        );
        let hi = Vec256::blend8(
            mask_hi,
            Vec256::permute8(index.raw[1], vec.raw[0]),
            Vec256::permute8(index.raw[1], vec.raw[1]),
        );

        Vec512::from([lo, hi])
    }

    #[inline]
    pub fn permute8_mz(mask: Vec512Mask8, index: Vec512, vec: Vec512) -> Vec512 {
        Vec512::mask8(mask, Vec512::permute8(index, vec))
    }

    #[inline]
    pub fn permute8_128(index: Vec512, vec: Vec128) -> Vec512 {
        let vec = unsafe { _mm256_broadcastsi128_si256(vec.raw).into() };
        let lo = Vec256::shuffle8(vec, index.raw[0]);
        let hi = Vec256::shuffle8(vec, index.raw[1]);

        Vec512::from([lo, hi])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn bitshuffle(a: Vec512, b: Vec512) -> Vec512Mask8 {
        ((Vec256::bitshuffle(a.raw[1], b.raw[1]) as Vec512Mask8) << 32) | Vec256::bitshuffle(a.raw[0], b.raw[0]) as Vec512Mask8
    }

    #[inline]
    pub fn mask_bitshuffle(mask: Vec512Mask8, a: Vec512, b: Vec512) -> Vec512Mask8 {
        mask & Vec512::bitshuffle(a, b)
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn lane_splat8to64(vec: Vec512) -> Vec512 {
        Vec512::from([Vec256::lane_splat8to64(vec.raw[0]), Vec256::lane_splat8to64(vec.raw[1])])
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn msb8(self) -> Vec512Mask8 {
        interleave64(self.raw[0].msb8(), self.raw[1].msb8())
    }

    #[inline]
    pub fn zero8(self) -> Vec512Mask8 {
        Vec512::eq8(self, Vec512::zero())
    }

    #[inline]
    pub fn zero16(self) -> Vec512Mask16 {
        Vec512::eq16(self, Vec512::zero())
    }

    #[inline]
    pub fn nonzero8(self) -> Vec512Mask8 {
        Vec512::neq8(self, Vec512::zero())
    }

    #[inline]
    pub fn nonzero16(self) -> Vec512Mask16 {
        Vec512::neq16(self, Vec512::zero())
    }

    #[inline]
    pub fn nonzero64(self) -> Vec512Mask64 {
        Vec512::neq64(self, Vec512::zero())
    }

    #[inline]
    pub fn eq8(a: Vec512, b: Vec512) -> Vec512Mask8 {
        interleave64(
            Vec256::eq8(a.raw[0], b.raw[0]),
            Vec256::eq8(a.raw[1], b.raw[1])
        )
    }

    #[inline]
    pub fn eq16(a: Vec512, b: Vec512) -> Vec512Mask16 {
        let x = interleave64(
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(a.raw[0].raw, b.raw[0].raw)) as Vec256Mask8 },
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(a.raw[1].raw, b.raw[1].raw)) as Vec256Mask8 },
        );

        unsafe { _pext_u64(x, 0xAAAAAAAAAAAAAAAA) as Vec512Mask16 }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn neq8(a: Vec512, b: Vec512) -> Vec512Mask8 {
        interleave64(
            Vec256::neq8(a.raw[0], b.raw[0]),
            Vec256::neq8(a.raw[1], b.raw[1])
        )
    }

    #[inline]
    pub fn neq16(a: Vec512, b: Vec512) -> Vec512Mask16 {
        let x = interleave64(
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(a.raw[0].raw, b.raw[0].raw)) as Vec256Mask8 },
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(a.raw[1].raw, b.raw[1].raw)) as Vec256Mask8 },
        );

        unsafe { _pext_u64(!x, 0xAAAAAAAAAAAAAAAA) as Vec512Mask16 }
    }

    #[inline]
    pub fn neq64(a: Vec512, b: Vec512) -> Vec512Mask64 {
        let x = interleave64(
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi64(a.raw[0].raw, b.raw[0].raw)) as Vec256Mask8 },
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi64(a.raw[1].raw, b.raw[1].raw)) as Vec256Mask8 },
        );

        unsafe { _pext_u64(!x, 0x8080808080808080) as Vec512Mask64 }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn test16(a: Vec512, b: Vec512) -> Vec512Mask16 {
        Vec512::neq16(a & b, Vec512::zero())
    }

    #[inline]
    pub fn testn8(a: Vec512, b: Vec512) -> Vec512Mask8 {
        Vec512::eq8(a & b, Vec512::zero())
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn expand_mask8(mask: Vec512Mask8) -> Vec512 {
        let shuffled0 = Vec256::shuffle8(
            Vec256::splat32(mask as u32),
            unsafe {
                _mm256_setr_epi64x(
                    0x0000000000000000,
                    0x0101010101010101,
                    0x0202020202020202,
                    0x0303030303030303,
                ).into()
            }
        );
        let shuffled1 = Vec256::shuffle8(
            Vec256::splat32((mask >> 32) as u32),
            unsafe {
                _mm256_setr_epi64x(
                    0x0000000000000000,
                    0x0101010101010101,
                    0x0202020202020202,
                    0x0303030303030303,
                ).into()
            }
        );
        
        let and_mask = Vec256::splat64(0x8040201008040201);
        
        Vec512::from([
            Vec256::eq8_vm(and_mask, and_mask & shuffled0),
            Vec256::eq8_vm(and_mask, and_mask & shuffled1),
        ])
    }

    #[inline]
    pub fn expand_mask16(mask: Vec512Mask16) -> Vec512 {
        Vec512::from([
            Vec256::expand_mask16(mask as Vec256Mask16),
            Vec256::expand_mask16((mask >> 16) as Vec256Mask16)
        ])
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
        Vec512 { raw: [ Vec256::from(vec), Vec256::zero() ] }
    }
}

impl From<Vec256> for Vec512 {
    #[inline]
    fn from(vec: Vec256) -> Vec512 {
        Vec512 { raw: [vec, Vec256::zero()] }
    }
}

impl From<[Vec256; 2]> for Vec512 {
    #[inline]
    fn from(arr: [Vec256; 2]) -> Vec512 {
        Vec512 { raw: arr }
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
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for Vec512 {
            type Output = Self;

            #[inline]
            fn $fn(self, other: Vec512) -> Vec512 {
                Vec512::from([
                    self.raw[0].$fn(other.raw[0]),
                    self.raw[1].$fn(other.raw[1]),
                ])
            }
        }
    )*}
}

macro_rules! impl_vec512_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for Vec512 {
            #[inline]
            fn $fn(&mut self, other: Vec512) {
                self.raw[0].$fn(other.raw[0]);
                self.raw[1].$fn(other.raw[1]);
            }
        }
    )*}
}

impl_vec512_ops! {
    BitAnd, bitand;
    BitOr, bitor;
    BitXor, bitxor;
}

impl_vec512_assign_ops! {
    BitAndAssign, bitand_assign;
    BitOrAssign, bitor_assign;
    BitXorAssign, bitxor_assign;
}