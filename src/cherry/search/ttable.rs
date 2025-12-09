use std::sync::atomic::*;

use rayon::prelude::*;

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
    pub key: u16,
    pub depth: u16,
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
        key: u16,
        depth: u16,
        eval: Score,
        score: Score,
        mv: Option<Move>,
        flag: TTFlag,
        pv: bool,
        age: u8,
    ) -> TTData {
        TTData {
            key,
            depth,
            eval,
            score,
            mv,
            flag,
            pv,
            age,
        }
    }

    #[inline]
    pub fn pack(self) -> TTPackedData {
        TTPackedData {
            eval: self.eval,
            score: self.score,
            mv: self.mv,
            other: self.flag as u8 | ((self.pv as u8) << 2),
            age: self.age,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TTPackedData {
    eval: Score,
    score: Score,
    mv: Option<Move>,
    other: u8,
    age: u8,
}

impl TTPackedData {
    #[inline]
    pub fn unpack(self, key: u16, depth: u16) -> TTData {
        TTData::new(
            key,
            depth,
            self.eval,
            self.score,
            self.mv,
            unsafe { core::mem::transmute(self.other & 0x3) },
            (self.other & 0x4) != 0,
            self.age,
        )
    }
}

/*----------------------------------------------------------------*/

const CLUSTER_SIZE: usize = 5;

pub struct TTCluster {
    data: [AtomicU64; CLUSTER_SIZE],
    depth: [AtomicU16; CLUSTER_SIZE],
    key: [AtomicU16; CLUSTER_SIZE],
}

impl TTCluster {
    #[inline]
    pub fn empty() -> TTCluster {
        TTCluster {
            data: core::array::from_fn(|_| AtomicU64::new(0)),
            depth: core::array::from_fn(|_| AtomicU16::new(0)),
            key: core::array::from_fn(|_| AtomicU16::new(0)),
        }
    }

    #[inline]
    pub fn load(&self, index: usize) -> TTData {
        let packed_data: TTPackedData =
            unsafe { core::mem::transmute(self.data[index].load(Ordering::Relaxed)) };

        packed_data.unpack(
            self.key[index].load(Ordering::Relaxed),
            self.depth[index].load(Ordering::Relaxed)
        )
    }

    #[inline]
    pub fn store(&self, index: usize, data: TTData) {
        self.data[index].store(
            unsafe { core::mem::transmute(data.pack()) },
            Ordering::Relaxed,
        );
        self.depth[index].store(data.depth, Ordering::Relaxed);
        self.key[index].store(data.key, Ordering::Relaxed);
    }

    #[inline]
    pub fn clear(&self) {
        for i in 0..CLUSTER_SIZE {
            self.data[i].store(0, Ordering::Relaxed);
            self.depth[i].store(0, Ordering::Relaxed);
            self.key[i].store(0, Ordering::Relaxed);
        }
    }
}

/*----------------------------------------------------------------*/

pub struct TTable {
    clusters: Box<[TTCluster]>,
    age: AtomicU8,
}

impl TTable {
    #[inline]
    pub fn new(mb: u64) -> TTable {
        let size = (mb * 1024 * 1024 / size_of::<TTCluster>() as u64) as usize;

        TTable {
            clusters: (0..size).map(|_| TTCluster::empty()).collect(),
            age: AtomicU8::new(0),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn fetch(&self, board: &Board, ply: u16) -> Option<TTData> {
        let hash = board.hash();
        let partial_key = hash as u16;
        let cluster = &self.clusters[self.index(hash)];

        for i in 0..CLUSTER_SIZE {
            let mut entry = cluster.load(i);
            if entry.key == partial_key {
                return if entry.flag != TTFlag::None {
                    entry.score = score_from_tt(entry.score, ply);
                    Some(entry)
                } else {
                    None
                };
            }
        }

        None
    }

    #[inline]
    pub fn store(
        &self,
        board: &Board,
        depth: u16,
        ply: u16,
        eval: Score,
        score: Score,
        mv: Option<Move>,
        flag: TTFlag,
        pv: bool,
    ) {
        let hash = board.hash();
        let partial_key = hash as u16;
        let cluster = &self.clusters[self.index(hash)];
        let age = self.age.load(Ordering::Relaxed);

        let mut index = 0;
        let mut min_value = i32::MAX;

        for i in 0..CLUSTER_SIZE {
            let entry = cluster.load(i);

            if entry.key == partial_key || entry.flag == TTFlag::None {
                index = i;
                break;
            }

            let relative_age = entry.age.wrapping_sub(age);
            let entry_value = entry.depth as i32 - W::tt_relative_age() * relative_age as i32;

            if entry_value < min_value {
                index = i;
                min_value = entry_value;
            }
        }

        cluster.store(
            index,
            TTData::new(
                partial_key,
                depth,
                eval,
                score_to_tt(score, ply),
                mv.or_else(|| cluster.load(index).mv),
                flag,
                pv,
                age,
            ),
        );
    }

    #[inline]
    pub fn clear(&self) {
        self.clusters.par_iter().for_each(|c| c.clear());
        self.age.store(0, Ordering::Relaxed);
    }

    #[inline]
    pub fn age(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    fn index(&self, hash: u64) -> usize {
        ((u128::from(hash) * self.clusters.len() as u128) >> 64) as usize
    }
}
