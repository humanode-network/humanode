//! Provides the [`ChainSpec`] portion of the config.

use humanode_runtime::{
    AccountId, BalancesConfig, GenesisConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_runtime::{
    app_crypto::{sr25519, Pair, Public},
    traits::{IdentifyAccount, Verify},
};

/// The concrete chain spec type we're using for the humanode network.
pub type ChainSpec = sc_service::GenericChainSpec<humanode_runtime::GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// The public key for the accounts.
type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// A configuration for local testnet.
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            GenesisConfig {
                frame_system: SystemConfig {
                    // Add Wasm runtime to storage.
                    code: wasm_binary.to_vec(),
                    changes_trie_config: Default::default(),
                },
                pallet_balances: BalancesConfig {
                    // Configure endowed accounts with initial balance of 1 << 60.
                    balances: vec![],
                },
                pallet_sudo: SudoConfig {
                    // Assign network admin rights.
                    key: get_account_id_from_seed::<sr25519::Public>("Alice"),
                },
            }
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}
