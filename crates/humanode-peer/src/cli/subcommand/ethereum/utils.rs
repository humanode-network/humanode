//! Common utils to process ethereum keys.

use bip32::XPrv;
use bip39::{Mnemonic, Seed};
use libsecp256k1::{PublicKey, SecretKey};
use sha3::{Digest, Keccak256};
use sp_core::{H160, H256};

/// A helper function to extract and print keys based on provided mnemonic.
pub fn extract_and_print_keys(
    mnemonic: &Mnemonic,
    account_index: Option<u32>,
) -> sc_cli::Result<()> {
    // Retrieves the seed from the mnemonic.
    let seed = Seed::new(mnemonic, "");

    // Generate the derivation path from the account-index
    let derivation_path = format!("m/44'/60'/0'/0/{}", account_index.unwrap_or(0))
        .parse()
        .expect("Verified DerivationPath is used");

    // Derives the private key from.
    let ext = XPrv::derive_from_path(seed, &derivation_path)
        .expect("Mnemonic is either a new one or verified before");
    let private_key =
        SecretKey::parse_slice(&ext.to_bytes()).expect("Verified ExtendedPrivKey is used");

    // Retrieves the public key.
    let public_key = PublicKey::from_secret_key(&private_key);

    // Convert into Ethereum-style address.
    let mut m = [0u8; 64];
    m.copy_from_slice(&public_key.serialize()[1..65]);
    let account = H160::from(H256::from_slice(Keccak256::digest(m).as_slice()));

    println!("Address:      {account:?}");
    println!("Mnemonic:     {}", mnemonic.phrase());
    println!("Private Key:  {:?}", H256::from(private_key.serialize()));
    println!("Path:         {derivation_path}");

    Ok(())
}
