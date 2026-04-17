use crate::cherry::Score;

#[inline]
pub fn wdl_params(material: i16) -> (f64, f64) {
    let a = [-90.46060983, 180.58434842, -183.30051524, 352.00571429];
    let b = [-13.25291718, 51.82458721, -11.76784060, 44.37897932];

    let m = material.clamp(17, 78) as f64 / 58.0;

    (
        ((a[0] * m + a[1]) * m + a[2]) * m + a[3],
        ((b[0] * m + b[1]) * m + b[2]) * m + b[3],
    )
}

#[inline]
pub fn wdl_model(score: Score, material: i16) -> (i16, i16) {
    let (a, b) = wdl_params(material);
    let x = score.0 as f64;

    (
        f64::round(1000.0 / (1.0 + f64::exp((a - x) / b))) as i16,
        f64::round(1000.0 / (1.0 + f64::exp((a + x) / b))) as i16,
    )
}

impl Score {
    #[inline]
    pub fn normalise(self, material: i16) -> Score {
        if self == Score::ZERO || self.is_decisive() {
            return self;
        }

        let normalised = (self.0 as f64) / wdl_params(material).0;
        Score(f64::round(normalised * 100.0) as i32).clamp_nomate()
    }
}
