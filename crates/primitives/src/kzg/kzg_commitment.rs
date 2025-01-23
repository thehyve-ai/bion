// Inspired from lighthouse

use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;

use alloy_primitives::{FixedBytes, B256};
use alloy_rlp::{Decodable, Encodable};
use ethereum_hashing::hash_fixed;
use kzg::types::g1::FsG1;
use kzg_traits::eip_4844::BYTES_PER_COMMITMENT;
use kzg_traits::G1;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tree_hash::{PackedEncoding, TreeHash};

use crate::utils::hex_encode;

pub const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
#[repr(C)]
pub struct KzgCommitment(FixedBytes<BYTES_PER_COMMITMENT>);

impl KzgCommitment {
    pub fn new(commitment: FsG1) -> Self {
        commitment.into()
    }
}
impl Encodable for KzgCommitment {
    fn length(&self) -> usize {
        self.0.length()
    }

    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.0.encode(out)
    }
}

impl Decodable for KzgCommitment {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self(FixedBytes::<BYTES_PER_COMMITMENT>::decode(buf)?))
    }
}

impl Default for KzgCommitment {
    fn default() -> Self {
        Self(FixedBytes::new([0; BYTES_PER_COMMITMENT]))
    }
}

impl KzgCommitment {
    pub fn proj(&self) -> Result<FsG1, String> {
        FsG1::from_bytes(self.0.as_slice())
    }
}
impl TryFrom<KzgCommitment> for FsG1 {
    type Error = String;

    fn try_from(value: KzgCommitment) -> Result<Self, Self::Error> {
        FsG1::from_bytes(value.0.as_slice())
    }
}

impl From<FsG1> for KzgCommitment {
    fn from(value: FsG1) -> Self {
        let bytes = value.to_bytes();

        Self(FixedBytes::new(bytes))
    }
}

impl From<FixedBytes<48>> for KzgCommitment {
    fn from(value: FixedBytes<48>) -> Self {
        Self(value)
    }
}

impl KzgCommitment {
    pub fn calculate_versioned_hash(&self) -> B256 {
        let mut versioned_hash = hash_fixed(self.0.as_slice());
        versioned_hash[0] = VERSIONED_HASH_VERSION_KZG;
        B256::from_slice(versioned_hash.as_slice())
    }
}

impl Display for KzgCommitment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for i in &self.0[0..2] {
            write!(f, "{:02x}", i)?;
        }
        write!(f, "…")?;
        for i in &self.0[BYTES_PER_COMMITMENT - 2..BYTES_PER_COMMITMENT] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl Debug for KzgCommitment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex_encode(self.0))
    }
}

impl TreeHash for KzgCommitment {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        <[u8; BYTES_PER_COMMITMENT] as TreeHash>::tree_hash_type()
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        self.0.tree_hash_packed_encoding()
    }

    fn tree_hash_packing_factor() -> usize {
        <[u8; BYTES_PER_COMMITMENT] as TreeHash>::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> tree_hash::Hash256 {
        self.0.tree_hash_root()
    }
}

impl Serialize for KzgCommitment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

impl<'de> Deserialize<'de> for KzgCommitment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::from_str(&string).map_err(serde::de::Error::custom)
    }
}

impl FromStr for KzgCommitment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix("0x") {
            let bytes = hex::decode(stripped).map_err(|e| e.to_string())?;
            if bytes.len() == BYTES_PER_COMMITMENT {
                let mut kzg_commitment_bytes = [0; BYTES_PER_COMMITMENT];
                kzg_commitment_bytes[..].copy_from_slice(&bytes);
                Ok(Self(FixedBytes::new(kzg_commitment_bytes)))
            } else {
                Err(format!(
                    "InvalidByteLength: got {}, expected {}",
                    bytes.len(),
                    BYTES_PER_COMMITMENT
                ))
            }
        } else {
            Err("must start with 0x".to_string())
        }
    }
}

impl arbitrary::Arbitrary<'_> for KzgCommitment {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let mut bytes = [0u8; BYTES_PER_COMMITMENT];
        u.fill_buffer(&mut bytes)?;
        Ok(KzgCommitment(FixedBytes::new(bytes)))
    }
}

#[test]
fn kzg_commitment_display() {
    let display_commitment_str = "0x53fa…adac";
    let display_commitment = KzgCommitment::from_str(
        "0x53fa09af35d1d1a9e76f65e16112a9064ce30d1e4e2df98583f0f5dc2e7dd13a4f421a9c89f518fafd952df76f23adac",
    )
    .unwrap()
    .to_string();

    assert_eq!(display_commitment, display_commitment_str);
}

#[test]
fn kzg_commitment_debug() {
    let debug_commitment_str =
        "0x53fa09af35d1d1a9e76f65e16112a9064ce30d1e4e2df98583f0f5dc2e7dd13a4f421a9c89f518fafd952df76f23adac";
    let debug_commitment = KzgCommitment::from_str(debug_commitment_str).unwrap();

    assert_eq!(format!("{debug_commitment:?}"), debug_commitment_str);
}
