use sp_runtime::{
    app_crypto::{sr25519, Pair, Public},
    traits::{IdentifyAccount, Verify},
};

use super::*;
use crate::opaque::SessionKeys;

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
pub fn new_test_ext_with() -> sp_io::TestExternalities {
    let authorities = vec![
        authority_keys_from_seed("Alice"),
        authority_keys_from_seed("Bob"),
    ];
    let endowed_accounts = vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
    ];
    // Build test genesis.
    let config = GenesisConfig {
        ethereum_chain_id: EthereumChainIdConfig { chain_id: 5234 },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 10000000000000))
                .collect(),
        },
        session: SessionConfig {
            keys: authorities
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
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        bootnodes: BootnodesConfig {
            bootnodes: endowed_accounts.try_into().unwrap(),
        },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
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

#[test]
fn it_works() {
    let chain_id = 5234;

    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Assert the state.
        assert_eq!(EthereumChainId::chain_id(), chain_id);
    })
}
