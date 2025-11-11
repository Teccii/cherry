use std::sync::atomic::*;
use crate::*;

/*----------------------------------------------------------------*/

pub const MAX_TT_SIZE: u64 = 64 * 1024 * 1024; //64 TiB

#[inline]
fn score_from_tt(score: Score, ply: u16) -> Score {
    if score.is_loss() {
        score + ply as i16
    } else if score.is_win() {
        score - ply as i16
    } else {
        score
    }
}

#[inline]
fn score_to_tt(score: Score, ply: u16) -> Score {
    if score.is_loss() {
        score - ply as i16
    } else if score.is_win() {
        score + ply as i16
    } else {
        score
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TTFlag {
    None,
    UpperBound,
    LowerBound,
    Exact,
}

#[derive(Debug, Copy, Clone)]
pub struct TTData {
    pub depth: u8,
    pub eval: Score,
    pub score: Score,
    pub mv: Option<Move>,
    pub flag: TTFlag,
    pub age: u8,
    pub pv: bool,
}

impl TTData {
    #[inline]
    pub fn new(
        depth: u8,
        eval: Score,
        score: Score,
        mv: Option<Move>,
        flag: TTFlag,
        pv: bool,
        age: u8,
    ) -> TTData {
        TTData { depth, eval, score, mv, flag, pv, age }
    }

    #[inline]
    pub fn pack(self) -> TTPackedData {
        TTPackedData {
            depth: self.depth,
            eval: self.eval,
            score: self.score,
            mv: self.mv,
            other: self.flag as u8 | ((self.pv as u8) << 2) | (self.age << 3),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TTPackedData {
    depth: u8,
    eval: Score,
    score: Score,
    mv: Option<Move>,
    other: u8
}

impl TTPackedData {
    #[inline]
    pub fn unpack(self) -> TTData {
        TTData::new(
            self.depth,
            self.eval,
            self.score,
            self.mv,
            unsafe { core::mem::transmute(self.other & 0x3)},
            (self.other & 0x4) != 0,
            self.other >> 3,
        )
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug)]
pub struct TTEntry {
    pub key: AtomicU64,
    pub data: AtomicU64,
}

impl TTEntry {
    #[inline]
    pub fn empty() -> TTEntry {
        TTEntry {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn store(&self, hash: u64, data: TTData) {
        let data: u64 = unsafe { core::mem::transmute(data.pack()) };

        self.key.store(hash ^ data, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }

    #[inline]
    pub fn reset(&self) {
        self.key.store(0, Ordering::Relaxed);
        self.data.store(0, Ordering::Relaxed);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn hash(&self) -> u64 {
        self.key.load(Ordering::Relaxed) ^ self.data.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn data(&self) -> TTData {
        let data: TTPackedData = unsafe { core::mem::transmute(self.data.load(Ordering::Relaxed)) };

        data.unpack()
    }
}

/*----------------------------------------------------------------*/

pub struct TTable {
    entries: Box<[TTEntry]>,
    age: AtomicU8,
}

impl TTable {
    #[inline]
    pub fn new(mb: u64) -> TTable {
        let size = (mb * 1024 * 1024 / size_of::<TTEntry>() as u64) as usize;

        TTable {
            entries: (0..size).map(|_| TTEntry::empty()).collect(),
            age: AtomicU8::new(0),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn fetch(&self, board: &Board, ply: u16) -> Option<TTData> {
        let hash = board.hash();
        let index = self.index(hash);

        let entry = &self.entries[index];

        if entry.hash() == hash {
            let mut data = entry.data();
            
            return if data.flag != TTFlag::None {
                data.score = score_from_tt(data.score, ply);
                Some(data)
            } else {
                None
            };
        }

        None
    }

    #[inline]
    pub fn store(
        &self,
        board: &Board,
        depth: u8,
        ply: u16,
        eval: Score,
        score: Score,
        mv: Option<Move>,
        flag: TTFlag,
        pv: bool
    ) {
        let old_data = self.fetch(board, ply);
        let new_data = TTData::new(
            depth,
            eval,
            score_to_tt(score, ply),
            mv.or_else(|| old_data.and_then(|d| d.mv)),
            flag,
            pv,
            self.age.load(Ordering::Relaxed)
        );

        let hash = board.hash();
        let index = self.index(hash);

        self.entries[index].store(hash, new_data);
    }

    #[inline]
    pub fn clear(&self) {
        self.entries.iter().for_each(|e| e.reset());
        self.age.store(0, Ordering::Relaxed);
    }

    #[inline]
    pub fn age(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
        self.age.fetch_and(0b11111, Ordering::Relaxed);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn index(&self, hash: u64) -> usize {
        ((u128::from(hash) * self.entries.len() as u128) >> 64) as usize
    }
}