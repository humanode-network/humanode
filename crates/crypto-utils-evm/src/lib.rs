//! Various crypto helper functions for EVM.

use sp_core::{H160, H256};

/// A structure representing the key information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyData {
    /// The account address.
    pub account: H160,
    /// The private key of this account.
    pub private_key: H256,
    /// The mnemonic phrase.
    pub mnemonic: String,
    /// The drivation path used for this account.
    pub derivation_path: bip32::DerivationPath,
}

impl KeyData {
    /// Create a new [`KeyData`] from the given BIP39 mnemonic and an account index.
    pub fn from_mnemonic_bip39(
        mnemonic: &bip39::Mnemonic,
        password: &str,
        derivation_path: &bip32::DerivationPath,
    ) -> Self {
        // Retrieve the seed from the mnemonic.
        let seed = bip39::Seed::new(mnemonic, password);

        // Derives the private key from.
        let ext = bip32::XPrv::derive_from_path(seed, derivation_path).unwrap();

        let private_key = libsecp256k1::SecretKey::parse_slice(&ext.to_bytes()).unwrap();

        // Retrieves the public key.
        let public_key = libsecp256k1::PublicKey::from_secret_key(&private_key);

        // Convert into Ethereum-style address.
        let mut raw_public_key = [0u8; 64];
        raw_public_key.copy_from_slice(&public_key.serialize()[1..65]);

        use sha3::Digest;
        let digest = sha3::Keccak256::digest(raw_public_key);

        let account = H160::from(H256::from_slice(digest.as_ref()));

        Self {
            account,
            mnemonic: mnemonic.phrase().to_owned(),
            private_key: H256::from(private_key.serialize()),
            derivation_path: derivation_path.clone(),
        }
    }

    /// Construct the key info from the BIP39 mnemonic using BIP44 convenions.
    pub fn from_mnemonic_bip44(
        mnemonic: &bip39::Mnemonic,
        password: &str,
        account_index: Option<u32>,
    ) -> Self {
        let derivation_path = format!("m/44'/60'/0'/0/{}", account_index.unwrap_or(0));
        let derivation_path = derivation_path.parse().unwrap();
        Self::from_mnemonic_bip39(mnemonic, password, &derivation_path)
    }

    /// Construct the key info from the BIP39 mnemonic phrase (in English) using BIP44 convenions.
    /// If you need other language - use [`Self::from_mnemonic_bip44`].
    pub fn from_phrase_bip44(phrase: &str, password: &str, account_index: Option<u32>) -> Self {
        let mnemonic = bip39::Mnemonic::from_phrase(phrase, bip39::Language::English).unwrap();
        Self::from_mnemonic_bip44(&mnemonic, password, account_index)
    }

    /// Construct the key info from the account on the Substrate standard dev seed.
    pub fn from_dev_seed(account_index: u32) -> Self {
        Self::from_phrase_bip44(sp_core::crypto::DEV_PHRASE, "", Some(account_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mnemonic_bip44() {
        let cases = [
            (
                "test test test test test test test test test test test junk",
                "",
                KeyData {
                    account: H160(hex_literal::hex!(
                        "f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
                    )),
                    private_key: H256(hex_literal::hex!(
                        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                    )),
                    mnemonic: "test test test test test test test test test test test junk".into(),
                    derivation_path: "m/44'/60'/0'/0/0".parse().unwrap(),
                },
            ),
            (
                "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
                "Substrate",
                KeyData {
                    account: H160(hex_literal::hex!(
                        "cf1269fb02698ab9ee45426297e20c84142d9195"
                    )),
                    private_key: H256(hex_literal::hex!(
                        "b29728f71053098351f20350e7087dcb091b151689f8a878734b519901d19853"
                    )),
                    mnemonic: "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong".into(),
                    derivation_path: "m/44'/60'/0'/0/0".parse().unwrap(),
                },
            ),
        ];

        for (phrase, pw, expected_key_info) in cases {
            let key_info = KeyData::from_phrase_bip44(phrase, pw, None);
            assert_eq!(key_info, expected_key_info);
        }
    }

    #[test]
    fn dev_seed() {
        let cases = [
            (
                0,
                KeyData {
                    account: H160(hex_literal::hex!(
                        "f24ff3a9cf04c71dbc94d0b566f7a27b94566cac"
                    )),
                    private_key: H256(hex_literal::hex!(
                        "5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133"
                    )),
                    mnemonic:
                        "bottom drive obey lake curtain smoke basket hold race lonely fit walk"
                            .into(),
                    derivation_path: "m/44'/60'/0'/0/0".parse().unwrap(),
                },
            ),
            (
                1,
                KeyData {
                    account: H160(hex_literal::hex!(
                        "3cd0a705a2dc65e5b1e1205896baa2be8a07c6e0"
                    )),
                    private_key: H256(hex_literal::hex!(
                        "8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b"
                    )),
                    mnemonic:
                        "bottom drive obey lake curtain smoke basket hold race lonely fit walk"
                            .into(),
                    derivation_path: "m/44'/60'/0'/0/1".parse().unwrap(),
                },
            ),
        ];

        for (account_index, expected_key_info) in cases {
            let key_info = KeyData::from_dev_seed(account_index);
            assert_eq!(key_info, expected_key_info);
        }
    }
}
