use crate::*;

#[derive(Debug, Copy, Clone)]
pub struct Window {
    start: i16,
    window: i16,
    center: Score,
    alpha: Score,
    beta: Score,
}

impl Window {
    #[inline]
    pub fn new(window: i16) -> Window {
        Window {
            window,
            start: window,
            center: Score::ZERO,
            alpha: Score(-window),
            beta: Score(window),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn set_center(&mut self, score: Score) {
        self.center = score;
    }

    #[inline]
    pub fn reset(&mut self) {
        self.window = self.start;
        self.alpha = self.center - self.window;
        self.beta = self.center + self.window;
    }

    #[inline]
    pub fn expand(&mut self) {
        self.window += self.window * W::asp_window_expand() / 64;
    }

    #[inline]
    pub fn fail_high(&mut self) {
        self.beta = self.center + self.window;
    }

    #[inline]
    pub fn fail_low(&mut self) {
        self.beta = (self.alpha + self.beta) / 2;
        self.alpha = self.center - self.window;
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get(&self) -> (Score, Score) {
        (self.alpha, self.beta)
    }
}