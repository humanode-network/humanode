//! Tests to verify token claims and vesting logic.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use eip712_common_test_utils::{ecdsa_pair, ecdsa_sign, ethereum_address_from_seed, U256};
use fp_self_contained::{CheckedExtrinsic, CheckedSignature};
use frame_support::{
    assert_noop, assert_ok, assert_storage_noop,
    dispatch::{DispatchClass, DispatchInfo, Pays},
    pallet_prelude::InvalidTransaction,
    traits::{OnFinalize, OnInitialize},
    weights::Weight,
};
use primitives_ethereum::EcdsaSignature;
use sp_runtime::traits::Applyable;
use vesting_schedule_linear::LinearSchedule;

use super::*;
use crate::{
    dev_utils::{account_id, authority_keys},
    eth_sig::genesis_verifying_contract,
    opaque::SessionKeys,
    token_claims::types::ClaimInfo,
};

const INIT_BALANCE: u128 = 10u128.pow(18 + 6);
const VESTING_BALANCE: u128 = 1000;

const START_TIMESTAMP: u64 = 1000;
const CLIFF: u64 = 1000;
const VESTING_DURATION: u64 = 3000;

fn set_timestamp(inc: UnixMilliseconds) {
    Timestamp::set(RuntimeOrigin::none(), inc).unwrap();
}

fn switch_block() {
    if System::block_number() != 0 {
        AllPalletsWithSystem::on_finalize(System::block_number());
    }
    System::set_block_number(System::block_number() + 1);
    AllPalletsWithSystem::on_initialize(System::block_number());
}

fn sign_sample_token_claim(
    seed: &[u8],
    account_id: AccountId,
) -> (EthereumAddress, EcdsaSignature) {
    let chain_id: [u8; 32] = U256::from(EthereumChainId::get()).into();
    let verifying_contract = genesis_verifying_contract();
    let domain = eip712_common::Domain {
        name: "Humanode Token Claim",
        version: "1",
        chain_id: &chain_id,
        verifying_contract: &verifying_contract,
    };

    let pair = ecdsa_pair(seed);
    let msg_hash = eip712_token_claim::make_message_hash(domain, account_id.as_ref());
    (
        ethereum_address_from_seed(seed),
        ecdsa_sign(&pair, &msg_hash),
    )
}

