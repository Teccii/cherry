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
    weights: &NetworkWeights,
    output: &mut i32
) {
    let (zero, qa) = (zero(), splat_i16(QA as i16));
    let mut sum = zero;

    for i in 0..(HL / I16_CHUNK) {
        let offset = i * I16_CHUNK;

        unsafe {
            let stm = clamp_i16(load_i16(stm.as_ptr().add(offset)), zero, qa);
            let ntm = clamp_i16(load_i16(ntm.as_ptr().add(offset)), zero, qa);
            let stm_weight = load_i16(weights.out_weights.as_ptr().add(offset));
            let ntm_weight = load_i16(weights.out_weights.as_ptr().add(HL + offset));

            sum = add_i32(sum, madd_i16(mullo_i16(stm_weight, stm), stm));
            sum = add_i32(sum, madd_i16(mullo_i16(ntm_weight, ntm), ntm));
        }
    }

    *output = reduce_add_i32(sum);
}