//! Tests to verify fixed supply logic.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use std::str::FromStr;

use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchClass, DispatchError, DispatchInfo, Pays},
    traits::{Currency, ExistenceRequirement},
    weights::Weight,
};
use sp_core::ByteArray;
use sp_runtime::traits::SignedExtension;

use super::*;
use crate::dev_utils::*;
use crate::opaque::SessionKeys;

const INIT_BALANCE: u128 = 10u128.pow(18 + 6);

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
                            TreasuryPot::account_id(), 10 * INIT_BALANCE
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

fn assert_total_issuance() {
    let native_to_evm_swap_bridge_pot =
        Balances::total_balance(&NativeToEvmSwapBridgePot::account_id());
    let evm_to_native_swap_bridge_pot =
        EvmBalances::total_balance(&EvmToNativeSwapBridgePot::account_id());

    let existential_deposit =
        <<Runtime as pallet_balances::Config>::ExistentialDeposit as Get<u128>>::get();
    let evm_existential_deposit =
        <<Runtime as pallet_evm_balances::Config>::ExistentialDeposit as Get<u128>>::get();

    let existential_deposit_balance = existential_deposit.max(evm_existential_deposit);

    let total_issuance = Balances::total_issuance();
    let evm_total_issuance = EvmBalances::total_issuance();

    assert_eq!(
        total_issuance - native_to_evm_swap_bridge_pot,
        evm_to_native_swap_bridge_pot - existential_deposit_balance
    );
    assert_eq!(
        evm_total_issuance - evm_to_native_swap_bridge_pot,
        native_to_evm_swap_bridge_pot - existential_deposit_balance
    );
}

#[test]
fn total_issuance_genesis() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        assert_total_issuance();
    })
}

#[test]
fn total_issuance_transaction_fee() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let existential_deposit =
            <<Runtime as pallet_balances::Config>::ExistentialDeposit as Get<u128>>::get();

        // Check total issuance before making transfer.
        let total_issuance_before = Balances::total_issuance();
        let evm_total_issuance_before = EvmBalances::total_issuance();

        // Make KeepAlive transfer.
        assert_ok!(Balances::transfer_keep_alive(
            Some(account_id("Bob")).into(),
            account_id("Alice").into(),
            INIT_BALANCE - existential_deposit
        ));
        // Check total issuance after making transfer.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
        assert_eq!(EvmBalances::total_issuance(), evm_total_issuance_before);
        assert_total_issuance();
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
        let evm_total_issuance_before = EvmBalances::total_issuance();

        // Make AllowDeath transfer.
        assert_ok!(Balances::transfer(
            Some(account_id("Bob")).into(),
            account_id("Alice").into(),
            INIT_BALANCE - existential_deposit + 1,
        ));
        // Check total issuance after making transfer.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
        assert_eq!(EvmBalances::total_issuance(), evm_total_issuance_before);
        // Check that the account is dead.
        assert!(!frame_system::Account::<Runtime>::contains_key(account_id(
            "Bob"
        )));
        assert_total_issuance();
    })
}

#[test]
fn total_issuance_transaction_payment_validate() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Prepare test data.
        let call = pallet_balances::Call::transfer {
            dest: account_id("Alice").into(),
            value: 100,
        }
        .into();
        let normal = DispatchInfo {
            weight: Weight::from_parts(10, 0),
            class: DispatchClass::Normal,
            pays_fee: Pays::Yes,
        };

        // Check total issuance before making transaction validate.
        let total_issuance_before = Balances::total_issuance();
        let evm_total_issuance_before = EvmBalances::total_issuance();

        assert_ok!(
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0).validate(
                &account_id("Bob"),
                &call,
                &normal,
                10
            )
        );

        // !!! WARNING !!!
        // Here the actual behaviour is that the total issuance does change after the tx validation.
        // However, in practice to don't care about it, since the resulting state of the tx
        // validation is dropped, and in the real execution, the unbalance is properly dealt with.

        // This assertion is set to check that the total balance is *not* equal to the previous one,
        // but not because it is the intended behaviour (on the contrrary! we'd rather have
        // the balance intact!), but to alert us if/when things change, and this balance becomes
        // intact after the calculation.
        // If you see this assertion start failing (while the *rest of the suite is ok*) - it might
        // mean that the liquidity drop at tx validation has been fixed.
        assert_ne!(Balances::total_issuance(), total_issuance_before);

        // Evm balances shouldn't be changed.
        assert_eq!(EvmBalances::total_issuance(), evm_total_issuance_before);
    })
}

#[test]
fn total_issuance_evm_withdraw() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let existential_deposit =
            <<Runtime as pallet_balances::Config>::ExistentialDeposit as Get<u128>>::get();

        // Check total issuance before making evm withdraw.
        let total_issuance_before = Balances::total_issuance();
        let evm_total_issuance_before = EvmBalances::total_issuance();

        // Calculate bob related evm truncated address.
        let bob_evm_truncated = H160::from_slice(&account_id("Bob").as_slice()[0..20]);

        // Send tokens to hashed_bob_evm to make withdraw from bob_evm.
        assert_ok!(EvmBalances::transfer(
            &evm_account_id("EvmAlice"),
            &bob_evm_truncated,
            INIT_BALANCE - existential_deposit - 1,
            ExistenceRequirement::KeepAlive
        ));

        // Invoke the function under test.
        assert_noop!(
            EVM::withdraw(Some(account_id("Bob")).into(), bob_evm_truncated, 1000),
            DispatchError::BadOrigin
        );

        // Check total issuance after making evm withdraw.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
        assert_eq!(EvmBalances::total_issuance(), evm_total_issuance_before);
        assert_total_issuance();
    })
}

