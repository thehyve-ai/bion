pub use alloy_primitives::Signature as AlloySignature;
use alloy_primitives::{Address, B256, U256};

use super::util::secp256k1;

/// The order of the secp256k1 curve, divided by two. Signatures that should be checked according
/// to EIP-2 should have an S value less than or equal to this.
///
/// `57896044618658097711785492504343953926418782139537452191302581570759080747168`
const SECP256K1N_HALF: U256 = U256::from_be_bytes([
    0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D, 0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
]);

/// Recover signer from message hash, _without ensuring that the signature has a low `s`
/// value_.
///
/// Using this for signature validation will succeed, even if the signature is malleable or not
/// compliant with EIP-2. This is provided for compatibility with old signatures which have
/// large `s` values.
pub fn recover_signer_unchecked(
    signature: &alloy_primitives::Signature,
    hash: B256,
) -> Option<Address> {
    let mut sig: [u8; 65] = [0; 65];

    sig[0..32].copy_from_slice(&signature.r().to_be_bytes::<32>());
    sig[32..64].copy_from_slice(&signature.s().to_be_bytes::<32>());
    sig[64] = signature.v().y_parity_byte();

    // NOTE: we are removing error from underlying crypto library as it will restrain primitive
    // errors and we care only if recovery is passing or not.
    secp256k1::recover_signer_unchecked(&sig, &hash.0).ok()
}

/// Recover signer address from message hash. This ensures that the signature S value is
/// greater than `secp256k1n / 2`, as specified in
/// [EIP-2](https://eips.ethereum.org/EIPS/eip-2).
///
/// If the S value is too large, then this will return `None`
pub fn recover_signer(signature: &alloy_primitives::Signature, hash: B256) -> Option<Address> {
    if signature.s() > SECP256K1N_HALF {
        return None;
    }

    recover_signer_unchecked(signature, hash)
}
