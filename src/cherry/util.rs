use std::sync::{Arc, atomic::*};
use colored::Colorize;
/*----------------------------------------------------------------*/

pub const MAX_DEPTH: u8 = 128;
pub const MAX_PLY: u16 = 256;
pub const REDUCTION_SCALE: i32 = 1024;

/*----------------------------------------------------------------*/

#[derive(Debug)]
pub struct BatchedAtomicCounter {
    global: Arc<AtomicU64>,
    local: u64,
    buffer: u64,
}

impl BatchedAtomicCounter {
    #[inline]
    pub fn new() -> BatchedAtomicCounter {
        BatchedAtomicCounter {
            global: Arc::new(AtomicU64::new(0)), 
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

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct LookUp<T: Copy + Default, const M: usize, const N: usize> {
    table: [[T; N]; M],
}

impl<T: Copy + Default, const M: usize, const N: usize> LookUp<T, M, N> {
    pub fn new<F: Fn(usize, usize) -> T>(f: F) -> Self {
        let mut table = [[T::default(); N]; M];

        for i in 0..M {
            for j in 0..N {
                table[i][j] = f(i, j);
            }
        }

        Self { table }
    }

    #[inline]
    pub fn get(&self, i: usize, j: usize) -> T {
        if i >= M || j >= N {
            panic!("LookUp::get(): Indices out of bounds");
        }

        self.table[i][j]
    }
}

/*----------------------------------------------------------------*/

pub fn progress_bar(progress: usize, max: usize) -> String {
    format!("[{}{}]", "#".repeat(progress).bright_green(), ".".repeat(max - progress))
}

#[allow(dead_code)]
pub fn secs_to_hms(seconds: u32) -> (u32, u32, u32) {
    let minutes = seconds / 60;
    let hours = minutes / 60;

    (hours, minutes - hours * 60, seconds - minutes * 60)
}