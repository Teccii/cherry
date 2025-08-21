use std::sync::atomic::*;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TTBound {
    #[allow(dead_code)] None,
    UpperBound,
    LowerBound,
    Exact
}

impl TTBound {
    #[inline]
    pub const fn index(i: usize) -> TTBound {
        if i < 4 {
            return unsafe {
                ::core::mem::transmute::<u8, TTBound>(i as u8)
            };
        }

        panic!("TTBound::index(): Index out of bounds");
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct TTData {
    pub depth: u8,
    pub score: Score,
    pub eval: Option<Score>,
    pub table_mv: Option<Move>,
    pub bound: TTBound,
    pub age: u8,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct TTPackedData {
    pub depth: u8,
    pub score: Score,
    pub eval: Score,
    pub table_mv: Option<Move>,
    pub other: u8,
}

impl TTData {
    #[inline]
    pub fn new(
        depth: u8,
        score: Score,
        eval: Option<Score>,
        table_mv: Option<Move>,
        bound: TTBound,
        age: u8,
    ) -> TTData {
        TTData {
            depth,
            score,
            eval,
            table_mv,
            bound,
            age
        }
    }
    
    #[inline]
    pub fn from_bits(bits: u64) -> TTData {
        let packed = unsafe {
            ::core::mem::transmute::<u64, TTPackedData>(bits)
        };
        
        TTData {
            depth: packed.depth,
            score: packed.score,
            eval: Some(packed.eval).filter(|s| !s.is_infinite()),
            table_mv: packed.table_mv,
            bound: TTBound::index(((packed.other >> 6) & 0b11) as usize),
            age: packed.other & 0b111111,
        }
    }
    
    #[inline]
    pub fn to_bits(self) -> u64 {
        unsafe {
            ::core::mem::transmute::<TTPackedData, u64>(TTPackedData {
                depth: self.depth,
                score: self.score,
                eval: self.eval.unwrap_or(Score::INFINITE),
                table_mv: self.table_mv,
                other: (self.age & 0b111111) | ((self.bound as u8) << 6)
            })
        }
    }
}

/*----------------------------------------------------------------*/

pub const TT_ENTRY_SIZE: usize = size_of::<AtomicU16>() + size_of::<AtomicU64>();

#[derive(Debug)]
pub struct TTable {
    hash: Box<[AtomicU16]>,
    data: Box<[AtomicU64]>,
    age: AtomicU8,
}

impl TTable {
    #[inline]
    pub fn new(mb: usize) -> TTable {
        let size = mb * 1024 * 1024 / TT_ENTRY_SIZE;
        
        TTable {
            hash: (0..size).map(|_| AtomicU16::new(0)).collect(),
            data: (0..size).map(|_| AtomicU64::new(0)).collect(),
            age: AtomicU8::new(0),
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        (self.hash.len() * TT_ENTRY_SIZE).div_ceil(1024 * 1024)
    }

    /*----------------------------------------------------------------*/

    pub fn prefetch(&self, board: &Board) {
        #[cfg(target_feature = "sse")] {
            use ::core::arch::x86_64::*;

            let hash = board.hash();
            let index = self.index(hash);

            unsafe {
                _mm_prefetch::<_MM_HINT_T0>(self.hash.as_ptr().add(index) as *const i8);
                _mm_prefetch::<_MM_HINT_T0>(self.data.as_ptr().add(index) as *const i8);
            }
        }
    }

    pub fn probe(&self, board: &Board) -> Option<TTData> {
        let hash = board.hash();
        let partial = hash as u16;
        let index = self.index(hash);

        if self.hash[index].load(Ordering::Relaxed) == partial {
            return Some(TTData::from_bits(self.data[index].load(Ordering::Relaxed))).filter(|d| d.bound != TTBound::None);
        }
        
        None
    }
    
    pub fn store(
        &self,
        board: &Board,
        depth: u8,
        score: Score,
        eval: Option<Score>,
        table_mv: Option<Move>,
        flag: TTBound,
    ) {
        let new_data = TTData::new(
            depth,
            score,
            eval,
            table_mv,
            flag,
            self.age.load(Ordering::Relaxed)
        );
        
        let hash = board.hash();
        let partial = Self::partial(hash);
        let index = self.index(hash);
        self.hash[index].store(partial, Ordering::Relaxed);
        self.data[index].store(new_data.to_bits(), Ordering::Relaxed);
    }

    pub fn hash_usage(&self) -> u16 {
        let mut result = 0;
        let age = self.age.load(Ordering::Relaxed);

        for i in 0..1000 {
            if self.hash[i].load(Ordering::Relaxed) != 0 && TTData::from_bits(self.data[i].load(Ordering::Relaxed)).age == age {
                result += 1;
            }
        }
        result
    }

    #[inline]
    pub fn age(&self) {
        let new_age = (self.age.load(Ordering::Relaxed) + 1) & 0b111111;
        self.age.store(new_age, Ordering::Relaxed);
    }

    #[inline]
    pub fn clean(&self) {
        self.hash.iter().for_each(|e| e.store(0, Ordering::Relaxed));
        self.data.iter().for_each(|e| e.store(0, Ordering::Relaxed));
        self.age.store(0, Ordering::Relaxed);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn partial(hash: u64) -> u16 {
        hash as u16
    }

    #[inline]
    fn index(&self, hash: u64) -> usize {
        ((u128::from(hash) * self.hash.len() as u128) >> 64) as usize
    }
}