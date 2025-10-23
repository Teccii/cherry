use std::sync::{Arc, atomic::*};
use colored::Colorize;

/*----------------------------------------------------------------*/

pub const MAX_DEPTH: u16 = 256;
pub const MAX_PLY: u16 = 256;
pub const REDUCTION_SCALE: i32 = 1024;

/*----------------------------------------------------------------*/

#[derive(Default)]
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

pub fn new_zeroed<T>() -> Box<T> {
    unsafe { Box::new_zeroed().assume_init() }
}

/*----------------------------------------------------------------*/

#[allow(dead_code)]
pub fn progress_bar(progress: usize, max: usize) -> String {
    format!("[{}{}]", "#".repeat(progress).bright_green(), ".".repeat(max - progress))
}

pub fn fmt_big_num(num: u64) -> String {
    match num {
        0..1000 => format!("{}", num),
        1000..1_000_000 => format!("{:.2}K", num as f32 / 1000.0),
        1_000_000..1_000_000_000 => format!("{:.2}M", num as f32 / 1_000_000.0),
        _ => format!("{:.2}B", num as f32 / 1_000_000_000.0),
    }
}

pub fn fmt_time(millis: u64) -> String {
    let (h, m, s) = secs_to_hms((millis / 1000) as u32);

    format!("{}h {}m {}s {}ms", h, m, s, millis % 1000)
}

#[allow(dead_code)]
pub fn secs_to_hms(seconds: u32) -> (u32, u32, u32) {
    let minutes = seconds / 60;
    let hours = minutes / 60;

    (hours, minutes - hours * 60, seconds - minutes * 60)
}