use alloy_rlp::{Decodable, Encodable};
use kzg::types::{fr::FsFr, g1::FsG1};
use kzg_traits::Fr;

use crate::kzg::{Coefficient, KzgProof};

pub struct Blob {
    pub input_data: Vec<u8>,
    pub length: u64,
    pub padded_length: u64,
}

/// An erasure coded piece of the blob in coefficient form. Includes the KZG proof of the
/// coefficients.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[repr(C)]
pub struct BlobShard {
    pub coefficients: Vec<Coefficient>,
    pub proof: KzgProof,
}

impl Encodable for BlobShard {
    fn length(&self) -> usize {
        self.coefficients.length() + self.proof.length()
    }

    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.coefficients.encode(out);
        self.proof.encode(out);
    }
}

impl Decodable for BlobShard {
    fn decode(buf: &mut &[u8]) -> Result<Self, alloy_rlp::Error> {
        Ok(Self {
            coefficients: Decodable::decode(buf)?,
            proof: Decodable::decode(buf)?,
        })
    }
}

impl BlobShard {
    pub fn new(coefficients: Vec<FsFr>, proof: FsG1) -> Self {
        let coefficients = coefficients
            .into_iter()
            .map(|c| Coefficient::new(c))
            .collect();

        let proof = KzgProof::new(proof);
        Self {
            coefficients,
            proof,
            // todo: change
        }
    }

    pub fn data_size(&self) -> usize {
        self.coefficients.len() * 32
    }

    pub fn coefficients(&self) -> Result<Vec<FsFr>, String> {
        self.coefficients
            .iter()
            .map(|c| FsFr::from_bytes(c.as_bytes()))
            .collect()
    }

    pub fn bytes_size(&self) -> usize {
        self.coefficients.len() * 32
    }
}

// impl Decode for BlobShard{

//     fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {

//         let mut builder = ssz::SszDecoderBuilder::new(bytes);
//         builder.register_anonymous_variable_length_item()?;
//         builder.register_type_parameterized(true, BYTES_PER_G1)?;

//         let mut decoder = builder.build();

//     }

// }