// We can avoid using signed extrinsic in assumption that it's already checked. So, we can just operate
// `CheckedExtrinsic`, `DispatchInfo` and go directly to checking the Extra using the Applyable trait
// (both apply and validate).
fn prepare_applyable_data(
    call: RuntimeCall,
    account_id: AccountId,
) -> (
    CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra, H160>,
    DispatchInfo,
    usize,
) {
    let extra = (
        frame_system::CheckSpecVersion::<Runtime>::new(),
        frame_system::CheckTxVersion::<Runtime>::new(),
        frame_system::CheckGenesis::<Runtime>::new(),
        frame_system::CheckEra::<Runtime>::from(sp_runtime::generic::Era::Immortal),
        frame_system::CheckNonce::<Runtime>::from(0),
        frame_system::CheckWeight::<Runtime>::new(),
        pallet_bioauth::CheckBioauthTx::<Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
        pallet_token_claims::CheckTokenClaim::<Runtime>::new(),
    );

    let normal_dispatch_info = DispatchInfo {
        weight: Weight::from_parts(100, 0),
        class: DispatchClass::Normal,
        pays_fee: Pays::No,
    };
    let len = 0;

    let checked_extrinsic = CheckedExtrinsic {
        signed: CheckedSignature::Signed(account_id, extra),
        function: call,
    };

    (checked_extrinsic, normal_dispatch_info, len)
}

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
fn new_test_ext() -> sp_io::TestExternalities {
    let authorities = [authority_keys("Alice")];
    let bootnodes = vec![account_id("Alice")];

    let endowed_accounts = [account_id("Alice"), account_id("Bob")];
    let pot_accounts = vec![TreasuryPot::account_id(), FeesPot::account_id()];
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
                            TokenClaimsPot::account_id(),
                            2 * VESTING_BALANCE + <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
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
        token_claims: TokenClaimsConfig {
            claims: vec![
                (
                    ethereum_address_from_seed(b"Dubai"),
                    ClaimInfo {
                        balance: VESTING_BALANCE,
                        vesting: vec![].try_into().unwrap(),
                    },
                ),
                (
                    ethereum_address_from_seed(b"Batumi"),
                    ClaimInfo {
                        balance: VESTING_BALANCE,
                        vesting: vec![LinearSchedule {
                            balance: VESTING_BALANCE,
                            cliff: CLIFF,
                            vesting: VESTING_DURATION,
                        }]
                        .try_into()
                        .unwrap(),
                    },
                ),
            ],
            total_claimable: Some(2 * VESTING_BALANCE),
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
        ethereum_chain_id: EthereumChainIdConfig { chain_id: 1 },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}

fn assert_genesis_json(token_claims: &str, token_claim_pot_balance: u128) {
    let json_input = prepare_genesis_json(token_claims, token_claim_pot_balance);
    let config: GenesisConfig = serde_json::from_str(json_input.as_str()).unwrap();
    let _ = config.build_storage();
}

fn assert_applyable_validate_all_transaction_sources(
    checked_extrinsic: &CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra, H160>,
    normal_dispatch_info: &DispatchInfo,
    len: usize,
) {
    assert_storage_noop!(assert_ok!(Applyable::validate::<Runtime>(
        checked_extrinsic,
        sp_runtime::transaction_validity::TransactionSource::Local,
        normal_dispatch_info,
        len,
    )));
    assert_storage_noop!(assert_ok!(Applyable::validate::<Runtime>(
        checked_extrinsic,
        sp_runtime::transaction_validity::TransactionSource::InBlock,
        normal_dispatch_info,
        len,
    )));
    assert_storage_noop!(assert_ok!(Applyable::validate::<Runtime>(
        checked_extrinsic,
        sp_runtime::transaction_validity::TransactionSource::External,
        normal_dispatch_info,
        len,
    )));
}

fn prepare_genesis_json(token_claims: &str, token_claim_pot_balance: u128) -> String {
    format!(
        r#"{{
        "system": {{
            "code": ""
        }},
        "bootnodes": {{
            "bootnodes": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"]
        }},
        "bioauth": {{
            "robonodePublicKey": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            "consumedAuthTicketNonces": [],
            "activeAuthentications": []
        }},
        "babe": {{
            "authorities": [],
            "epochConfig": {{
                "c": [1, 4],
                "allowed_slots": "PrimaryAndSecondaryPlainSlots"
            }}
        }},
        "balances": {{
            "balances": [
                [
                    "5EYCAe5h8DABNonHVCji5trNkxqKaz1WcvryauRMm4zYYDdQ",
                    500
                ],
                [
                    "5EYCAe5h8DABNogda2AhGjVZCcYAxcoVhSTMZXwWiQhVx9sY",
                    500
                ],
                [
                    "5EYCAe5h8D3eoqQjYNXVzehEzFAnY7cFnhV8ahjqgo5VxmeP",
                    500
                ],
                [
                    "5EYCAe5h8DABNonG7tbqC8bjDUw9jM1ewHJWssszZYbjkH2e",
                    {token_claim_pot_balance}
                ]
            ]
        }},
        "treasuryPot": {{
            "initialState": "Initialized"
        }},
        "feesPot": {{
            "initialState": "Initialized"
        }},
        "tokenClaimsPot": {{
            "initialState": "Initialized"
        }},
        "transactionPayment": {{
            "multiplier": "1000000000000000000"
        }},
        "session": {{
            "keys": [
                [
                    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                    {{
                        "babe": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                        "grandpa": "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu",
                        "im_online": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                    }}
                ]
            ]
        }},
        "chainProperties": {{
            "ss58Prefix": 1
        }},
        "ethereumChainId": {{
            "chainId": 1
        }},
        "sudo": {{
            "key": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        }},
        "grandpa": {{
            "authorities": []
        }},
        "ethereum": {{}},
        "evm": {{
            "accounts": {{
                "0x6d6f646c686d63732f656e310000000000000000": {{
                    "nonce": "0x0",
                    "balance": "0xd3c21bcecceda10001f4",
                    "storage": {{}},
                    "code": []
                }}
            }}
        }},
        "imOnline": {{
            "keys": []
        }},
        "evmAccountsMapping": {{
            "mappings": []
        }},
        "tokenClaims": {token_claims},
        "nativeToEvmSwapBridgePot": {{
            "initialState": "Initialized"
        }},
        "evmToNativeSwapBridgePot": {{
            "initialState": "Initialized"
        }},
        "balancedCurrencySwapBridgesInitializer": null,
        "dummyPrecompilesCode": {{}}
    }}"#
    )
}

