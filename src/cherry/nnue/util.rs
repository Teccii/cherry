use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, align(64))]
pub struct Align64<T>(pub T);

impl<T> std::ops::Deref for Align64<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Align64<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/*----------------------------------------------------------------*/

pub fn feed_forward(
    stm: &[i16; HL],
    ntm: &[i16; HL],
    bucket: usize,
    weights: &NetworkWeights,
    output: &mut i32,
) {
    let bucket_offset = bucket * L1;
    let out_weights = &weights.out_weights;
    let (zero, qa) = (NativeVec::zero(), NativeVec::splat16(QA as u16));
    let mut sum = zero;

    for i in 0..(HL / NativeVec::CHUNKS_16) {
        let offset = i * NativeVec::CHUNKS_16;

        unsafe {
            let stm = NativeVec::clamp16(NativeVec::load(stm.as_ptr().add(offset)), zero, qa);
            let ntm = NativeVec::clamp16(NativeVec::load(ntm.as_ptr().add(offset)), zero, qa);
            let stm_weight = NativeVec::load(out_weights.as_ptr().add(bucket_offset + offset));
            let ntm_weight = NativeVec::load(out_weights.as_ptr().add(bucket_offset + HL + offset));

            sum = NativeVec::add32(
                sum,
                NativeVec::madd16(NativeVec::mullo16(stm_weight, stm), stm),
            );
            sum = NativeVec::add32(
                sum,
                NativeVec::madd16(NativeVec::mullo16(ntm_weight, ntm), ntm),
            );
        }
    }

    *output = sum.reduce_add32();
}
