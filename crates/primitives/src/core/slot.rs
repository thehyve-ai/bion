use std::{
    fmt::{Debug, Display},
    ops::{Add, Sub},
};

use num_traits::CheckedDiv;
use serde::{Deserialize, Serialize};

use super::Epoch;

#[derive(
    arbitrary::Arbitrary,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct Slot(u64);

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot({})", self.0)
    }
}

impl From<u64> for Slot {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Slot> for u64 {
    fn from(value: Slot) -> u64 {
        value.0
    }
}

impl Add<u64> for Slot {
    type Output = Self;

    fn add(self, rhs: u64) -> Self {
        Self(self.0 + rhs)
    }
}

impl Add<Slot> for Slot {
    type Output = Self;

    fn add(self, rhs: Slot) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub<u64> for Slot {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self {
        Self(self.0 - rhs)
    }
}

impl Slot {
    pub const fn new(slot: u64) -> Slot {
        Slot(slot)
    }

    pub fn epoch(self, slots_per_epoch: u64) -> Epoch {
        Epoch::new(self.0)
            .checked_div(&slots_per_epoch.into())
            .expect("slots_per_epoch is not 0")
    }

    pub fn max_value() -> Slot {
        Slot(u64::MAX)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
