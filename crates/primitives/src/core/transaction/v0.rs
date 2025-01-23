use alloy_rlp::{Decodable, Encodable};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tree_hash_derive::TreeHash;

use crate::kzg::KzgCommitment;

/// The transaction data without Blob for version 0 of the DA.
///
/// # Fields
/// - `dac_id`: The ID of the DAC that the transaction is for.
/// - `nonce`: The nonce of the transaction for the account.
/// - `ttl`: The time-to-live of the transaction in seconds.
/// - `blob_length`: The size of the blob in bytes.
/// - `kzg_commitment`: The KZG Commitment of the blob.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TreeHash, arbitrary::Arbitrary)]
pub struct TransactionV0 {
    /// The ID of the DAC that the transaction is for.
    #[serde_as(as = "DisplayFromStr")]
    pub dac_id: u64,

    /// The nonce of the transaction for the account.
    #[serde_as(as = "DisplayFromStr")]
    pub nonce: u64,

    /// The time-to-live of the transaction in seconds.
    #[serde_as(as = "DisplayFromStr")]
    pub ttl: u64,

    /// The size of the blob in bytes.
    #[serde_as(as = "DisplayFromStr")]
    pub blob_length: u64,

    /// The KZG Commitment of the blob.
    pub kzg_commitment: KzgCommitment,

    #[serde_as(as = "DisplayFromStr")]
    pub total_shards: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub shard_size: u64,
}

impl Encodable for TransactionV0 {
    fn length(&self) -> usize {
        self.dac_id.length()
            + self.nonce.length()
            + self.ttl.length()
            + self.blob_length.length()
            + self.kzg_commitment.length()
            + self.total_shards.length()
            + self.shard_size.length()
    }

    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.dac_id.encode(out);
        self.nonce.encode(out);
        self.ttl.encode(out);
        self.blob_length.encode(out);
        self.kzg_commitment.encode(out);
        self.total_shards.encode(out);
    }
}

impl Decodable for TransactionV0 {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            dac_id: Decodable::decode(buf)?,
            nonce: Decodable::decode(buf)?,
            ttl: Decodable::decode(buf)?,
            blob_length: Decodable::decode(buf)?,
            kzg_commitment: Decodable::decode(buf)?,
            total_shards: Decodable::decode(buf)?,
            shard_size: Decodable::decode(buf)?,
        })
    }
}
