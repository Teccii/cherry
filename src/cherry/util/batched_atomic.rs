use std::sync::{Arc, atomic::*};

pub struct BatchedAtomicCounter {
    global: Arc<AtomicU64>,
    local: u64,
    buffer: u64,
}

impl BatchedAtomicCounter {
    #[inline]
    pub fn new(global: Arc<AtomicU64>) -> BatchedAtomicCounter {
        BatchedAtomicCounter {
            global,
            local: 0,
            buffer: 0,
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn inc(&mut self) {
        self.buffer += 1;

        if self.buffer >= Self::BATCH_SIZE {
            self.flush();
        }
    }

    #[inline]
    pub fn flush(&mut self) {
        self.global.fetch_add(self.buffer, Ordering::Relaxed);
        self.local += self.buffer;
        self.buffer = 0;
    }

    #[inline]
    pub fn reset(&mut self) {
        self.global.store(0, Ordering::Relaxed);
        self.local = 0;
        self.buffer = 0;
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn global(&self) -> u64 {
        self.global.load(Ordering::Relaxed) + self.buffer
    }

    #[inline]
    pub fn local(&self) -> u64 {
        self.local + self.buffer
    }

    /*----------------------------------------------------------------*/

    pub const BATCH_SIZE: u64 = 2048;
}

impl Clone for BatchedAtomicCounter {
    #[inline]
    fn clone(&self) -> Self {
        BatchedAtomicCounter {
            global: Arc::clone(&self.global),
            local: 0,
            buffer: 0,
        }
    }
}
