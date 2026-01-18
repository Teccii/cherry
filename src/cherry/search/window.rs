use crate::*;

#[derive(Debug, Copy, Clone)]
pub struct Window {
    window: i16,
    initial_window: i16,
    center: Score,
    alpha: Score,
    beta: Score,
}

impl Window {
    #[inline]
    pub fn new(window: i16) -> Window {
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
        self.window = (self.window as i32)
            .saturating_add(self.window as i32 * W::asp_window_expand() / 64)
            .min(Score::INFINITE.0 as i32) as i16;
    }

    #[inline]
    pub fn fail_high(&mut self) {
        self.beta = self.center.saturating_add(Score(self.window));
    }

    #[inline]
    pub fn fail_low(&mut self) {
        self.beta = Score(((self.alpha.0 as i32 + self.beta.0 as i32) / 2) as i16);
        self.alpha = self.center.saturating_sub(Score(self.window));
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get(&self) -> (Score, Score) {
        (self.alpha, self.beta)
    }
}
