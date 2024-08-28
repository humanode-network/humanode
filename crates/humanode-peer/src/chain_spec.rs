//! Provides the [`ChainSpec`] portion of the config.

use crypto_utils::{authority_keys_from_seed, get_account_id_from_seed};
use frame_support::BoundedVec;
use hex_literal::hex;
use humanode_runtime::{
    opaque::SessionKeys, robonode, token_claims::types::ClaimInfo, AccountId, BabeConfig, Balance,
    BalancesConfig, BioauthConfig, BootnodesConfig, ChainPropertiesConfig, EVMConfig,
    EthereumAddress, EthereumChainIdConfig, EthereumConfig, EvmAccountId, GenesisConfig,
    GrandpaConfig, ImOnlineConfig, SessionConfig, Signature, SudoConfig, SystemConfig,
    TokenClaimsConfig, WASM_BINARY,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec_derive::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{app_crypto::sr25519, traits::Verify};

/// The concrete chain spec type we're using for the humanode network.
pub type ChainSpec = sc_service::GenericChainSpec<humanode_runtime::GenesisConfig, Extensions>;

/// Extensions for `ChainSpec`.
#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension, Default,
)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// The URL of robonode to authenticate with.
    pub robonode_url: Option<String>,
    /// The Web App URL, necessary for printing the Web App QR Code.
    pub webapp_url: Option<String>,
}

/// The public key for the accounts.
type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn account_id(seed: &str) -> AccountId {
    get_account_id_from_seed::<sr25519::Public, AccountPublic, AccountId>(seed)
}

/// Generate consensus authority keys.
pub fn authority_keys(seed: &str) -> (AccountId, BabeId, GrandpaId, ImOnlineId) {
    authority_keys_from_seed::<sr25519::Public, AccountPublic, AccountId>(seed)
}

/// Generate an EVM account from seed.
pub fn evm_account_id_from_dev_seed(account_index: u32) -> EvmAccountId {
    let key_data = crypto_utils_evm::KeyData::from_dev_seed(account_index);
    key_data.account
}

/// The default Humanode ss58 prefix.
pub const SS58_PREFIX: u16 = 5234;
/// Default ethereum chain id.
pub const ETH_CHAIN_ID: u64 = 5234;
/// The development robonode public key.
pub const DEFAULT_DEV_ROBONODE_PUBLIC_KEY: [u8; 32] =
    hex!("5dde03934419252d13336e5a5881f5b1ef9ea47084538eb229f86349e7f394ab");

/// Provide the dev robonode public key.
///
/// This fn cosults undocumented `DEV_ROBONODE_PUBLIC_KEY` env var and attempts to use that first
/// to allow for the key override. This override mechanism is useful during development, and is
/// intended only for development.
fn dev_robonode_public_key(default: &'static [u8]) -> Result<robonode::PublicKey, String> {
    match std::env::var("DEV_ROBONODE_PUBLIC_KEY") {
        Ok(val) => {
            let val = hex::decode(val)
                .map_err(|err| format!("robonode public key in not in hex format: {err:?}"))?;
            robonode::PublicKey::from_bytes(&val)
        }
        Err(std::env::VarError::NotPresent) => robonode::PublicKey::from_bytes(default),
        Err(std::env::VarError::NotUnicode(val)) => {
            return Err(format!("invalid robonode public key: {val:?}"))
        }
    }
    .map_err(|err| format!("unable to parse robonode public key: {err:?}"))
}

/// A helper function to construct an EVM genesis account with a predefined balance.
fn evm_genesis_account(init_balance: Balance) -> fp_evm::GenesisAccount {
    fp_evm::GenesisAccount {
        balance: init_balance.into(),
        code: Default::default(),
        nonce: Default::default(),
        storage: Default::default(),
    }
}

/// A configuration for local testnet.
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    let robonode_public_key = dev_robonode_public_key(&DEFAULT_DEV_ROBONODE_PUBLIC_KEY)?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys("Alice"), authority_keys("Bob")],
                // Sudo account
                account_id("Alice"),
                // Pre-funded accounts
                vec![
                    account_id("Alice"),
                    account_id("Bob"),
                    account_id("Charlie"),
                    account_id("Dave"),
                    account_id("Eve"),
                    account_id("Ferdie"),
                    account_id("Alice//stash"),
                    account_id("Bob//stash"),
                    account_id("Charlie//stash"),
                    account_id("Dave//stash"),
                    account_id("Eve//stash"),
                    account_id("Ferdie//stash"),
                ],
                vec![
                    evm_account_id_from_dev_seed(0),
                    evm_account_id_from_dev_seed(1),
                ],
                robonode_public_key,
                vec![account_id("Alice")],
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Fork
        None,
        // Properties
        Some(properties()),
        // Extensions
        Extensions::default(),
    ))
}

