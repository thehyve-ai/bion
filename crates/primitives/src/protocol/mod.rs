use lighthouse_bls::PublicKeyBytes;
use serde::{Deserialize, Serialize};

use crate::core::TransactionId;

/// Data about an attester.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttesterData {
    /// The BLS public key of the attester.
    pub pubkey: PublicKeyBytes,

    /// The index of the operator within all operators ever.
    pub operator_index: u64,

    /// The size of the commitee the attester is in.
    pub committee_length: u64,

    /// The index of the attester within the committee.
    pub operator_committee_index: u64,
}

/// Data about an attester and the indices of a blob they are attesting to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttesterDataAndIndices {
    pub transaction_id: TransactionId,
    pub data: AttesterData,
    pub indices: Vec<u64>,
}
