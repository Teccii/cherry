use crate::*;

#[derive(Debug, Copy, Clone)]
pub struct Window {
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
        self.alpha = self
            .center
            .0
            .checked_sub(self.window)
            .map_or(-Score::INFINITE, Score::new);
        self.beta = self
            .center
            .0
            .checked_add(self.window)
            .map_or(Score::INFINITE, Score::new);
    }

    #[inline]
    pub fn expand(&mut self) {
        self.window = (self.window as i32)
            .checked_add(self.window as i32 * W::asp_window_expand() / 64)
            .unwrap_or(Score::INFINITE.0 as i32) as i16;
    }

    #[inline]
    pub fn fail_high(&mut self) {
        self.beta = self
            .center
            .0
            .checked_add(self.window)
            .map_or(Score::INFINITE, Score::new);
    }

    #[inline]
    pub fn fail_low(&mut self) {
        self.beta = (self.alpha + self.beta) / 2;
        self.alpha = self
            .center
            .0
            .checked_sub(self.window)
            .map_or(-Score::INFINITE, Score::new);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn get(&self) -> (Score, Score) {
        (self.alpha, self.beta)
    }
}
