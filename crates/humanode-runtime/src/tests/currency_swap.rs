//! Tests to verify currency swap related basic operations.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use std::collections::BTreeMap;

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
                    BalancesPot::account_id(),
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
                let evm_pot_accounts = vec![EvmBalancesPot::account_id()];

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
                    .collect::<BTreeMap<_, _>>()
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
        let balances_pot_before = Balances::total_balance(&BalancesPot::account_id());
        let alice_evm_balance_before = EvmBalances::total_balance(&evm_account_id("EvmAlice"));
        let evm_balances_pot_before = EvmBalances::total_balance(&EvmBalancesPot::account_id());
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
            Balances::total_balance(&BalancesPot::account_id()),
            balances_pot_before + swap_balance
        );
        assert_eq!(
            EvmBalances::total_balance(&evm_account_id("EvmAlice")),
            alice_evm_balance_before + swap_balance
        );
        assert_eq!(
            EvmBalances::total_balance(&EvmBalancesPot::account_id()),
            evm_balances_pot_before - swap_balance
        );
    })
}

#[test]
fn currency_swap_proxy_ethereum_execute() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Check fees pot balance before executing an ethereum transaction.
        let fees_pot_balance_before = Balances::total_balance(&FeesPot::account_id());

        let evm_bob_origin =
            pallet_ethereum::RawOrigin::EthereumTransaction(evm_account_id("EvmBob"));

        let gas_price = 20_000_000_000_u128;
        let gas_limit = 21000;

        // This test legacy data transaction obtained from
        // <https://github.com/rust-blockchain/ethereum/blob/0ffbe47d1da71841be274442a3050da9c895e10a/src/transaction.rs#L788>.
        let legacy_transaction = pallet_ethereum::Transaction::Legacy(ethereum::LegacyTransaction {
			nonce: 0.into(),
			gas_price: gas_price.into(),
			gas_limit: gas_limit.into(),
			action: ethereum::TransactionAction::Call(
				hex_literal::hex!("727fc6a68321b754475c668a6abfb6e9e71c169a").into(),
			),
			value: U256::from(10) * 1_000_000_000 * 1_000_000_000,
			input: hex_literal::hex!("a9059cbb000000000213ed0f886efd100b67c7e4ec0a85a7d20dc971600000000000000000000015af1d78b58c4000").into(),
			signature: ethereum::TransactionSignature::new(38, hex_literal::hex!("be67e0a07db67da8d446f76add590e54b6e92cb6b8f9835aeb67540579a27717").into(), hex_literal::hex!("2d690516512020171c1ec870f6ff45398cc8609250326be89915fb538e7bd718").into()).unwrap(),
		});

        // Execute an ethereum transaction.
        assert_ok!(Ethereum::transact(evm_bob_origin.into(), legacy_transaction));

        // Check fees pot balance after executing ethereum transaction.
        assert_eq!(Balances::total_balance(&FeesPot::account_id()), fees_pot_balance_before + (gas_price * gas_limit));
    })
}
