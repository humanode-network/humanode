//! Tests to verify currency swap related basic operations.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use frame_support::{assert_ok, once_cell::sync::Lazy, traits::Currency};
use precompile_utils::{EvmDataWriter, LogsBuilder};
use sp_core::H160;

use super::*;
use crate::dev_utils::*;
use crate::opaque::SessionKeys;

pub(crate) static PRECOMPILE_ADDRESS: Lazy<H160> = Lazy::new(|| H160::from_low_u64_be(0x900));
pub(crate) static GAS_PRICE: Lazy<U256> =
    Lazy::new(|| <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price().0);

const INIT_BALANCE: Balance = 10u128.pow(18 + 6);

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
fn new_test_ext_with() -> sp_io::TestExternalities {
    let authorities = [authority_keys("Alice")];
    let bootnodes = vec![account_id("Alice")];

    let endowed_accounts = [account_id("Alice"), account_id("Bob")];
    let pot_accounts = vec![FeesPot::account_id()];

    let evm_endowed_accounts = vec![evm_account_id("EvmAlice"), evm_account_id("EvmBob")];
    // Build test genesis.
    let config = GenesisConfig {
        balances: BalancesConfig {
            balances: {
                endowed_accounts
                    .iter()
                    .cloned()
                    .chain(pot_accounts)
                    .map(|k| (k, INIT_BALANCE))
                    .chain(
                        [(
                            TreasuryPot::account_id(),
                            10 * INIT_BALANCE,
                        ),
                        (
                            TokenClaimsPot::account_id(),
                            <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
                        ),
                        (
                            NativeToEvmSwapBridgePot::account_id(),
                            <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
                        )]
                    )
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
                let init_genesis_account = fp_evm::GenesisAccount {
                    balance: INIT_BALANCE.into(),
                    code: Default::default(),
                    nonce: Default::default(),
                    storage: Default::default(),
                };

                evm_endowed_accounts
                    .into_iter()
                    .map(|k| (k, init_genesis_account.clone()))
                    .chain([(
                        EvmToNativeSwapBridgePot::account_id(),
                        fp_evm::GenesisAccount {
                            balance: <EvmBalances as frame_support::traits::Currency<
                                EvmAccountId,
                            >>::minimum_balance()
                            .into(),
                            code: Default::default(),
                            nonce: Default::default(),
                            storage: Default::default(),
                        },
                    )])
                    .collect()
            },
        },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}

/// This test verifies that bridges initialization has been applied at genesis.
#[test]
fn currencies_are_balanced() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        assert_eq!(
            BalancedCurrencySwapBridgesInitializer::last_initializer_version(),
            pallet_balanced_currency_swap_bridges_initializer::CURRENT_BRIDGES_INITIALIZER_VERSION
        );
        assert!(BalancedCurrencySwapBridgesInitializer::is_balanced().unwrap());
    })
}

/// This test verifies that swap native call works in the happy path.
#[test]
fn currency_swap_native_call_works() {
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
        assert!(BalancedCurrencySwapBridgesInitializer::is_balanced().unwrap());
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
