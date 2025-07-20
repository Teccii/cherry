use crate::Score;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct Window {
    start: i16,
    window: i16,
    midpoint: Score,
    alpha: Score,
    beta: Score,
}

impl Window {
    #[inline]
    pub fn new(window: i16) -> Window {
        Window {
            window,
            start: window,
            midpoint: Score::ZERO,
            alpha: Score(-window),
            beta: Score(window),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn set_midpoint(&mut self, score: Score) {
        self.midpoint = score;
    }

    #[inline]
    pub fn reset(&mut self) {
        self.window = self.start;
        self.alpha = self.midpoint - self.window;
        self.beta = self.midpoint + self.window;
    }

    #[inline]
    pub fn expand(&mut self) {
        self.window += 7 + self.window / 2;
    }

    #[inline]
    pub fn fail_high(&mut self) {
        self.beta = self.midpoint + self.window;
        self.expand();
    }

    #[inline]
    pub fn fail_low(&mut self) {
        self.beta = (self.alpha + self.beta) / 2;
        self.alpha = self.midpoint - self.window;
        self.expand();
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get(&self) -> (Score, Score) {
        (self.alpha, self.beta)
    }
}