/// This test verifies that `GenesisConfig` with claims is parsed in happy path.
#[test]
fn genesis_claims_works() {
    let token_claims = r#"
    {
        "claims": [
            [
                "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
                {
                    "balance": 1000000000000000000000000,
                    "vesting": [{"balance":1000000000000000000000000,"cliff":10,"vesting":10}]
                }
            ]
        ],
        "totalClaimable": 1000000000000000000000000
    }"#;
    assert_genesis_json(token_claims, 1000000000000000000000500);
}

/// This test verifies that `GenesisConfig` with claims fails due to invalid pot balance.
#[test]
#[should_panic = "invalid balance in the token claims pot account: got 500, expected 1000000000000000000000500"]
fn genesis_claims_invalid_pot_balance() {
    let token_claims = r#"
    {
        "claims": [
            [
                "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
                {
                    "balance": 1000000000000000000000000,
                    "vesting": [{"balance":1000000000000000000000000,"cliff":10,"vesting":10}]
                }
            ]
        ],
        "totalClaimable": 1000000000000000000000000
    }"#;
    assert_genesis_json(token_claims, 500);
}

/// This test verifies that `GenesisConfig` with claims fails due to invalid vesting initialization with null.
#[test]
#[should_panic = "invalid type: null, expected a sequence"]
fn genesis_claims_invalid_vesting_initialization_with_null() {
    let token_claims = r#"
    {
        "claims": [
            [
                "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
                {
                    "balance": 1000000000000000000000000,
                    "vesting": null
                }
            ]
        ],
        "totalClaimable": 1000000000000000000000000
    }"#;
    assert_genesis_json(token_claims, 1000000000000000000000500);
}

