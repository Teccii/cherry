use std::{cmp::Ordering, fmt, ops::*};
use super::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score(pub i16);

impl Score {
    #[inline(always)]
    pub fn new_mate(ply: u16) -> Score {
        Score::MAX_MATE - ply as i16
    }

    #[inline(always)]
    pub fn new_mated(ply: u16) -> Score {
        -Score::MAX_MATE + ply as i16
    }

    #[inline(always)]
    pub fn is_mate(self) -> bool {
        let abs_score = self.abs();

        abs_score >= Score::MIN_MATE && abs_score <= Score::MAX_MATE
    }

    #[inline(always)]
    pub fn mate_in(self) -> Option<i16> {
        if self.is_mate() {
            let abs_score = self.abs();
            let sign = self.0.signum();

            return Some(sign * (Score::MAX_MATE.0 - abs_score.0));
        }

        None
    }
    
    #[inline(always)]
    pub fn is_infinite(self) -> bool {
        let abs_score = self.abs();
        
        abs_score >= Score::INFINITE
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub const fn abs(self) -> Score {
        Score(self.0.abs())
    }
    
    #[inline(always)]
    pub const fn sign(self) -> i16 {
        self.0.signum()
    }

    /*----------------------------------------------------------------*/

    pub const MIN_MATE: Score = Score(i16::MAX - (2 * MAX_PLY) as i16);
    pub const MAX_MATE: Score = Score(i16::MAX - MAX_PLY as i16);
    pub const ZERO: Score = Score(0);
    pub const INFINITE: Score = Score(i16::MAX - (MAX_PLY as i16 / 2));
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_infinite() {
            if self.0 > 0 {
                write!(f, "+INF")
            } else {
                write!(f, "-INF")
            }
        } else if let Some(ply) = self.mate_in() {
            if ply > 0 {
                write!(f, "#{}", ply)
            } else {
                write!(f, "#-{}", ply)
            }
        } else if self.0 > 0 {
            write!(f, "+{:.1}", self.0 as f32 / 100.0)
        } else {
            write!(f, "{:.1}", self.0 as f32 / 100.0)
        }
    }
}

impl From<i16> for Score {
    #[inline(always)]
    fn from(value: i16) -> Self {
        Score(value)
    }
}

impl From<Score> for i16 {
    #[inline(always)]
    fn from(score: Score) -> i16 {
        score.0
    }
}

impl PartialEq<i16> for Score {
    #[inline(always)]
    fn eq(&self, other: &i16) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Score> for i16 {
    #[inline(always)]
    fn eq(&self, other: &Score) -> bool {
        *self == other.0
    }
}

impl PartialOrd<i16> for Score {
    #[inline(always)]
    fn partial_cmp(&self, other: &i16) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<Score> for i16 {
    #[inline(always)]
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Neg for Score {
    type Output = Score;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Score(-self.0)
    }
}

macro_rules! impl_score_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<Score> for Score {
            type Output = Score;

            #[inline(always)]
            fn $fn(self, rhs: Score) -> Self::Output {
                Score(self.0.$fn(rhs.0))
            }
        }
    )*};
}

macro_rules! impl_score_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<Score> for Score {
            #[inline(always)]
            fn $fn(&mut self, rhs: Score) {
                self.0.$fn(rhs.0);
            }
        }
    )*};
}

macro_rules! impl_score_i16_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<i16> for Score {
            type Output = Score;

            #[inline(always)]
            fn $fn(self, rhs: i16) -> Self::Output {
                Score(self.0.$fn(rhs))
            }
        }

        impl $trait<Score> for i16 {
            type Output = Score;

            #[inline(always)]
            fn $fn(self, rhs: Score) -> Self::Output {
                Score(self.$fn(rhs.0))
            }
        }
    )*};
}

macro_rules! impl_score_i16_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait<i16> for Score {
            #[inline(always)]
            fn $fn(&mut self, rhs: i16) {
                self.0.$fn(rhs);
            }
        }
    )*};
}

impl_score_ops! {
    Add, add;
    Sub, sub;
}

impl_score_assign_ops! {
    AddAssign, add_assign;
    SubAssign, sub_assign;
}

impl_score_i16_ops! {
    Add, add;
    Sub, sub;
    Mul, mul;
    Div, div;
}

impl_score_i16_assign_ops! {
    AddAssign, add_assign;
    SubAssign, sub_assign;
    MulAssign, mul_assign;
    DivAssign, div_assign;
}

/*----------------------------------------------------------------*/

#[test]
fn test_score() {
    assert_eq!(Score::INFINITE.is_mate(), false);
    assert_eq!((-Score::INFINITE).is_mate(), false);
    
    for i in 0..MAX_PLY {
        let mate_score = Score::new_mate(i);
        let mated_score = Score::new_mated(i);
        
        assert_eq!(mate_score.is_mate(), true);
        assert_eq!(mated_score.is_mate(), true);
        assert_eq!(mate_score.mate_in().unwrap(), i as i16);
        assert_eq!(mated_score.mate_in().unwrap(), -(i as i16));
        assert_eq!(Score::INFINITE > mate_score, true);
        assert_eq!(-Score::INFINITE < mated_score, true);
    }
}