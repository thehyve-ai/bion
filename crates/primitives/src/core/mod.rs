pub mod attestation;
pub mod attestation_data;
pub mod epoch;
pub mod signature;
pub mod slot;
pub mod transaction;

pub use self::epoch::Epoch;
pub use self::transaction::{TransactionId, TransactionV0};

mod util;

/// The [`SecurityParameters`] struct determines the coding rate of the
/// erasure encoding algorithm. The adversary threshold is the percentage of nodes
/// that can be adversarial, and the quorum threshold is the percentage of nodes
/// that must be honest. The coding rate is the result of `quorum_threshold - adversary_threshold`.
///
/// ### Note: the percentages are represented as integers, not decimals (i.e. 80% is 80, not 0.8).
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct SecurityParameters {
    adversary_threshold: u32,
    quorum_threshold: u32,
}

impl SecurityParameters {
    pub fn new(adversary_threshold: u32, quorum_threshold: u32) -> Result<Self, String> {
        if adversary_threshold >= quorum_threshold {
            return Err("Adversary threshold must be less than quorum threshold".to_string());
        }

        Ok(Self {
            adversary_threshold,
            quorum_threshold,
        })
    }
    pub fn adversary_threshold(&self) -> u32 {
        self.adversary_threshold
    }

    pub fn quorum_threshold(&self) -> u32 {
        self.quorum_threshold
    }
}

impl Default for SecurityParameters {
    fn default() -> Self {
        Self {
            adversary_threshold: 80,
            quorum_threshold: 90,
        }
    }
}
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