/// This test verifies that claiming without vesting works (direct runtime call) in the happy path.
#[test]
fn direct_claiming_without_vesting_works() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = sign_sample_token_claim(b"Dubai", account_id("Alice"));

        let total_issuance_before = Balances::total_issuance();

        // Test preconditions.
        assert!(TokenClaims::claims(ethereum_address).is_some());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );

        // Invoke the claim call.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Ensure the claim is gone from the state after the extrinsic is processed.
        assert!(TokenClaims::claims(ethereum_address).is_none());

        // Ensure the balance of the target account is properly adjusted.
        assert_eq!(
            Balances::free_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );

        // Ensure that the balance is not locked.
        //
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that claiming with vesting (direct runtime call) works in the happy path.
#[test]
fn direct_claiming_with_vesting_works() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = sign_sample_token_claim(b"Batumi", account_id("Alice"));

        let total_issuance_before = Balances::total_issuance();

        // Test preconditions.
        assert!(TokenClaims::claims(ethereum_address).is_some());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );
        assert!(Vesting::locks(account_id("Alice")).is_none());

        // Invoke the claim call.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Ensure the claim is gone from the state after the extrinsic is processed.
        assert!(TokenClaims::claims(ethereum_address).is_none());

        // Ensure the balance of the target account is properly adjusted.
        assert_eq!(
            Balances::free_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );

        // Ensure that the vesting balance is locked.
        //
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is equal to vesting balance > existential deposit.
        //
        // As a result, usable balance contains existential deposit.
        assert_eq!(Balances::usable_balance(account_id("Alice")), INIT_BALANCE);

        // Ensure that the vesting is armed for the given account and matches the parameters.
        assert_eq!(
            Vesting::locks(account_id("Alice")),
            Some(
                vec![LinearSchedule {
                    balance: VESTING_BALANCE,
                    cliff: CLIFF,
                    vesting: VESTING_DURATION,
                }]
                .try_into()
                .unwrap()
            )
        );

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that unlocking full balance (direct runtime call) works in the happy path.
#[test]
fn direct_unlock_full_balance_works() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = sign_sample_token_claim(b"Batumi", account_id("Alice"));

        let total_issuance_before = Balances::total_issuance();

        // Invoke the claim call for future unlocking.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Run blocks with setting proper timestamp to make full unlocking.
        set_timestamp(START_TIMESTAMP + CLIFF + VESTING_DURATION);
        switch_block();

        // Invoke the unlock call.
        assert_ok!(Vesting::unlock(Some(account_id("Alice")).into()));

        // Ensure funds are unlocked.
        //
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );

        // Ensure the vesting is gone from the state.
        assert!(Vesting::locks(account_id("Alice")).is_none());

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that unlocking partial balance works (direct runtime call) in the happy path.
#[test]
fn direct_unlock_partial_balance_works() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // 2/3 from VESTING_DURATION.
        const PARTIAL_DURATION: u64 = 2000;
        const PARTIAL_VESTING_TIMESTAMP: u64 = START_TIMESTAMP + CLIFF + PARTIAL_DURATION;
        // 2/3 from VESTING_BALANCE rounded up.
        const EXPECTED_PARTIAL_UNLOCKED_FUNDS: u128 = 667;

        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = sign_sample_token_claim(b"Batumi", account_id("Alice"));

        let total_issuance_before = Balances::total_issuance();

        // Invoke the claim call for future unlocking.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Run blocks with setting proper timestamp to make full unlocking.
        set_timestamp(PARTIAL_VESTING_TIMESTAMP);
        switch_block();

        // Invoke the unlock call.
        assert_ok!(Vesting::unlock(Some(account_id("Alice")).into()));

        let unlocked_balance = VESTING_BALANCE - System::account(account_id("Alice")).data.frozen;

        // Ensure funds are partially unlocked and rounding works as expected.
        assert_eq!(unlocked_balance, EXPECTED_PARTIAL_UNLOCKED_FUNDS);

        // Ensure the vesting still exists.
        assert!(Vesting::locks(account_id("Alice")).is_some());

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that direct claiming fails if `ethereum_address`
/// doesn't correspond to submitted `ethereum_signature`.
#[test]
fn direct_claiming_fails_when_eth_signature_invalid() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, _) = sign_sample_token_claim(b"Dubai", account_id("Alice"));

        let total_issuance_before = Balances::total_issuance();

        // Test preconditions.
        assert!(TokenClaims::claims(ethereum_address).is_some());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );

        // Invoke the claim call.
        assert_noop!(
            TokenClaims::claim(
                Some(account_id("Alice")).into(),
                ethereum_address,
                EcdsaSignature::default()
            ),
            pallet_token_claims::Error::<Runtime>::InvalidSignature
        );

        // Ensure claims related state hasn't been changed.
        assert!(TokenClaims::claims(ethereum_address).is_some());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that direct claiming fails in case not existing claim.
