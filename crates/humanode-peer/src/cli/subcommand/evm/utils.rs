//! Common utils to process ethereum keys.

/// A helper function to extract and print keys based on provided mnemonic.
pub fn extract_and_print_keys(mnemonic: &bip39::Mnemonic, account_index: Option<u32>) {
    let key_data = crypto_utils_evm::KeyData::from_mnemonic_bip44(mnemonic, "", account_index);

    println!("Address:      {:?}", key_data.account);
    println!("Mnemonic:     {}", key_data.mnemonic);
    println!("Private Key:  {:?}", key_data.private_key);
    println!("Path:         {}", key_data.derivation_path);
}
