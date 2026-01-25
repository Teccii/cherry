use crate::*;

#[derive(Debug, Copy, Clone)]
pub struct Window {
    window: i32,
    initial_window: i32,
    center: Score,
    alpha: Score,
    beta: Score,
}

impl Window {
    #[inline]
    pub fn new(window: i32) -> Window {
        Window {
            window,
            initial_window: window,
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
        self.window = self.initial_window;
        self.alpha = self.center.saturating_sub(Score(self.window));
        self.beta = self.center.saturating_add(Score(self.window));
    }

    #[inline]
    pub fn expand(&mut self) {
        self.window = self
            .window
            .saturating_add(self.window * W::asp_window_expand() / 64)
            .min(Score::INFINITE.0);
    }

    #[inline]
    pub fn fail_high(&mut self) {
        self.beta = self.center.saturating_add(Score(self.window));
    }

    #[inline]
    pub fn fail_low(&mut self) {
        self.beta = (self.alpha + self.beta) / 2;
        self.alpha = self.center.saturating_sub(Score(self.window));
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get(&self) -> (Score, Score) {
        (self.alpha, self.beta)
    }
}
