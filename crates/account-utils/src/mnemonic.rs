use bip39::{Language, Mnemonic, MnemonicType};

/// Returns a random 24-word english mnemonic.
pub fn random_mnemonic() -> Mnemonic {
    Mnemonic::new(MnemonicType::Words24, Language::English)
}

/// Attempts to parse a mnemonic phrase.
pub fn mnemonic_from_phrase(phrase: &str) -> Result<Mnemonic, String> {
    Mnemonic::from_phrase(phrase, Language::English).map_err(|e| e.to_string())
}
