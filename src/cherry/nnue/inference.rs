use crate::*;

pub fn feed_forward(
    stm: &[i16; HL],
    ntm: &[i16; HL],
    bucket: usize,
    weights: &NetworkWeights,
    output: &mut i32,
) {
    let bucket_offset = bucket * L1;
    let out_weights = &weights.out_weights;
    let (zero, qa) = (i16x32::splat(0), i16x32::splat(QA as i16));
    let mut sum = i32x16::splat(0);

    for i in 0..(HL / 32) {
        let offset = i * 32;

        unsafe {
            let stm = i16x32::load(stm.as_ptr().add(offset)).clamp(zero, qa);
            let ntm = i16x32::load(ntm.as_ptr().add(offset)).clamp(zero, qa);
            let stm_weight = i16x32::load(out_weights.as_ptr().add(bucket_offset + offset));
            let ntm_weight = i16x32::load(out_weights.as_ptr().add(bucket_offset + HL + offset));

            sum += (stm * stm_weight).madd(stm);
            sum += (ntm * ntm_weight).madd(ntm);
        }
    }

    *output = sum.reduce_sum();
}
