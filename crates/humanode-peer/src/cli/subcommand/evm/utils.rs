//! Common utils to process ethereum keys.

use libsecp256k1::{PublicKey, SecretKey};
use sha3::{Digest, Keccak256};
use sp_core::{H160, H256};

/// A structure representing the key information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInfo {
    /// The account address.
    pub account: H160,
    /// The private key of this account.
    pub private_key: H256,
    /// The mnemonic phrase.
    pub mnemonic: String,
    /// The drivation path used for this account.
    pub derivation_path: bip32::DerivationPath,
}

impl KeyInfo {
    /// Create a new [`KeyInfo`] from the given mnemonic and account index.
    pub fn from_bip39_mnemonic(
        mnemonic: &bip39::Mnemonic,
        password: &str,
        account_index: Option<u32>,
    ) -> Self {
        // Retrieves the seed from the mnemonic.
        let seed = bip39::Seed::new(mnemonic, password);

        // Generate the derivation path from the account-index
        let derivation_path = account_index
            .map(|index| format!("m/44'/60'/0'/0/{index}"))
            .unwrap_or_else(|| "m/44'/60'/0'/0/0".into())
            .parse()
            .expect("Verified DerivationPath is used");

        // Derives the private key from.
        let ext = bip32::XPrv::derive_from_path(seed, &derivation_path)
            .expect("Mnemonic is either a new one or verified before");

        let private_key =
            SecretKey::parse_slice(&ext.to_bytes()).expect("Verified ExtendedPrivKey is used");

        // Retrieves the public key.
        let public_key = PublicKey::from_secret_key(&private_key);

        // Convert into Ethereum-style address.
        let mut m = [0u8; 64];
        m.copy_from_slice(&public_key.serialize()[1..65]);
        let account = H160::from(H256::from_slice(Keccak256::digest(m).as_slice()));

        Self {
            account,
            mnemonic: mnemonic.phrase().to_owned(),
            private_key: H256::from(private_key.serialize()),
            derivation_path,
        }
    }

    /// Print the key info to the stdout.
    pub fn print(&self) {
        println!("Address:      {:?}", self.account);
        println!("Mnemonic:     {}", self.mnemonic);
        println!("Private Key:  {:?}", self.private_key);
        println!("Path:         {}", self.derivation_path);
    }
}

/// A helper function to extract and print keys based on provided mnemonic.
pub fn extract_and_print_keys(mnemonic: &bip39::Mnemonic, account_index: Option<u32>) {
    KeyInfo::from_bip39_mnemonic(mnemonic, "", account_index).print();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mnemonic() {
        let cases = [
            (
                "test test test test test test test test test test test junk",
                "",
                KeyInfo {
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
                KeyInfo {
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

        for (mnemonic, pw, expected_key_info) in cases {
            let mnemonic =
                bip39::Mnemonic::from_phrase(mnemonic, bip39::Language::English).unwrap();
            let key_info = KeyInfo::from_bip39_mnemonic(&mnemonic, pw, None);

            assert_eq!(key_info, expected_key_info);
        }
    }
}
