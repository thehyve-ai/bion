use alloy_primitives::FixedBytes;
use alloy_rlp::{Decodable, Encodable};
use kzg::types::g1::{FsG1, FsG1Affine};
use kzg_traits::eip_4844::BYTES_PER_PROOF;
use kzg_traits::{G1Affine, G1};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use tree_hash::{PackedEncoding, TreeHash};

use crate::utils::hex_encode;

#[derive(PartialEq, Hash, Eq, Clone, Copy)]
#[repr(C)]
pub struct KzgProof(FixedBytes<BYTES_PER_PROOF>);

impl KzgProof {
    pub fn new(data: FsG1) -> Self {
        Self(FixedBytes::new(data.to_bytes()))
    }
}

impl KzgProof {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0 .0
    }

    pub fn as_g1(&self) -> Result<FsG1, String> {
        FsG1::from_bytes(self.as_bytes())
    }
}

impl Encodable for KzgProof {
    fn length(&self) -> usize {
        self.0.length()
    }

    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.0.encode(out);
    }
}

impl Decodable for KzgProof {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self(FixedBytes::<BYTES_PER_PROOF>::decode(buf)?))
    }
}

impl Default for KzgProof {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<KzgProof> for FixedBytes<BYTES_PER_PROOF> {
    fn from(from: KzgProof) -> FixedBytes<BYTES_PER_PROOF> {
        from.0
    }
}

impl From<FsG1Affine> for KzgProof {
    fn from(value: FsG1Affine) -> Self {
        let bytes = value.to_proj().to_bytes();
        Self(FixedBytes::new(bytes))
    }
}
impl TryInto<FsG1> for KzgProof {
    type Error = String;

    fn try_into(self) -> Result<FsG1, Self::Error> {
        FsG1::from_bytes(self.0.as_slice())
    }
}

impl TryInto<FsG1Affine> for KzgProof {
    type Error = String;

    fn try_into(self) -> Result<FsG1Affine, Self::Error> {
        let g1: FsG1 = self.try_into()?;
        Ok(FsG1Affine::into_affine(&g1))
    }
}

impl KzgProof {
    /// Creates a valid proof using `G1_POINT_AT_INFINITY`.
    pub fn empty() -> Self {
        let mut bytes = [0; BYTES_PER_PROOF];
        bytes[0] = 0xc0;
        Self(FixedBytes::new(bytes))
    }
}

impl fmt::Display for KzgProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex_encode(self.0))
    }
}

impl From<[u8; BYTES_PER_PROOF]> for KzgProof {
    fn from(bytes: [u8; BYTES_PER_PROOF]) -> Self {
        Self(FixedBytes::new(bytes))
    }
}

impl From<KzgProof> for [u8; BYTES_PER_PROOF] {
    fn from(from: KzgProof) -> [u8; BYTES_PER_PROOF] {
        from.0 .0
    }
}

impl TreeHash for KzgProof {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        <[u8; BYTES_PER_PROOF]>::tree_hash_type()
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        self.0.tree_hash_packed_encoding()
    }

    fn tree_hash_packing_factor() -> usize {
        <[u8; BYTES_PER_PROOF]>::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> tree_hash::Hash256 {
        self.0.tree_hash_root()
    }
}

impl Serialize for KzgProof {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for KzgProof {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::from_str(&string).map_err(serde::de::Error::custom)
    }
}

impl FromStr for KzgProof {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix("0x") {
            let bytes = hex::decode(stripped).map_err(|e| e.to_string())?;
            if bytes.len() == BYTES_PER_PROOF {
                let mut kzg_proof_bytes = [0; BYTES_PER_PROOF];
                kzg_proof_bytes[..].copy_from_slice(&bytes);
                Ok(Self(FixedBytes::new(kzg_proof_bytes)))
            } else {
                Err(format!(
                    "InvalidByteLength: got {}, expected {}",
                    bytes.len(),
                    BYTES_PER_PROOF
                ))
            }
        } else {
            Err("must start with 0x".to_string())
        }
    }
}

impl Debug for KzgProof {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex_encode(self.0))
    }
}

impl arbitrary::Arbitrary<'_> for KzgProof {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let mut bytes = [0u8; BYTES_PER_PROOF];
        u.fill_buffer(&mut bytes)?;
        Ok(KzgProof(FixedBytes::new(bytes)))
    }
}