#[test]
fn direct_claiming_fails_when_no_claim() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) =
            sign_sample_token_claim(b"Invalid", account_id("Alice"));

        let total_issuance_before = Balances::total_issuance();

        // Test preconditions.
        assert!(TokenClaims::claims(ethereum_address).is_none());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );

        // Invoke the claim call.
        assert_noop!(
            TokenClaims::claim(
                Some(account_id("Alice")).into(),
                ethereum_address,
                signature
            ),
            pallet_token_claims::Error::<Runtime>::NoClaim
        );

        // Ensure claims related state hasn't been changed.
        assert!(TokenClaims::claims(ethereum_address).is_none());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that direct unlock fails in case not existing vesting.
#[test]
fn direct_unlock_fails_when_no_vesting() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = sign_sample_token_claim(b"Batumi", account_id("Alice"));

        let total_issuance_before = Balances::total_issuance();

        // Invoke the claim call for future unlocking.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Run blocks with setting proper timestamp to make full unlocking.
        set_timestamp(START_TIMESTAMP + CLIFF + VESTING_DURATION);
        switch_block();

        // Invoke the unlock call.
        assert_noop!(
            Vesting::unlock(Some(account_id("Unknown")).into()),
            pallet_vesting::Error::<Runtime>::NoVesting
        );

        // Ensure funds are still locked.
        //
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is equal to vesting balance > existential deposit.
        //
        // As a result, usable balance contains existential deposit.
        assert_eq!(Balances::usable_balance(account_id("Alice")), INIT_BALANCE);

        // Ensure the vesting isn't gone from the state.
        assert!(Vesting::locks(account_id("Alice")).is_some());

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that repeated claim fails for the account that already has vesting
/// after adding new claim info by sudo call.
#[test]
fn claims_fails_when_vesting_already_engaged() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // 2/3 from VESTING_DURATION.
        const PARTIAL_DURATION: u64 = 2000;
        const PARTIAL_VESTING_TIMESTAMP: u64 = START_TIMESTAMP + CLIFF + PARTIAL_DURATION;
        // 2/3 from VESTING_BALANCE rounded up.
        const EXPECTED_PARTIAL_UNLOCKED_FUNDS: u128 = 667;

        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = sign_sample_token_claim(b"Batumi", account_id("Alice"));

        let total_issuance_before = Balances::total_issuance();

        // Invoke the claim call for future unlocking.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Run blocks with setting proper timestamp to make partial unlocking.
        set_timestamp(PARTIAL_VESTING_TIMESTAMP);
        switch_block();

        // Invoke the unlock call.
        assert_ok!(Vesting::unlock(Some(account_id("Alice")).into()));

        let unlocked_balance = VESTING_BALANCE - System::account(account_id("Alice")).data.frozen;

        // Ensure funds are partially unlocked and rounding works as expected.
        assert_eq!(unlocked_balance, EXPECTED_PARTIAL_UNLOCKED_FUNDS);

        // Ensure the vesting still exists.
        assert!(Vesting::locks(account_id("Alice")).is_some());

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);

        // Prepare new claim info data.
        let new_claim_info = ClaimInfo {
            balance: 10000,
            vesting: vec![].try_into().unwrap(),
        };

        // Invoke the add_claim call by sudo account.
        assert_ok!(TokenClaims::add_claim(
            RuntimeOrigin::root(),
            ethereum_address,
            new_claim_info,
            TreasuryPot::account_id()
        ));

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);

        // Invoke the claim call.
        assert_noop!(
            TokenClaims::claim(
                Some(account_id("Alice")).into(),
                ethereum_address,
                signature
            ),
            pallet_vesting::Error::<Runtime>::VestingAlreadyEngaged
        );
    })
}

