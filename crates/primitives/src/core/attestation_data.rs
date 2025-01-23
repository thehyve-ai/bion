use alloy_primitives::B256;
use alloy_rlp::{Decodable, Encodable};
use serde::{Deserialize, Serialize};
use tree_hash_derive::TreeHash;

#[derive(
    arbitrary::Arbitrary, Debug, Clone, PartialEq, Serialize, Deserialize, Hash, TreeHash, Default,
)]
pub struct AttestationData {
    /// TODO: define what this data is exactly going to be
    pub data: B256,
}

impl AttestationData {
    pub fn new(transaction_id: B256) -> Self {
        Self {
            data: transaction_id,
        }
    }

    pub fn data(&self) -> B256 {
        self.data
    }

    pub fn fixed_len() -> usize {
        B256::len_bytes()
    }
}

impl Encodable for AttestationData {
    fn length(&self) -> usize {
        self.data.length()
    }

    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.data.encode(out);
    }
}

impl Decodable for AttestationData {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            data: Decodable::decode(buf)?,
        })
    }
}
