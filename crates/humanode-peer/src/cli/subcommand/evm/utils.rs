//! Common utils to process ethereum keys.

/// A helper function to extract and print keys based on provided mnemonic or the dev mnemonic
/// if none was provided.
pub fn extract_and_print_keys(
    mnemonic: Option<&bip39::Mnemonic>,
    account_index: Option<u32>,
) -> Result<(), crypto_utils_evm::FromMnemonicBip44Error> {
    let key_data = match mnemonic {
        Some(mnemonic) => {
            crypto_utils_evm::KeyData::from_mnemonic_bip44(mnemonic, "", account_index)?
        }
        None => crypto_utils_evm::KeyData::from_dev_seed(account_index.unwrap_or(0)),
    };

    println!("Address:      {:?}", key_data.account);
    println!("Mnemonic:     {}", key_data.mnemonic);
    println!("Private Key:  {:?}", key_data.private_key);
    println!("Path:         {}", key_data.derivation_path);

    Ok(())
}