/// This test verifies that claiming without vesting (dispatch call) works in the happy path.
#[test]
fn dispatch_claiming_without_vesting_works() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, ethereum_signature) =
            sign_sample_token_claim(b"Dubai", account_id("Alice"));

        // Prepare token claim data that are used to validate and apply `CheckedExtrinsic`.
        let (checked_extrinsic, normal_dispatch_info, len) = prepare_applyable_data(
            RuntimeCall::TokenClaims(pallet_token_claims::Call::claim {
                ethereum_address,
                ethereum_signature,
            }),
            account_id("Alice"),
        );

        let total_issuance_before = Balances::total_issuance();

        // Test preconditions.
        assert!(TokenClaims::claims(ethereum_address).is_some());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );

        // Validate already checked extrinsic with all possible transaction sources.
        assert_applyable_validate_all_transaction_sources(
            &checked_extrinsic,
            &normal_dispatch_info,
            len,
        );
        // Apply already checked extrinsic.
        assert_ok!(Applyable::apply::<Runtime>(
            checked_extrinsic,
            &normal_dispatch_info,
            len
        ));

        // Ensure the claim is gone from the state after the extrinsic is processed.
        assert!(TokenClaims::claims(ethereum_address).is_none());

        // Ensure the balance of the target account is properly adjusted.
        assert_eq!(
            Balances::free_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );

        // Ensure that the balance is not locked.
        //
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that claiming with vesting (dispatch call) works in the happy path.
#[test]
fn dispatch_claiming_with_vesting_works() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, ethereum_signature) =
            sign_sample_token_claim(b"Batumi", account_id("Alice"));

        // Prepare token claim data that are used to validate and apply `CheckedExtrinsic`.
        let (checked_extrinsic, normal_dispatch_info, len) = prepare_applyable_data(
            RuntimeCall::TokenClaims(pallet_token_claims::Call::claim {
                ethereum_address,
                ethereum_signature,
            }),
            account_id("Alice"),
        );

        let total_issuance_before = Balances::total_issuance();

        // Test preconditions.
        assert!(TokenClaims::claims(ethereum_address).is_some());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );
        assert!(Vesting::locks(account_id("Alice")).is_none());

        // Validate already checked extrinsic with all possible transaction sources.
        assert_applyable_validate_all_transaction_sources(
            &checked_extrinsic,
            &normal_dispatch_info,
            len,
        );
        // Apply already checked extrinsic.
        assert_ok!(Applyable::apply::<Runtime>(
            checked_extrinsic,
            &normal_dispatch_info,
            len
        ));

        // Ensure the claim is gone from the state after the extrinsic is processed.
        assert!(TokenClaims::claims(ethereum_address).is_none());

        // Ensure the balance of the target account is properly adjusted.
        assert_eq!(
            Balances::free_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );

        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is equal to vesting balance > existential deposit.
        //
        // As a result, usable balance contains existential deposit.
        assert_eq!(Balances::usable_balance(account_id("Alice")), INIT_BALANCE);

        // Ensure that the vesting is armed for the given account and matches the parameters.
        assert_eq!(
            Vesting::locks(account_id("Alice")),
            Some(
                vec![LinearSchedule {
                    balance: VESTING_BALANCE,
                    cliff: CLIFF,
                    vesting: VESTING_DURATION,
                }]
                .try_into()
                .unwrap()
            )
        );

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that unlocking full balance (dispatch call) works in the happy path.
#[test]
fn dispatch_unlock_full_balance_works() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, ethereum_signature) =
            sign_sample_token_claim(b"Batumi", account_id("Alice"));

        // Invoke the direct runtime claim call for future unlocking.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            ethereum_signature
        ));

        // Run blocks with setting proper timestamp to make full unlocking.
        set_timestamp(START_TIMESTAMP + CLIFF + VESTING_DURATION);
        switch_block();

        // Prepare unlock data that are used to validate and apply `CheckedExtrinsic`.
        let (checked_extrinsic, normal_dispatch_info, len) = prepare_applyable_data(
            RuntimeCall::Vesting(pallet_vesting::Call::unlock {}),
            account_id("Alice"),
        );

        // Test preconditions.
        assert_eq!(
            Balances::free_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is equal to vesting balance < existential deposit.
        //
        // As a result, usable balance contains existential deposit.
        assert_eq!(Balances::usable_balance(account_id("Alice")), INIT_BALANCE);
        assert!(Vesting::locks(account_id("Alice")).is_some());

        let total_issuance_before = Balances::total_issuance();

        // Validate already checked extrinsic with all possible transaction sources.
        assert_applyable_validate_all_transaction_sources(
            &checked_extrinsic,
            &normal_dispatch_info,
            len,
        );
        // Apply already checked extrinsic.
        assert_ok!(Applyable::apply::<Runtime>(
            checked_extrinsic,
            &normal_dispatch_info,
            len
        ));

        // Ensure funds are unlocked.
        //
        // Alice account can't be reaped as the account takes part in consensus (bootnode).
        // Frozen balance is 0 < existential deposit.
        //
        // As a result, usable balance doesn't contain existential deposit.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
                - <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance()
        );

        // Ensure the vesting is gone from the state.
        assert!(Vesting::locks(account_id("Alice")).is_none());

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that unlocking partial balance (dispatch call) works in the happy path.
#[test]
fn dispatch_unlock_partial_balance_works() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // 2/3 from VESTING_DURATION.
        const PARTIAL_DURATION: u64 = 2000;
        const PARTIAL_VESTING_TIMESTAMP: u64 = START_TIMESTAMP + CLIFF + PARTIAL_DURATION;
        // 2/3 from VESTING_BALANCE rounded up.
        const EXPECTED_PARTIAL_UNLOCKED_FUNDS: u128 = 667;

        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, ethereum_signature) =
            sign_sample_token_claim(b"Batumi", account_id("Alice"));

        // Invoke the direct runtime claim call for future unlocking.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            ethereum_signature
        ));

        // Run blocks with setting proper timestamp to make full unlocking.
        set_timestamp(PARTIAL_VESTING_TIMESTAMP);
        switch_block();

        // Prepare unlock data that are used to validate and apply `CheckedExtrinsic`.
        let (checked_extrinsic, normal_dispatch_info, len) = prepare_applyable_data(
            RuntimeCall::Vesting(pallet_vesting::Call::unlock {}),
            account_id("Alice"),
        );

        // Test preconditions.
        assert_eq!(
            Balances::free_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );
        assert_eq!(Balances::usable_balance(account_id("Alice")), INIT_BALANCE);
        assert!(Vesting::locks(account_id("Alice")).is_some());

        let total_issuance_before = Balances::total_issuance();

        // Validate already checked extrinsic with all possible transaction sources.
        assert_applyable_validate_all_transaction_sources(
            &checked_extrinsic,
            &normal_dispatch_info,
            len,
        );
        // Apply already checked extrinsic.
        assert_ok!(Applyable::apply::<Runtime>(
            checked_extrinsic,
            &normal_dispatch_info,
            len
        ));

        let unlocked_balance = VESTING_BALANCE - System::account(account_id("Alice")).data.frozen;

        // Ensure funds are partially unlocked and rounding works as expected.
        assert_eq!(unlocked_balance, EXPECTED_PARTIAL_UNLOCKED_FUNDS);

        // Ensure the vesting still exists.
        assert!(Vesting::locks(account_id("Alice")).is_some());

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that dispatch claiming fails if `ethereum_address`
/// doesn't correspond to submitted `ethereum_signature`.
#[test]
fn dispatch_claiming_fails_when_eth_signature_invalid() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Prepare token claim data that are used to validate and apply `CheckedExtrinsic`.
        let (checked_extrinsic, normal_dispatch_info, len) = prepare_applyable_data(
            RuntimeCall::TokenClaims(pallet_token_claims::Call::claim {
                ethereum_address: EthereumAddress::default(),
                ethereum_signature: EcdsaSignature::default(),
            }),
            account_id("Alice"),
        );

        // Validate already checked extrinsic.
        assert_noop!(
            Applyable::validate::<Runtime>(
                &checked_extrinsic,
                sp_runtime::transaction_validity::TransactionSource::Local,
                &normal_dispatch_info,
                len,
            ),
            TransactionValidityError::Invalid(InvalidTransaction::BadProof)
        );
        // Apply already checked extrinsic.
        //
        // We don't use assert_noop as apply invokes pre_dispatch that uses fee.
        // As a result state is changed.
        assert_eq!(
            Applyable::apply::<Runtime>(checked_extrinsic, &normal_dispatch_info, len),
            Err(TransactionValidityError::Invalid(
                InvalidTransaction::BadProof
            ))
        );
    })
}

