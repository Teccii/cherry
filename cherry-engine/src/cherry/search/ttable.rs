use std::sync::atomic::*;
use cherry_core::*;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TTFlag {
    None,
    UpperBound,
    LowerBound,
    Exact
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct TTData {
    pub depth: u8,
    pub score: Score,
    pub eval: Option<Score>,
    pub table_mv: Option<Move>,
    pub flag: TTFlag,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct TTPackedData {
    pub depth: u8,
    pub score: Score,
    pub eval: Score,
    pub table_mv: Option<Move>,
    pub flag: TTFlag,
}

impl TTData {
    #[inline(always)]
    pub fn new(
        depth: u8,
        score: Score,
        eval: Option<Score>,
        table_mv: Option<Move>,
        flag: TTFlag,
    ) -> TTData {
        TTData {
            depth,
            score,
            eval,
            table_mv,
            flag
        }
    }
    
    #[inline(always)]
    pub fn from_bits(bits: u64) -> TTData {
        let packed = unsafe {
            std::mem::transmute::<u64, TTPackedData>(bits)
        };
        
        TTData {
            depth: packed.depth,
            score: packed.score,
            eval: Some(packed.eval).filter(|s| !s.is_infinite()),
            table_mv: packed.table_mv,
            flag: packed.flag,
        }
    }
    
    #[inline(always)]
    pub fn to_bits(self) -> u64 {
        unsafe {
            std::mem::transmute::<TTPackedData, u64>(TTPackedData {
                depth: self.depth,
                score: self.score,
                eval: self.eval.unwrap_or(Score::INFINITE),
                table_mv: self.table_mv,
                flag: self.flag
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
    #[inline(always)]
    pub fn set(&self, hash: u64, data: TTData) {
        let data = data.to_bits();
        
        self.key.store(hash ^ data, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }
    
    #[inline(always)]
    pub fn reset(&self) {
        self.key.store(0, Ordering::Relaxed);
        self.data.store(0, Ordering::Relaxed);
    }
    
    #[inline(always)]
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
    size: u64,
}

impl TTable {
    #[inline(always)]
    pub fn new(mb: usize) -> TTable {
        let size = mb * 1024 * 1024 / size_of::<TTEntry>();
        
        TTable {
            entries: (0..size).map(|_| TTEntry::zero()).collect(),
            size: size as u64
        }
    }

    /*----------------------------------------------------------------*/

    pub fn probe(&self, board: &Board) -> Option<TTData> {
        let hash = board.hash();
        let index = self.index(hash);
        
        let entry = &self.entries[index];
        let data = entry.data.load(Ordering::Relaxed);
        
        if entry.key.load(Ordering::Relaxed) ^ data == hash {
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
        flag: TTFlag,
    ) {
        let new_data = TTData::new(
            depth,
            score,
            eval,
            table_mv,
            flag,
        );
        
        let hash = board.hash();
        let index = self.index(hash);
        
        let old_entry = &self.entries[index];
        let old_data = old_entry.data.load(Ordering::Relaxed);
        
        if old_entry.key.load(Ordering::Relaxed) ^ old_data == hash {
            if TTable::replace(new_data, TTData::from_bits(old_data)) {
                old_entry.set(hash, new_data);
            }
        } else {
            old_entry.set(hash, new_data);
        }
    }
    
    pub fn clean(&self) {
        self.entries.iter().for_each(|e| e.reset());
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    fn replace(new_data: TTData, old_data: TTData) -> bool {
        let new_priority = new_data.depth + new_data.flag as u8;
        let old_priority = old_data.depth + old_data.flag as u8;
        
        old_data.flag == TTFlag::None || new_priority >= old_priority
    }

    #[inline(always)]
    fn index(&self, hash: u64) -> usize {
        (hash % self.size) as usize
    }
}