/// A configuration for dev.
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    let robonode_public_key = dev_robonode_public_key(&DEFAULT_DEV_ROBONODE_PUBLIC_KEY)?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys("Alice")],
                // Sudo account
                account_id("Alice"),
                // Pre-funded accounts
                vec![
                    account_id("Alice"),
                    account_id("Bob"),
                    account_id("Alice//stash"),
                    account_id("Bob//stash"),
                ],
                vec![
                    evm_account_id_from_dev_seed(0),
                    evm_account_id_from_dev_seed(1),
                ],
                robonode_public_key,
                vec![account_id("Alice")],
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Fork
        None,
        // Properties
        Some(properties()),
        // Extensions
        Extensions::default(),
    ))
}

/// A configuration for benchmarking.
pub fn benchmark_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // Public key is taken from the first entry of https://ed25519.cr.yp.to/python/sign.input
    // Must be compatible with secret key provided in AuthTicketSigner trait implemented for
    // Runtime in crates/humanode-runtime/src/lib.rs.
    let robonode_public_key = dev_robonode_public_key(&hex!(
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
    ))?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Benchmark",
        // ID
        "benchmark",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys("Alice")],
                // Sudo account
                account_id("Alice"),
                // Pre-funded accounts
                vec![
                    account_id("Alice"),
                    account_id("Bob"),
                    account_id("Alice//stash"),
                    account_id("Bob//stash"),
                ],
                vec![
                    evm_account_id_from_dev_seed(0),
                    evm_account_id_from_dev_seed(1),
                ],
                robonode_public_key,
                vec![account_id("Alice")],
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Fork
        None,
        // Properties
        Some(properties()),
        // Extensions
        Extensions::default(),
    ))
}

/// The standard balance we put into genesis-endowed dev accounts.
const DEV_ACCOUNT_BALANCE: Balance = 10u128.pow(18 + 6);

/// The existential deposit of the runtime.
const EXISTENTIAL_DEPOSIT: Balance = 500;

/// The initial pot accounts balance for testnet genesis.
const INITIAL_POT_ACCOUNT_BALANCE: Balance = EXISTENTIAL_DEPOSIT + DEV_ACCOUNT_BALANCE;

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, BabeId, GrandpaId, ImOnlineId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    evm_endowed_accounts: Vec<EvmAccountId>,
    robonode_public_key: robonode::PublicKey,
    bootnodes: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance.
            balances: {
                let pot_accounts = vec![
                    (
                        humanode_runtime::TreasuryPot::account_id(),
                        INITIAL_POT_ACCOUNT_BALANCE,
                    ),
                    (
                        humanode_runtime::FeesPot::account_id(),
                        INITIAL_POT_ACCOUNT_BALANCE,
                    ),
                    (
                        humanode_runtime::TokenClaimsPot::account_id(),
                        INITIAL_POT_ACCOUNT_BALANCE,
                    ),
                    (
                        humanode_runtime::NativeToEvmSwapBridgePot::account_id(),
                        INITIAL_POT_ACCOUNT_BALANCE,
                    ),
                ];
                pot_accounts
                    .into_iter()
                    .chain(
                        endowed_accounts
                            .into_iter()
                            .map(|k| (k, DEV_ACCOUNT_BALANCE)),
                    )
                    .collect()
            },
        },
        session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        SessionKeys {
                            babe: x.1.clone(),
                            grandpa: x.2.clone(),
                            im_online: x.3.clone(),
                        },
                    )
                })
                .collect::<Vec<_>>(),
        },
        babe: BabeConfig {
            authorities: vec![],
            epoch_config: Some(humanode_runtime::BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: GrandpaConfig {
            authorities: vec![],
        },
        im_online: ImOnlineConfig { keys: vec![] },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
        bootnodes: BootnodesConfig {
            bootnodes: bootnodes.try_into().unwrap(),
        },
        bioauth: BioauthConfig {
            robonode_public_key,
            consumed_auth_ticket_nonces: BoundedVec::default(),
            active_authentications: BoundedVec::default(),
            black_listed_validator_public_keys: BoundedVec::default(),
        },
        chain_properties: ChainPropertiesConfig {
            ss58_prefix: SS58_PREFIX,
        },
        ethereum_chain_id: EthereumChainIdConfig {
            chain_id: ETH_CHAIN_ID,
        },
        evm: EVMConfig {
            accounts: {
                let evm_pot_accounts = vec![(
                    humanode_runtime::EvmToNativeSwapBridgePot::account_id(),
                    evm_genesis_account(INITIAL_POT_ACCOUNT_BALANCE),
                )];

                evm_pot_accounts
                    .into_iter()
                    .chain(
                        evm_endowed_accounts
                            .into_iter()
                            .map(|k| (k, evm_genesis_account(DEV_ACCOUNT_BALANCE))),
                    )
                    .collect()
            },
        },
        evm_accounts_mapping: Default::default(),
        ethereum: EthereumConfig {},
        transaction_payment: Default::default(),
        fees_pot: Default::default(),
        treasury_pot: Default::default(),
        token_claims_pot: Default::default(),
        native_to_evm_swap_bridge_pot: Default::default(),
        evm_to_native_swap_bridge_pot: Default::default(),
        token_claims: TokenClaimsConfig {
            claims: vec![(
                EthereumAddress(hex!("bf0b5a4099f0bf6c8bc4252ebec548bae95602ea")),
                ClaimInfo {
                    balance: DEV_ACCOUNT_BALANCE,
                    vesting: vec![].try_into().unwrap(),
                },
            )],
            total_claimable: Some(DEV_ACCOUNT_BALANCE),
        },
        balanced_currency_swap_bridges_initializer: Default::default(),
        dummy_precompiles_code: Default::default(),
    }
}

