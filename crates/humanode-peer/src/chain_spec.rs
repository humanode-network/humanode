//! Provides the [`ChainSpec`] portion of the config.

use std::{collections::BTreeMap, str::FromStr};

use hex_literal::hex;
use humanode_runtime::{
    opaque::SessionKeys, robonode, AccountId, BabeConfig, BalancesConfig, BioauthConfig,
    BootnodesConfig, EVMConfig, EthereumChainIdConfig, EthereumConfig, GenesisConfig,
    GrandpaConfig, ImOnlineConfig, SessionConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec_derive::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    app_crypto::{sr25519, Pair, Public},
    traits::{IdentifyAccount, Verify},
};

/// The concrete chain spec type we're using for the humanode network.
pub type ChainSpec = sc_service::GenericChainSpec<humanode_runtime::GenesisConfig, Extensions>;

/// Extensions for ChainSpec.
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

/// Generate consensus authority keys.
pub fn authority_keys_from_seed(seed: &str) -> (AccountId, BabeId, GrandpaId, ImOnlineId) {
    (
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<ImOnlineId>(seed),
    )
}

/// A configuration for local testnet.
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    let robonode_public_key = robonode::PublicKey::from_bytes(
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
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                robonode_public_key,
                vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
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
        None,
        // Extensions
        Extensions::default(),
    ))
}

/// A configuration for dev.
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    let robonode_public_key = robonode::PublicKey::from_bytes(
        &hex!("5dde03934419252d13336e5a5881f5b1ef9ea47084538eb229f86349e7f394ab")[..],
    )
    .map_err(|err| format!("{:?}", err))?;

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
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                robonode_public_key,
                vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
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
        None,
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
    let robonode_public_key = robonode::PublicKey::from_bytes(
        &hex!("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")[..],
    )
    .map_err(|err| format!("{:?}", err))?;

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
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                robonode_public_key,
                vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
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
        None,
        // Extensions
        Extensions::default(),
    ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, BabeId, GrandpaId, ImOnlineId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    robonode_public_key: robonode::PublicKey,
    bootnodes: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
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
            consumed_auth_ticket_nonces: vec![],
            active_authentications: vec![],
        },
        ethereum_chain_id: EthereumChainIdConfig { chain_id: 5234 },
        evm: EVMConfig {
            accounts: {
                let mut map = BTreeMap::new();
                map.insert(
                    // H160 address of Alice dev account
                    // Derived from SS58 (42 prefix) address
                    // SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
                    // hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
                    // Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
                    H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
                        .expect("internal H160 is valid; qed"),
                    fp_evm::GenesisAccount {
                        balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                            .expect("internal U256 is valid; qed"),
                        code: Default::default(),
                        nonce: Default::default(),
                        storage: Default::default(),
                    },
                );
                map.insert(
                    // H160 address of Gerald dev account
                    // Public address: 0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b
                    // Private key: 0x99b3c12287537e38c90a9219d4cb074a89a16e9cdb20bf85728ebd97c343e342
                    // A proper private key should be used to allow testing EVM as Ethereum developer
                    // For example, use it at Metamask, Remix, Truffle configuration, etc
                    // We don't have a good converter between Substrate and Ethereum private keys for now.
                    H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
                        .expect("internal H160 is valid; qed"),
                    fp_evm::GenesisAccount {
                        balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                            .expect("internal U256 is valid; qed"),
                        code: Default::default(),
                        nonce: Default::default(),
                        storage: Default::default(),
                    },
                );
                map
            },
        },
        ethereum: EthereumConfig {},
        dynamic_fee: Default::default(),
        base_fee: Default::default(),
        transaction_payment: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

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
