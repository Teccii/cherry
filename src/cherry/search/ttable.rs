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

#[derive(Debug)]
pub struct TTEntry {
    key: AtomicU64,
    data: AtomicU64
}

impl TTEntry {
    #[inline]
    pub fn data(&self) -> TTData {
        TTData::from_bits(self.data.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn set(&self, hash: u64, data: TTData) {
        let data = data.to_bits();
        
        self.key.store(hash, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }
    
    #[inline]
    pub fn reset(&self) {
        self.key.store(0, Ordering::Relaxed);
        self.data.store(0, Ordering::Relaxed);
    }
    
    #[inline]
    pub fn zero() -> TTEntry {
        TTEntry {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0)
        }
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug)]
pub struct TTable {
    entries: Box<[TTEntry]>,
    age: AtomicU8,
}

impl TTable {
    #[inline]
    pub fn new(mb: usize) -> TTable {
        let size = mb * 1024 * 1024 / size_of::<TTEntry>();
        
        TTable {
            entries: (0..size).map(|_| TTEntry::zero()).collect(),
            age: AtomicU8::new(0),
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.entries.len() * size_of::<TTEntry>() / (1024 * 1024)
    }

    /*----------------------------------------------------------------*/

    pub fn prefetch(&self, board: &Board) {
        #[cfg(target_feature = "sse")] {
            use ::core::arch::x86_64::*;

            let hash = board.hash();
            let index = self.index(hash);

            unsafe {
                _mm_prefetch::<_MM_HINT_T0>(self.entries.as_ptr().add(index) as *const i8)
            }
        }
    }

    pub fn probe(&self, board: &Board) -> Option<TTData> {
        let hash = board.hash();
        let index = self.index(hash);
        
        let entry = &self.entries[index];
        let data = entry.data.load(Ordering::Relaxed);
        
        if entry.key.load(Ordering::Relaxed) == hash {
            return Some(TTData::from_bits(data));
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
        let index = self.index(hash);

        if self.entries[index].data().bound != TTBound::Exact {
            self.entries[index].set(hash, new_data);
        }
    }

    pub fn hash_usage(&self) -> u16 {
        let mut result = 0;
        let age = self.age.load(Ordering::Relaxed);

        for i in 0..1000 {
            if self.entries[i].key.load(Ordering::Relaxed) != 0 && self.entries[i].data().age == age {
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
        self.entries.iter().for_each(|e| e.reset());
        self.age.store(0, Ordering::Relaxed);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn index(&self, hash: u64) -> usize {
        ((u128::from(hash) * self.entries.len() as u128) >> 64) as usize
    }
}