/// The standard properties we use.
fn properties() -> sc_chain_spec::Properties {
    let properties = serde_json::json!({
        "tokenDecimals": 18,
    });
    serde_json::from_value(properties).unwrap() // embedded value, should never fail
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use sp_runtime::BuildStorage;

    use super::*;

    fn assert_genesis_config(
        chain_spec_result: Result<ChainSpec, String>,
    ) -> sp_core::storage::Storage {
        chain_spec_result.unwrap().build_storage().unwrap()
    }

    fn assert_balanced_currency_swap(storage: sp_core::storage::Storage) {
        Into::<sp_io::TestExternalities>::into(storage).execute_with(move || {
            assert!(
                humanode_runtime::BalancedCurrencySwapBridgesInitializer::is_balanced().unwrap()
            );
        });
    }

    #[test]
    fn local_testnet_config_works() {
        let storage = assert_genesis_config(local_testnet_config());
        assert_balanced_currency_swap(storage);
    }

    #[test]
    fn development_config_works() {
        let storage = assert_genesis_config(development_config());
        assert_balanced_currency_swap(storage);
    }

    #[test]
    fn benchmark_config_works() {
        let storage = assert_genesis_config(benchmark_config());
        assert_balanced_currency_swap(storage);
    }

    #[test]
    fn deserialize_bioauth_flow_params_extensions() {
        let expected = Extensions {
            robonode_url: Some("dummy_robonode_url".into()),
            webapp_url: Some("dummy_webapp_url".into()),
        };
        let value = serde_json::json!({
            "robonodeUrl": "dummy_robonode_url",
            "webappUrl": "dummy_webapp_url"
        });

        let sample: Extensions = serde_json::from_value(value).unwrap();

        assert_eq!(sample, expected)
    }

    #[test]
    fn deserialize_chain_spec() {
        let chain_spec_file_content = indoc! {r#"
          {
            "name": "Local Testnet",
            "id": "local_testnet",
            "chainType": "Local",
            "bootNodes": [],
            "telemetryEndpoints": null,
            "protocolId": null,
            "properties": null,
            "robonodeUrl": "dummy_robonode_url",
            "webappUrl": "dummy_webapp_url",
            "consensusEngine": null,
            "codeSubstitutes": {},
            "genesis": {}
          }
        "#};
        let bytes = chain_spec_file_content.as_bytes();
        let sample: ChainSpec = ChainSpec::from_json_bytes(bytes).unwrap();

        let expected = Extensions {
            robonode_url: Some("dummy_robonode_url".into()),
            webapp_url: Some("dummy_webapp_url".into()),
        };

        assert_eq!(sample.extensions().to_owned(), expected)
    }
}
