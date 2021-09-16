//! Provides the [`ChainSpec`] portion of the config.

use hex_literal::hex;
use humanode_runtime::{
    AccountId, AuraConfig, BalancesConfig, BioauthConfig, GenesisConfig, GrandpaConfig,
    RobonodePublicKeyWrapper, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use pallet_bioauth::StoredAuthTicket;
use sc_chain_spec_derive::ChainSpecExtension;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    app_crypto::{sr25519, Pair, Public},
    traits::{IdentifyAccount, Verify},
};

/// The concrete chain spec type we're using for the humanode network.
pub type ChainSpec = sc_service::GenericChainSpec<humanode_runtime::GenesisConfig, Extensions>;

/// Extensions for ChainSpec.
#[derive(Serialize, Deserialize, Clone, ChainSpecExtension, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// TODO
    #[serde(flatten)]
    pub bioauth_flow_params: BioauthFlowParamsExtension,
}

/// TODO
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BioauthFlowParamsExtension {
    /// The URL of robonode to authenticate with.
    pub robonode_url: Option<String>,
    /// TODO
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

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
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
                vec![pallet_bioauth::StoredAuthTicket {
                    public_key: authority_keys_from_seed("Alice").0,
                    nonce: "1".as_bytes().to_vec(),
                }],
                robonode_public_key,
            )
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
        Extensions::default(),
    ))
}

/// A configuration for dev.
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    let robonode_public_key = RobonodePublicKeyWrapper::from_bytes(
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
                vec![pallet_bioauth::StoredAuthTicket {
                    public_key: authority_keys_from_seed("Alice").0,
                    nonce: "1".as_bytes().to_vec(),
                }],
                robonode_public_key,
            )
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
        Extensions::default(),
    ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    stored_auth_tickets: Vec<StoredAuthTicket<AuraId>>,
    robonode_public_key: RobonodePublicKeyWrapper,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        bioauth: BioauthConfig {
            // Add Alice AuraId to StoredAuthTickets for producing blocks
            stored_auth_tickets,
            robonode_public_key,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_bioauth_flow_params_extensions() {
        // from_json_bytes();
        let expected = Extensions {
            bioauth_flow_params: BioauthFlowParamsExtension {
                robonode_url: Some("http://127.0.0.1:3033".into()),
                webapp_url: Some("https://webapp-test-1.dev.humanode.io".into()),
            },
        };
        let value = serde_json::json!({
            "name": "Local Testnet",
            "id": "local_testnet",
            "chainType": "Local",
            "bootNodes": [
              "/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWCXvRaPbhT6BugLAApEvV3e3dZxKeR8kPtgTJdU6eGzqB"
            ],
            "telemetryEndpoints": null,
            "protocolId": null,
            "properties": null,
            "robonodeUrl": "http://127.0.0.1:3033",
            "webappUrl": "https://webapp-test-1.dev.humanode.io",
            "consensusEngine": null,
            "codeSubstitutes": {}});

        let sample: Extensions = serde_json::from_value(value).unwrap();

        assert_eq!(sample, expected)
    }

    #[test]
    fn deserialize_chain_spec() {
        let chain_spec_file_content = b"{\"name\":\"Local Testnet\",\"id\":\"local_testnet\",\"chainType\":\"Local\",\"bootNodes\":[\"/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWCXvRaPbhT6BugLAApEvV3e3dZxKeR8kPtgTJdU6eGzqB\"],\"telemetryEndpoints\":null,\"protocolId\":null,\"properties\":null,\"robonodeUrl\":\"http://127.0.0.1:3033\",\"webappUrl\":\"https://webapp-test-1.dev.humanode.io\",\"consensusEngine\":null,\"codeSubstitutes\":{},\"genesis\":{\"runtime\":{\"system\":{\"changesTrieConfig\":null,\"code\":\"0x04\"},\"aura\":{\"authorities\":[\"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\",\"5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty\"]},\"balances\":{\"balances\":[[\"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\",1152921504606846976],[\"5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty\",1152921504606846976]]},\"sudo\":{\"key\":\"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\"},\"bioauth\":{\"storedAuthTickets\":[{\"public_key\":\"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\",\"nonce\":[49]}],\"robonodePublicKey\":[93,222,3,147,68,25,37,45,19,51,110,90,88,129,245,177,239,158,164,112,132,83,142,178,41,248,99,73,231,243,148,171]}}}}";
        let bytes = &chain_spec_file_content[..];
        let sample: ChainSpec = ChainSpec::from_json_bytes(bytes).unwrap();

        let expected = Extensions {
            bioauth_flow_params: BioauthFlowParamsExtension {
                robonode_url: Some("http://127.0.0.1:3033".into()),
                webapp_url: Some("https://webapp-test-1.dev.humanode.io".into()),
            },
        };

        assert_eq!(sample.extensions().to_owned(), expected)
    }
}
