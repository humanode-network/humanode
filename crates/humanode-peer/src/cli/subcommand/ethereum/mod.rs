//! Ethereum subcommands and related common utilities.

use bip39::{Language, Mnemonic, MnemonicType, Seed};
use libsecp256k1::{PublicKey, SecretKey};
use sha3::{Digest, Keccak256};
use sp_core::{H160, H256};
use structopt::StructOpt;
use tiny_hderive::bip32::ExtendedPrivKey;

/// Subcommands for the `ethereum` command.
#[derive(Debug, StructOpt)]
pub enum EthereumCmd {
    /// Ethereum key utilities (Generate or inspect mnemonic).
    Key(GenerateAccountCmd),
}

/// The `ethereum key` command.
#[derive(Debug, StructOpt)]
pub struct GenerateAccountCmd {
    /// Generate 24 words mnemonic instead of 12
    #[structopt(long, short = "w")]
    w24: bool,

    /// Specify the mnemonic
    #[structopt(long, short = "m")]
    mnemonic: Option<String>,

    /// The account index to use in the derivation path
    #[structopt(long = "account-index", short = "a")]
    account_index: Option<u32>,
}

impl GenerateAccountCmd {
    /// Run the key command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        // Retrieve the mnemonic from the args or generate random ones
        let mnemonic = if let Some(phrase) = &self.mnemonic {
            Mnemonic::from_phrase(phrase, Language::English).unwrap()
        } else {
            match self.w24 {
                true => Mnemonic::new(MnemonicType::Words24, Language::English),
                false => Mnemonic::new(MnemonicType::Words12, Language::English),
            }
        };

        // Retrieves the seed from the mnemonic
        let seed = Seed::new(&mnemonic, "");

        // Generate the derivation path from the account-index
        let derivation_path = format!("m/44'/60'/0'/0/{}", self.account_index.unwrap_or(0));

        // Derives the private key from
        let ext = ExtendedPrivKey::derive(seed.as_bytes(), derivation_path.as_str()).unwrap();
        let private_key = SecretKey::parse_slice(&ext.secret()).unwrap();

        // Retrieves the public key
        let public_key = PublicKey::from_secret_key(&private_key);

        // Convert into Ethereum-style address.
        let mut m = [0u8; 64];
        m.copy_from_slice(&public_key.serialize()[1..65]);
        let account = H160::from(H256::from_slice(Keccak256::digest(&m).as_slice()));

        println!("Address:      {:?}", account);
        println!("Mnemonic:     {}", mnemonic.phrase());
        println!("Private Key:  {:?}", H256::from(private_key.serialize()));
        println!("Path:         {}", derivation_path);

        Ok(())
    }
}