/// This test verifies that dispatch claiming fails in case not existing claim.
#[test]
fn dispatch_claiming_fails_when_no_claim() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, ethereum_signature) =
            sign_sample_token_claim(b"Invalid", account_id("Alice"));

        // Prepare token claim data that are used to validate and apply `CheckedExtrinsic`.
        let (checked_extrinsic, normal_dispatch_info, len) = prepare_applyable_data(
            RuntimeCall::TokenClaims(pallet_token_claims::Call::claim {
                ethereum_address,
                ethereum_signature,
            }),
            account_id("Alice"),
        );

        // Validate already checked extrinsic.
        assert_noop!(
            Applyable::validate::<Runtime>(
                &checked_extrinsic,
                sp_runtime::transaction_validity::TransactionSource::Local,
                &normal_dispatch_info,
                len,
            ),
            TransactionValidityError::Invalid(InvalidTransaction::Call)
        );
        // Apply already checked extrinsic.
        //
        // We don't use assert_noop as apply invokes pre_dispatch that uses fee.
        // As a result state is changed.
        assert_eq!(
            Applyable::apply::<Runtime>(checked_extrinsic, &normal_dispatch_info, len),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
        );
    })
}

/// This test verifies that claiming without vesting (dispatch call) works in the happy path with zero balance.
/// So, we verify that the call is free in term of transaction fee.
#[test]
fn dispatch_claiming_zero_balance_works() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, ethereum_signature) =
            sign_sample_token_claim(b"Dubai", account_id("Zero"));

        // Prepare token claim data that are used to validate and apply `CheckedExtrinsic`.
        let (checked_extrinsic, normal_dispatch_info, len) = prepare_applyable_data(
            RuntimeCall::TokenClaims(pallet_token_claims::Call::claim {
                ethereum_address,
                ethereum_signature,
            }),
            account_id("Zero"),
        );

        let total_issuance_before = Balances::total_issuance();

        // Test preconditions.
        assert!(TokenClaims::claims(ethereum_address).is_some());
        assert_eq!(Balances::free_balance(account_id("Zero")), 0);
        assert_eq!(Balances::usable_balance(account_id("Zero")), 0);

        // Validate already checked extrinsic with all possible transaction sources.
        assert_applyable_validate_all_transaction_sources(
            &checked_extrinsic,
            &normal_dispatch_info,
            len,
        );
        // Apply already checked extrinsic.
        assert_ok!(Applyable::apply::<Runtime>(
            checked_extrinsic,
            &normal_dispatch_info,
            len
        ));

        // Ensure the claim is gone from the state after the extrinsic is processed.
        assert!(TokenClaims::claims(ethereum_address).is_none());

        // Ensure the balance of the target account is properly adjusted.
        assert_eq!(Balances::free_balance(account_id("Zero")), VESTING_BALANCE);

        // Ensure that the balance is not locked.
        assert_eq!(
            Balances::usable_balance(account_id("Zero")),
            VESTING_BALANCE
        );

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}
