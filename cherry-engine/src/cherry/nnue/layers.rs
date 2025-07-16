use super::*;

#[inline(always)]
pub fn activate_ft(
    us: &Align64<[i16; HL]>,
    them: &Align64<[i16; HL]>,
    output: &mut Align64<[u8; L1]>,
) {
    screlu(us, &mut output[..HL]);
    screlu(them, &mut output[HL..]);
}

pub fn propagate<const L: usize, const NL: usize>(
    input: &Align64<[u8; L]>,
    weights: &Align64<[i16; L * NL]>,
    biases: &Align64<[i16; NL]>,
    output: &mut Align64<[u8; NL]>,
) {
    let mut values = biases.clone();

    for i in 0..L {
        let value = input[i];

        for j in 0..NL {
            values[j] += value as i16 * weights[i + j * L];
        }
    }

    screlu(&values, &mut output[..]);
}

pub fn propagate_out<const L: usize>(
    input: &Align64<[u8; L]>,
    weights: &Align64<[i16; L]>,
    bias: i16,
    output: &mut i16,
) {
    *output = bias + input.iter()
        .zip(weights.iter())
        .map(|(&i, &weight)| i as i16 * weight)
        .sum::<i16>();
}

pub fn screlu<const N: usize>(input: &Align64<[i16; N]>, output: &mut [u8]) {
    input.iter().zip(output.iter_mut()).for_each(|(&x, out)| {
        let x = x.max(0).min(QA) as u16;
        *out = ((x * x) >> 8) as u8;
    });
}