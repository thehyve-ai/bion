use alloy_primitives::FixedBytes;
use alloy_rlp::{Decodable, Encodable};
use kzg::types::fr::FsFr;
use kzg_traits::{eip_4844::BYTES_PER_FIELD_ELEMENT, Fr};
use ssz_types::typenum::U64;
use ssz_types::VariableList;

pub type Coefficients<Size = U64> = VariableList<Coefficient, Size>;

// DH-2
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[repr(C)]
pub struct Coefficient(FixedBytes<BYTES_PER_FIELD_ELEMENT>);

impl Encodable for Coefficient {
    fn length(&self) -> usize {
        self.0.length()
    }

    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.0.encode(out)
    }
}

impl Decodable for Coefficient {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self(FixedBytes::<BYTES_PER_FIELD_ELEMENT>::decode(buf)?))
    }
}

impl Default for Coefficient {
    fn default() -> Self {
        Self(FixedBytes::default())
    }
}

impl Coefficient {
    pub fn new(data: FsFr) -> Self {
        Self(FixedBytes::from_slice(&data.to_bytes()))
    }
    /// Zero-copy conversion to a `Fr`.
    pub fn try_into_fr(self) -> Result<FsFr, String> {
        FsFr::from_bytes(self.0.as_slice())
    }

    pub fn data_size(&self) -> usize {
        self.0.len() * BYTES_PER_FIELD_ELEMENT
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0 .0
    }
}
