use frame_support::{assert_ok, traits::Currency};
use sp_runtime::app_crypto::sr25519;

use super::*;
use crate::{
    crypto::{authority_keys_from_seed, get_account_id_from_seed},
    opaque::SessionKeys,
};

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
        balances: BalancesConfig {
            balances: endowed_accounts.iter().cloned().map(|k| (k, 100)).collect(),
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

#[test]
fn total_issuance_transaction_fee() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Assert the state.
        assert_eq!(Balances::total_issuance(), 200);
        assert_ok!(Balances::transfer(
            Some(get_account_id_from_seed::<sr25519::Public>("Alice")).into(),
            get_account_id_from_seed::<sr25519::Public>("Bob").into(),
            10
        ));
        assert_eq!(
            Balances::total_balance(&get_account_id_from_seed::<sr25519::Public>("Alice")),
            90
        );
        assert_eq!(
            Balances::total_balance(&get_account_id_from_seed::<sr25519::Public>("Bob")),
            110
        );
        assert_eq!(Balances::total_issuance(), 200);
    })
}
