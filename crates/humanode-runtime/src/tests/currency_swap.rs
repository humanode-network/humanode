//! Tests to verify currency swap related basic operations.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use frame_support::{assert_ok, traits::Currency};

use super::*;
use crate::dev_utils::*;
use crate::opaque::SessionKeys;

const INIT_BALANCE: Balance = 10u128.pow(18 + 6);

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
fn new_test_ext_with() -> sp_io::TestExternalities {
    let authorities = vec![authority_keys("Alice")];
    let bootnodes = vec![account_id("Alice")];
    let endowed_accounts = vec![account_id("Alice"), account_id("Bob")];
    let evm_endowed_accounts = vec![evm_account_id("EvmAlice"), evm_account_id("EvmBob")];
    // Build test genesis.
    let config = GenesisConfig {
        balances: BalancesConfig {
            balances: {
                let pot_accounts = vec![
                    TreasuryPot::account_id(),
                    FeesPot::account_id(),
                    NativeToEvmSwapBridgePot::account_id(),
                ];
                endowed_accounts
                    .iter()
                    .cloned()
                    .chain(pot_accounts.into_iter())
                    .map(|k| (k, INIT_BALANCE))
                    .chain([(
                        TokenClaimsPot::account_id(),
                        <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
                    )])
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
        evm: EVMConfig {
            accounts: {
                let evm_pot_accounts = vec![EvmToNativeSwapBridgePot::account_id()];

                let init_genesis_account = fp_evm::GenesisAccount {
                    balance: INIT_BALANCE.into(),
                    code: Default::default(),
                    nonce: Default::default(),
                    storage: Default::default(),
                };

                evm_endowed_accounts
                    .into_iter()
                    .chain(evm_pot_accounts.into_iter())
                    .map(|k| (k, init_genesis_account.clone()))
                    .collect()
            },
        },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}

#[test]
fn currency_swap_native_call() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let alice_balance_before = Balances::total_balance(&account_id("Alice"));
        let native_to_evm_swap_bridge_pot_before =
            Balances::total_balance(&NativeToEvmSwapBridgePot::account_id());
        let alice_evm_balance_before = EvmBalances::total_balance(&evm_account_id("EvmAlice"));
        let evm_to_native_swap_bridge_pot_before =
            EvmBalances::total_balance(&EvmToNativeSwapBridgePot::account_id());
        let swap_balance: Balance = 1000;

        // Make swap.
        assert_ok!(CurrencySwap::swap(
            Some(account_id("Alice")).into(),
            evm_account_id("EvmAlice"),
            swap_balance
        ));

        // Assert state changes.
        assert_eq!(
            Balances::total_balance(&account_id("Alice")),
            alice_balance_before - swap_balance
        );
        assert_eq!(
            Balances::total_balance(&NativeToEvmSwapBridgePot::account_id()),
            native_to_evm_swap_bridge_pot_before + swap_balance
        );
        assert_eq!(
            EvmBalances::total_balance(&evm_account_id("EvmAlice")),
            alice_evm_balance_before + swap_balance
        );
        assert_eq!(
            EvmBalances::total_balance(&EvmToNativeSwapBridgePot::account_id()),
            evm_to_native_swap_bridge_pot_before - swap_balance
        );
    })
}
