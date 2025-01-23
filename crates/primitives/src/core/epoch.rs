use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Add, Div, Sub},
};

use super::slot::Slot;

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
    Debug,
    RlpEncodable,
    RlpDecodable,
)]
#[serde(transparent)]
pub struct Epoch(u64);

impl Epoch {
    pub const fn new(epoch: u64) -> Self {
        Self(epoch)
    }

    /// The first slot in the epoch.
    pub fn start_slot(self, slots_per_epoch: u64) -> Slot {
        Slot::from(self.0.saturating_mul(slots_per_epoch))
    }

    /// The last slot in the epoch.
    pub fn end_slot(self, slots_per_epoch: u64) -> Slot {
        Slot::from(
            self.0
                .saturating_mul(slots_per_epoch)
                .saturating_add(slots_per_epoch.saturating_sub(1)),
        )
    }

    /// Position of some slot inside an epoch, if any.
    ///
    /// E.g., the first `slot` in `epoch` is at position `0`.
    pub fn position(self, slot: Slot, slots_per_epoch: u64) -> Option<usize> {
        let start = self.start_slot(slots_per_epoch);
        let end = self.end_slot(slots_per_epoch);

        if slot >= start && slot <= end {
            slot.as_usize().checked_sub(start.as_usize())
        } else {
            None
        }
    }
}

impl Display for Epoch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Epoch({})", self.0)
    }
}

impl From<u64> for Epoch {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Epoch> for u64 {
    fn from(value: Epoch) -> u64 {
        value.0
    }
}

impl Add<u64> for Epoch {
    type Output = Self;

    fn add(self, rhs: u64) -> Self {
        Self(self.0 + rhs)
    }
}

impl Sub<u64> for Epoch {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self {
        Self(self.0 - rhs)
    }
}

impl Div<u64> for Epoch {
    type Output = Self;

    fn div(self, rhs: u64) -> Self {
        Self(self.0 / rhs)
    }
}

impl Div<Epoch> for Epoch {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

impl num_traits::ops::checked::CheckedDiv for Epoch {
    fn checked_div(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_div(rhs.0).map(Self)
    }
}
