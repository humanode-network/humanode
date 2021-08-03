//! Provides the [`ChainSpec`] portion of the config.

use hex_literal::hex;
use humanode_runtime::{
    AccountId, AuraConfig, BalancesConfig, GenesisConfig, PalletBioauthConfig,
    RobonodePublicKeyWrapper, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
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

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> AuraId {
    get_from_seed::<AuraId>(s)
}

/// A configuration for local testnet.
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    let robonode_public_key = RobonodePublicKeyWrapper::from_bytes(
        &hex!("5dde03934419252d13336e5a5881f5b1ef9ea47084538eb229f86349e7f394ab")[..],
    )
    .map_err(|err| format!("{:?}", err))?;

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
                    balances: vec![
                        (
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            1 << 60,
                        ),
                        (
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            1 << 60,
                        ),
                        (get_account_id_from_seed::<sr25519::Public>("Bob"), 1 << 60),
                        (
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                            1 << 60,
                        ),
                    ],
                },
                pallet_aura: AuraConfig {
                    authorities: vec![
                        authority_keys_from_seed("Alice"),
                        authority_keys_from_seed("Bob"),
                    ],
                },
                pallet_sudo: SudoConfig {
                    // Assign network admin rights.
                    key: get_account_id_from_seed::<sr25519::Public>("Alice"),
                },
                pallet_bioauth: PalletBioauthConfig {
                    // Add Alice AuraId to StoredAuthTickets for producing blocks
                    stored_auth_tickets: vec![pallet_bioauth::StoredAuthTicket {
                        public_key: authority_keys_from_seed("Alice").as_slice().to_vec(),
                        nonce: "1".as_bytes().to_vec(),
                    }],
                    robonode_public_key,
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
