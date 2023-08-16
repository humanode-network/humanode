//! Tests to verify the fee prices.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use ethereum::EIP1559Transaction;

use super::*;
use crate::dev_utils::*;
use crate::opaque::SessionKeys;

const INIT_BALANCE: Balance = 10u128.pow(18 + 6);
const ONE_BALANCE_UNIT: Balance = 10u128.pow(18);

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
fn new_test_ext_with() -> sp_io::TestExternalities {
    let authorities = vec![authority_keys("Alice")];
    let bootnodes = vec![account_id("Alice")];
    let endowed_accounts = vec![account_id("Alice"), account_id("Bob")];
    let pot_accounts = vec![FeesPot::account_id()];
    // Build test genesis.
    let config = GenesisConfig {
        balances: BalancesConfig {
            balances: {
                endowed_accounts
                    .iter()
                    .cloned()
                    .chain(pot_accounts.into_iter())
                    .map(|k| (k, INIT_BALANCE))
                    .chain(
                        [(
                            TreasuryPot::account_id(),
                            10 * INIT_BALANCE
                        ),
                        (
                            TokenClaimsPot::account_id(),
                            <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
                        ),
                        (
                            NativeToEvmSwapBridgePot::account_id(),
                            <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
                        )]
                        .into_iter(),
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
                let evm_pot_accounts =
                    vec![(
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
                    )];

                evm_pot_accounts.into_iter().collect()
            },
        },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    let mut ext = sp_io::TestExternalities::new(storage);

    // Provide the keystore.
    ext.register_extension(sp_keystore::KeystoreExt(keystore().into()));

    ext
}

/// Crate a new keystore and populate it with some keys to use in tests.
fn keystore() -> sp_keystore::testing::KeyStore {
    use sp_keystore::SyncCryptoStore;

    let store = sp_keystore::testing::KeyStore::new();
    store
        .sr25519_generate_new(crypto::KEY_TYPE, Some("//Alice"))
        .unwrap();
    store
}

#[allow(clippy::integer_arithmetic)]
fn assert_fee(call: RuntimeCall, len: u32, expected_fee: Balance, epsilon: Balance) {
    let dispath_info = TransactionPayment::query_call_info(call, len);
    let effective_fee = dispath_info.partial_fee;

    let lower_threshold = expected_fee - epsilon;
    let upper_threshold = expected_fee + epsilon;

    assert!(
        effective_fee <= upper_threshold,
        "{effective_fee} is not within {epsilon} above {expected_fee} ({effective_fee} > {upper_threshold})"
    );
    assert!(
        effective_fee >= lower_threshold,
        "{effective_fee} is not within {epsilon} below {expected_fee} ({effective_fee} < {lower_threshold})"
    );
}

/// The testing cryptography to match the real one we use for the accounts.
/// We use it to simulate the signatures in the test to estimate the tx size.
pub mod crypto {
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        KeyTypeId, MultiSignature, MultiSigner,
    };

    pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"test");
    app_crypto!(sr25519, KEY_TYPE);

    pub struct TestAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

fn switch_block() {
    use frame_support::traits::{OnFinalize, OnInitialize};

    if System::block_number() != 0 {
        AllPalletsWithSystem::on_finalize(System::block_number());
    }
    System::set_block_number(System::block_number() + 1);
    AllPalletsWithSystem::on_initialize(System::block_number());
}

/// A test that validates that a simple balance transfer with a keep alive costs 0.1 HMND.
#[test]
fn simple_balances_transfer_keep_alive() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Prepare a sample call to transfer 1 HMND.
        let call = RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
            dest: account_id("Bob").into(),
            value: ONE_BALANCE_UNIT,
        });

        // Compute the length of the extrinsic constaining this call.
        let (call, len) = {
            let sign_by = account_public("Alice");
            let signed_by_id = account_id("Alice");
            let (call, signature) = utils::create_transaction::<Runtime, crypto::TestAuthId>(
                call,
                sign_by,
                signed_by_id,
                0,
            )
            .unwrap();

            let extrinsic = {
                use sp_runtime::traits::Extrinsic;
                crate::UncheckedExtrinsic::new(call.clone(), Some(signature)).unwrap()
            };

            let encoded = extrinsic.encode();

            let len = encoded.len().try_into().unwrap();

            (call, len)
        };

        // The expected fee that we aim to target: 0.1 HMND.
        let expected_fee = ONE_BALANCE_UNIT / 10;

        // The tolerance within which the actual fee is allowed to be around the expected fee.
        let epsilon = expected_fee / 200;

        assert_fee(call, len, expected_fee, epsilon);
    })
}

/// A test that validates that a simple EVM balance transfer with a keep alive costs 0.2 HMND.
/// Computes the fee via [`TransactionPayment::query_call_info`].
#[test]
fn simple_evm_transaction_via_query_call_info() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        switch_block();
        Timestamp::set(RuntimeOrigin::none(), 1000).unwrap();
        switch_block();

        // Prepare a sample call to transfer 1 HMND.
        let max_fee_per_gas = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price().0;
        assert_eq!(max_fee_per_gas, constants::evm_fees::FEE_PER_GAS.into());

        let to = H160(hex_literal::hex!(
            "0000000000000000000000000000000000000000"
        ));

        let call = RuntimeCall::Ethereum(pallet_ethereum::Call::transact {
            transaction: EthereumTransaction::EIP1559(EIP1559Transaction {
                chain_id: <Runtime as pallet_evm::Config>::ChainId::get(),
                nonce: 0.into(),
                max_priority_fee_per_gas: 0.into(),
                max_fee_per_gas,
                gas_limit: 21000.into(), // simple transfer
                action: ethereum::TransactionAction::Call(to),
                value: U256::from(ONE_BALANCE_UNIT),
                input: Default::default(),
                access_list: Default::default(),
                odd_y_parity: false,
                r: Default::default(),
                s: Default::default(),
            }),
        });

        // The expected fee that we aim to target: 0.2 HMND.
        let expected_fee = ONE_BALANCE_UNIT / 5;

        // The tolerance within which the actual fee is allowed to be around the expected fee.
        let epsilon = expected_fee / 10;

        assert_fee(call, 0, expected_fee, epsilon);
    })
}
