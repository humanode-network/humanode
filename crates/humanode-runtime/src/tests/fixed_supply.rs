//! Tests to verify fixed supply logic.

use frame_support::{
    dispatch::DispatchInfo,
    weights::{DispatchClass, Pays},
};
use sp_runtime::traits::SignedExtension;

use super::*;

const INIT_BALANCE: u128 = 10u128.pow(18 + 6);

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
pub fn new_test_ext_with() -> sp_io::TestExternalities {
    let authorities = vec![authority_keys("Alice")];
    let bootnodes = vec![account_id("Alice")];
    let endowed_accounts = vec![account_id("Alice"), account_id("Bob")];
    // Build test genesis.
    let config = GenesisConfig {
        balances: BalancesConfig {
            balances: {
                let pot_accounts = vec![TreasuryPot::account_id(), FeesPot::account_id()];
                endowed_accounts
                    .iter()
                    .chain(pot_accounts.iter())
                    .cloned()
                    .map(|k| (k, INIT_BALANCE))
                    .collect()
            },
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
            bootnodes: bootnodes.try_into().unwrap(),
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
        let existential_deposit =
            <<Runtime as pallet_balances::Config>::ExistentialDeposit as Get<u128>>::get();
        // Check total issuance before making transfer.
        let total_issuance_before = Balances::total_issuance();
        // Make KeepAlive transfer.
        assert_ok!(Balances::transfer_keep_alive(
            Some(account_id("Bob")).into(),
            account_id("Alice").into(),
            INIT_BALANCE - existential_deposit
        ));
        // Check total issuance after making transfer.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

#[test]
fn total_issuance_dust_removal() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let existential_deposit =
            <<Runtime as pallet_balances::Config>::ExistentialDeposit as Get<u128>>::get();
        // Check total issuance before making transfer.
        let total_issuance_before = Balances::total_issuance();

        // Make AllowDeath transfer.
        assert_ok!(Balances::transfer(
            Some(account_id("Bob")).into(),
            account_id("Alice").into(),
            INIT_BALANCE - existential_deposit + 1,
        ));
        // Check total issuance after making transfer.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
        // Check that the account is dead.
        assert!(!frame_system::Account::<Runtime>::contains_key(
            &account_id("Bob")
        ));
    })
}

#[test]
fn total_issuance_transaction_payment_validate() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let call = pallet_balances::Call::transfer {
            dest: account_id("Alice").into(),
            value: 100,
        }
        .into();

        let normal = DispatchInfo {
            weight: 10,
            class: DispatchClass::Normal,
            pays_fee: Pays::Yes,
        };

        assert_ok!(
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0).validate(
                &account_id("Bob"),
                &call,
                &normal,
                10
            )
        );
    })
}
