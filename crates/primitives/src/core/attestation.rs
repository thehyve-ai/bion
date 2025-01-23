use super::{attestation_data::AttestationData, transaction::TransactionId};
use lighthouse_bls::{AggregateSignature, Signature};
use serde::{Deserialize, Serialize};
use ssz_types::{typenum::U131072, BitList};

#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error("RlpTypesError")]
    SszTypesError(ssz_types::Error),

    #[error("AlreadySigned: {0}")]
    AlreadySigned(usize),

    #[error("IncorrectStateVariant")]
    IncorrectStateVariant,

    #[error("InvalidCommitteeLength")]
    InvalidCommitteeLength,

    #[error("InvalidCommitteeIndex")]
    InvalidCommitteeIndex,
}

// impl From<ssz_types::Error> for AttestationError {
//     fn from(e: ssz_types::Error) -> Self {
//         AttestationError::SszTypesError(e)
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attestation {
    /// A bitfield where each bit corresponds to the signature of the chunk at the same index.
    pub chunks_bits: BitList<U131072>,
    /// The signing root
    pub data: AttestationData,
    /// The aggregated signature
    pub signature: AggregateSignature,
}

impl Attestation {
    pub fn get_chunk_bit(&self, index: usize) -> Result<bool, ssz_types::Error> {
        self.chunks_bits.get(index)
    }

    pub fn is_zero(&self) -> bool {
        self.chunks_bits.is_zero()
    }

    pub fn is_fully_attested_for(&self) -> bool {
        self.chunks_bits.iter().all(|b| b)
    }

    pub fn empty_unsigned(
        num_chunks: usize,
        transaction_id: TransactionId,
    ) -> Result<Self, AttestationError> {
        Ok(Attestation {
            chunks_bits: BitList::with_capacity(num_chunks)
                .map_err(|_| AttestationError::InvalidCommitteeLength)?,
            data: AttestationData {
                data: transaction_id.id(),
            },
            signature: AggregateSignature::infinity(),
        })
    }

    pub fn add_signature(
        &mut self,
        signature: &Signature,
        chunk_index: usize,
    ) -> Result<(), AttestationError> {
        if self
            .chunks_bits
            .get(chunk_index)
            .map_err(AttestationError::SszTypesError)?
        {
            Err(AttestationError::AlreadySigned(chunk_index))
        } else {
            self.chunks_bits
                .set(chunk_index, true)
                .map_err(AttestationError::SszTypesError)?;
            self.signature.add_assign(signature);
            Ok(())
        }
    }

    pub fn aggregate(&mut self, other: &Self) {
        self.chunks_bits = self.chunks_bits.union(&other.chunks_bits);
        self.signature.add_assign_aggregate(&other.signature);
    }
}