#[test]
fn total_issuance_evm_call() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let existential_deposit =
            <<Runtime as pallet_balances::Config>::ExistentialDeposit as Get<u128>>::get();

        // Check total issuance before making evm call.
        let total_issuance_before = Balances::total_issuance();
        let evm_total_issuance_before = EvmBalances::total_issuance();

        // Calculate bob related evm truncated address.
        let bob_evm_truncated = H160::from_slice(&account_id("Bob").as_slice()[0..20]);

        // Send tokens to hashed_bob_evm to make call from bob_evm.
        assert_ok!(EvmBalances::transfer(
            &evm_account_id("EvmAlice"),
            &bob_evm_truncated,
            INIT_BALANCE - existential_deposit - 1,
            ExistenceRequirement::KeepAlive
        ));

        assert_noop!(
            EVM::call(
                Some(account_id("Bob")).into(),
                bob_evm_truncated,
                H160::from_str("1000000000000000000000000000000000000001").unwrap(),
                Vec::new(),
                U256::from(1_000),
                1000000,
                U256::from(2_000_000_000),
                Some(U256::from(1)),
                Some(U256::from(0)),
                Vec::new(),
            ),
            DispatchError::BadOrigin
        );

        // Check total issuance after making evm call.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
        assert_eq!(EvmBalances::total_issuance(), evm_total_issuance_before);
        assert_total_issuance();
    })
}

#[test]
fn total_issuance_evm_create() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let existential_deposit =
            <<Runtime as pallet_balances::Config>::ExistentialDeposit as Get<u128>>::get();

        // Check total issuance before making evm create.
        let total_issuance_before = Balances::total_issuance();
        let evm_total_issuance_before = EvmBalances::total_issuance();

        // Calculate bob related evm truncated address.
        let bob_evm_truncated = H160::from_slice(&account_id("Bob").as_slice()[0..20]);

        // Send tokens to hashed_bob_evm to make create from bob_evm.
        assert_ok!(EvmBalances::transfer(
            &evm_account_id("EvmAlice"),
            &bob_evm_truncated,
            INIT_BALANCE - existential_deposit - 1,
            ExistenceRequirement::KeepAlive
        ));

        assert_noop!(
            EVM::create(
                Some(account_id("Bob")).into(),
                bob_evm_truncated,
                Vec::new(),
                U256::from(1_000),
                1000000,
                U256::from(2_000_000_000),
                Some(U256::from(1)),
                Some(U256::from(0)),
                Vec::new(),
            ),
            DispatchError::BadOrigin
        );

        // Check total issuance after making evm call.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
        assert_eq!(EvmBalances::total_issuance(), evm_total_issuance_before);
        assert_total_issuance();
    })
}

#[test]
fn total_issuance_transaction_fee_ethereum_transact() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Check total issuance before executing an ethereum transaction.
        let total_issuance_before = Balances::total_issuance();
        let evm_total_issuance_before = EvmBalances::total_issuance();

        // Check fees pot balance before executing an ethereum transaction.
        let fees_pot_balance_before = Balances::total_balance(&FeesPot::account_id());

        let evm_bob_origin =
            pallet_ethereum::RawOrigin::EthereumTransaction(evm_account_id("EvmBob"));
        let gas_price: u128 = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price().0.try_into().unwrap();
        let gas_limit: u128 = <Runtime as pallet_evm::Config>::config().gas_transaction_call.into();

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

        // Check total issuance after executing ethereum transaction.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
        assert_eq!(EvmBalances::total_issuance(), evm_total_issuance_before);
        // Check fees pot balance after executing ethereum transaction.
        assert_eq!(Balances::total_balance(&FeesPot::account_id()), fees_pot_balance_before + (gas_price * gas_limit));
        assert_total_issuance();
    })
}

#[test]
fn total_issuance_dust_removal_evm_transfer() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let evm_existential_deposit =
            <<Runtime as pallet_evm_balances::Config>::ExistentialDeposit as Get<u128>>::get();
        let treasury_pot_before = Balances::total_balance(&TreasuryPot::account_id());

        // Check total issuance before executing an ethereum transaction.
        let total_issuance_before = Balances::total_issuance();
        let evm_total_issuance_before = EvmBalances::total_issuance();

        // Make AllowDeath transfer.
        assert_ok!(EvmBalances::transfer(
            &evm_account_id("EvmBob"),
            &evm_account_id("EvmAlice"),
            INIT_BALANCE - evm_existential_deposit + 1,
            ExistenceRequirement::AllowDeath
        ));

        // Check total issuance after making transfer.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
        assert_eq!(EvmBalances::total_issuance(), evm_total_issuance_before);
        // Check that the account is dead.
        assert!(!pallet_evm_system::Account::<Runtime>::contains_key(
            evm_account_id("EvmBob")
        ));
        // Check that treasury pot account balance has been updated properly.
        assert_eq!(
            Balances::total_balance(&TreasuryPot::account_id()),
            treasury_pot_before + evm_existential_deposit - 1
        );
        assert_total_issuance();
    })
}
