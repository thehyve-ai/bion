use alloy_primitives::B256;

/// A transaction hash is the keccak256 of an BZZ encoded signed transaction.
pub type TransactionHash = B256;

/// The identifier of the DA committee.
pub type DacId = u64;

/// The time-to-live of a blob in the DA protocol after inclusion in the chain.
pub type BlobTTL = u64;
