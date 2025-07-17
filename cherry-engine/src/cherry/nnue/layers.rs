use std::simd::prelude::*;
use super::simd::*;
use super::*;

#[inline]
pub fn activate_ft(
    us: &[i16; HL],
    them: &[i16; HL],
    output: &mut [u8; L1],
) {
    screlu(us, &mut output[..HL].try_into().unwrap());
    screlu(them, &mut output[HL..].try_into().unwrap());
}

pub fn propagate<const L: usize, const NL: usize>(
    input: &[u8; L],
    weights: &[i16; L * NL],
    biases: &[i16; NL],
    output: &mut [u8; NL],
) {
    let mut values = [0; NL];

    for j in 0..NL {
        let offset = j * L;
        
        propagate_out(
            input,
            weights[offset..(offset + L)].try_into().unwrap(),
            biases[j],
            &mut values[j]
        );
    }

    screlu(&values, output);
}

pub fn propagate_out<const L: usize>(
    input: &[u8; L],
    weights: &[i16; L],
    bias: i16,
    output: &mut i16,
) {
    let mut sum = I16Reg::splat(0);

    for i in 0..(L/CHUNK_SIZE) {
        let offset = i * CHUNK_SIZE;
        let input: I16Reg = U8Reg::from_slice(&input[offset..]).cast();
        let weight = I16Reg::from_slice(&weights[offset..]);

        sum += input * weight;
    }

    *output = bias + sum.reduce_sum();
    for i in (L - L % CHUNK_SIZE)..L {
        *output += input[i] as i16 * weights[i];
    }
}

pub fn screlu<const N: usize>(input: &[i16; N], output: &mut [u8; N]) {
    let zero = I16Reg::splat(0);
    let qa = I16Reg::splat(QA);
    
    for i in 0..(N/CHUNK_SIZE) {
        let offset = i * CHUNK_SIZE;
        let value = I16Reg::from_slice(&input[offset..]);
        let value = value.simd_clamp(zero, qa);
        let value: U8Reg = ((value * value) >> 8).cast();
        
        value.copy_to_slice(&mut output[offset..]);
    }

    for i in (N - N % CHUNK_SIZE)..N {
        let value = input[i].clamp(0, QA);
        output[i] = ((value * value) >> 8) as u8;
    }
}