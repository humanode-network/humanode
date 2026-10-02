//! Tests to verify the custom Humanode precompiles are guarded against `DELEGATECALL`.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use frame_support::{once_cell::sync::Lazy, traits::fungible::Inspect};
use precompile_utils::{Address, EvmDataWriter};
use sp_core::H160;

use super::*;
use crate::dev_utils::*;
use crate::frontier_precompiles::{hash, precompiles_constants::*};
use crate::opaque::SessionKeys;

static GAS_PRICE: Lazy<U256> =
    Lazy::new(|| <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price().0);

const INIT_BALANCE: Balance = 10u128.pow(18 + 6);

/// The kind of call the forwarder contract issues to its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForwardKind {
    DelegateCall,
    CallCode,
    StaticCall,
}

/// All the forwarder kinds.
const FORWARD_KINDS: [ForwardKind; 3] = [
    ForwardKind::DelegateCall,
    ForwardKind::CallCode,
    ForwardKind::StaticCall,
];

/// The address at which the forwarder contract of the given kind is deployed at genesis.
fn forwarder_address(kind: ForwardKind) -> H160 {
    H160::from_low_u64_be(0xf0f0f0 + kind as u64)
}

/// A minimal contract that forwards the provided input to the target via the given call kind
/// and bubbles up the result (return data on success, revert data on failure).
///
/// Calldata layout: `abi.encode(target)` (32 bytes, address right-aligned) followed by the raw
/// input to forward.
fn forwarder_code(kind: ForwardKind) -> Vec<u8> {
    let mut code = vec![
        // size = CALLDATASIZE - 32
        0x60, 0x20, // PUSH1 0x20
        0x36, // CALLDATASIZE
        0x03, // SUB
        // CALLDATACOPY(dest = 0, offset = 32, size)
        0x80, // DUP1
        0x60, 0x20, // PUSH1 0x20
        0x60, 0x00, // PUSH1 0x00
        0x37, // CALLDATACOPY
        // Call arguments, pushed in reverse: ret_size = 0, ret_offset = 0, args_size = size,
        // args_offset = 0.
        0x60, 0x00, // PUSH1 0x00
        0x60, 0x00, // PUSH1 0x00
        0x82, // DUP3
        0x60, 0x00, // PUSH1 0x00
    ];
    if kind == ForwardKind::CallCode {
        // CALLCODE additionally takes a value, which we set to zero.
        code.extend([0x60, 0x00]); // PUSH1 0x00
    }
    code.extend([
        // target = CALLDATALOAD(0), gas = GAS
        0x60,
        0x00, // PUSH1 0x00
        0x35, // CALLDATALOAD
        0x5a, // GAS
        match kind {
            ForwardKind::DelegateCall => 0xf4, // DELEGATECALL
            ForwardKind::CallCode => 0xf2,     // CALLCODE
            ForwardKind::StaticCall => 0xfa,   // STATICCALL
        },
        // RETURNDATACOPY(dest = 0, offset = 0, size = RETURNDATASIZE)
        0x3d, // RETURNDATASIZE
        0x60,
        0x00, // PUSH1 0x00
        0x60,
        0x00, // PUSH1 0x00
        0x3e, // RETURNDATACOPY
    ]);
    // if !success { REVERT(0, RETURNDATASIZE) } else { RETURN(0, RETURNDATASIZE) }
    let jumpdest = u8::try_from(code.len() + 9).unwrap();
    code.extend([
        0x3d, // RETURNDATASIZE
        0x60, 0x00, // PUSH1 0x00
        0x82, // DUP3
        0x15, // ISZERO
        0x60, jumpdest, // PUSH1 jumpdest
        0x57,     // JUMPI
        0xf3,     // RETURN
        0x5b,     // JUMPDEST
        0xfd,     // REVERT
    ]);
    code
}

/// The addresses of the custom Humanode precompiles.
const HUMANODE_PRECOMPILES: [u64; 4] = [
    BIOAUTH,
    EVM_ACCOUNTS_MAPPING,
    NATIVE_CURRENCY,
    EVM_TO_NATIVE_SWAP,
];

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
                    .chain([
                        (TreasuryPot::account_id(), 10 * INIT_BALANCE),
                        (
                            TokenClaimsPot::account_id(),
                            <Balances as Inspect<AccountId>>::minimum_balance(),
                        ),
                        (
                            NativeToEvmSwapBridgePot::account_id(),
                            <Balances as Inspect<AccountId>>::minimum_balance(),
                        ),
                    ])
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
                            balance: <EvmBalances as Inspect<EvmAccountId>>::minimum_balance()
                                .into(),
                            code: Default::default(),
                            nonce: Default::default(),
                            storage: Default::default(),
                        },
                    )])
                    .chain(FORWARD_KINDS.into_iter().map(|kind| {
                        (
                            forwarder_address(kind),
                            fp_evm::GenesisAccount {
                                balance: INIT_BALANCE.into(),
                                code: forwarder_code(kind),
                                nonce: Default::default(),
                                storage: Default::default(),
                            },
                        )
                    }))
                    .collect()
            },
        },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}

/// Run an EVM call from `EvmAlice` and return the call info.
fn evm_call(to: H160, data: Vec<u8>, value: U256) -> fp_evm::CallInfo {
    <Runtime as pallet_evm::Config>::Runner::call(
        evm_account_id("EvmAlice"),
        to,
        data,
        value,
        200_000, // a reasonable upper bound for tests
        Some(*GAS_PRICE),
        Some(*GAS_PRICE),
        None,
        Vec::new(),
        true,
        true,
        None,
        None,
        <Runtime as pallet_evm::Config>::config(),
    )
    .unwrap()
}

/// Run `input` against `target` via a call of the given kind issued by the forwarder contract.
fn forward_call(kind: ForwardKind, target: H160, input: &[u8]) -> fp_evm::CallInfo {
    let mut data = EvmDataWriter::new().write(Address(target)).build();
    data.extend_from_slice(input);
    evm_call(forwarder_address(kind), data, U256::zero())
}

/// Run `input` against `target` via a `DELEGATECALL` issued by the forwarder contract.
fn delegate_call(target: H160, input: &[u8]) -> fp_evm::CallInfo {
    forward_call(ForwardKind::DelegateCall, target, input)
}

/// A sample state-changing input for the custom Humanode precompiles that have one.
fn humanode_precompile_state_changing_input(precompile: u64) -> Option<Vec<u8>> {
    match precompile {
        NATIVE_CURRENCY => Some(
            EvmDataWriter::new_with_selector(precompile_native_currency::Action::Transfer)
                .write(Address(evm_account_id("EvmBob")))
                .write(U256::from(1))
                .build(),
        ),
        EVM_TO_NATIVE_SWAP => Some(humanode_precompile_input(precompile)),
        BIOAUTH | EVM_ACCOUNTS_MAPPING => None,
        _ => unreachable!("not a Humanode precompile: {precompile:#x}"),
    }
}

/// A sample valid input for each of the custom Humanode precompiles.
fn humanode_precompile_input(precompile: u64) -> Vec<u8> {
    match precompile {
        BIOAUTH => EvmDataWriter::new_with_selector(precompile_bioauth::Action::IsAuthenticated)
            .write(H256::from(account_id("Alice").as_ref()))
            .build(),
        EVM_ACCOUNTS_MAPPING => evm_account_id("EvmAlice").as_bytes().to_vec(),
        NATIVE_CURRENCY => {
            EvmDataWriter::new_with_selector(precompile_native_currency::Action::BalanceOf)
                .write(Address(evm_account_id("EvmAlice")))
                .build()
        }
        EVM_TO_NATIVE_SWAP => {
            EvmDataWriter::new_with_selector(precompile_evm_to_native_swap::Action::Swap)
                .write(H256::from(account_id("Alice").as_ref()))
                .build()
        }
        _ => unreachable!("not a Humanode precompile: {precompile:#x}"),
    }
}

/// This test verifies that a `DELEGATECALL` or a `CALLCODE` to each custom Humanode precompile
/// reverts.
#[test]
fn humanode_precompiles_reject_delegate_call_and_call_code() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        for kind in [ForwardKind::DelegateCall, ForwardKind::CallCode] {
            for precompile in HUMANODE_PRECOMPILES {
                let execinfo = forward_call(
                    kind,
                    hash(precompile),
                    &humanode_precompile_input(precompile),
                );
                assert_eq!(
                    execinfo.exit_reason,
                    fp_evm::ExitReason::Revert(fp_evm::ExitRevert::Reverted),
                    "precompile {precompile:#x} did not revert on {kind:?}"
                );
                assert_eq!(
                    execinfo.value,
                    b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
                    "precompile {precompile:#x} reverted with an unexpected message on {kind:?}"
                );
                assert!(
                    execinfo.logs.is_empty(),
                    "precompile {precompile:#x} emitted logs on {kind:?}"
                );
            }
        }
    })
}

/// This test verifies that a `STATICCALL` to a state-changing function of a custom Humanode
/// precompile reverts, and that no state changes take place.
#[test]
fn humanode_precompiles_reject_static_call_for_state_changes() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let forwarder = forwarder_address(ForwardKind::StaticCall);
        let forwarder_balance_before =
            <EvmBalances as Inspect<EvmAccountId>>::total_balance(&forwarder);
        let bob_balance_before =
            <EvmBalances as Inspect<EvmAccountId>>::total_balance(&evm_account_id("EvmBob"));

        for precompile in HUMANODE_PRECOMPILES {
            let Some(input) = humanode_precompile_state_changing_input(precompile) else {
                continue;
            };
            let execinfo = forward_call(ForwardKind::StaticCall, hash(precompile), &input);
            assert_eq!(
                execinfo.exit_reason,
                fp_evm::ExitReason::Revert(fp_evm::ExitRevert::Reverted),
                "precompile {precompile:#x} did not revert on STATICCALL"
            );
            assert_eq!(
                execinfo.value,
                b"can't call non-static function in static context".to_vec(),
                "precompile {precompile:#x} reverted with an unexpected message on STATICCALL"
            );
            assert!(
                execinfo.logs.is_empty(),
                "precompile {precompile:#x} emitted logs on STATICCALL"
            );
        }

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Inspect<EvmAccountId>>::total_balance(&forwarder),
            forwarder_balance_before
        );
        assert_eq!(
            <EvmBalances as Inspect<EvmAccountId>>::total_balance(&evm_account_id("EvmBob")),
            bob_balance_before
        );
    })
}

/// This test verifies that a `STATICCALL` to a view function of a custom Humanode precompile
/// still works.
#[test]
fn humanode_precompiles_accept_static_call_for_views() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        for precompile in [BIOAUTH, EVM_ACCOUNTS_MAPPING, NATIVE_CURRENCY] {
            let execinfo = forward_call(
                ForwardKind::StaticCall,
                hash(precompile),
                &humanode_precompile_input(precompile),
            );
            assert_eq!(
                execinfo.exit_reason,
                fp_evm::ExitReason::Succeed(fp_evm::ExitSucceed::Returned),
                "precompile {precompile:#x} failed on STATICCALL: {:?}",
                String::from_utf8_lossy(&execinfo.value)
            );
        }
    })
}

/// This test verifies that a direct `CALL` to each custom Humanode precompile still works.
#[test]
fn humanode_precompiles_accept_direct_call() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        for precompile in HUMANODE_PRECOMPILES {
            let value = if precompile == EVM_TO_NATIVE_SWAP {
                U256::from(1000)
            } else {
                U256::zero()
            };
            let execinfo = evm_call(
                hash(precompile),
                humanode_precompile_input(precompile),
                value,
            );
            assert_eq!(
                execinfo.exit_reason,
                fp_evm::ExitReason::Succeed(fp_evm::ExitSucceed::Returned),
                "precompile {precompile:#x} failed on direct CALL: {:?}",
                String::from_utf8_lossy(&execinfo.value)
            );
        }
    })
}

/// This test verifies that the standard (stateless) precompiles remain callable via
/// `DELEGATECALL`.
#[test]
fn standard_precompiles_accept_delegate_call() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        let input = b"hello, humanode";

        // Ethereum precompiles.
        let execinfo = delegate_call(hash(IDENTITY), input);
        assert_eq!(
            execinfo.exit_reason,
            fp_evm::ExitReason::Succeed(fp_evm::ExitSucceed::Returned)
        );
        assert_eq!(execinfo.value, input.to_vec());

        let execinfo = delegate_call(hash(SHA_256), input);
        assert_eq!(
            execinfo.exit_reason,
            fp_evm::ExitReason::Succeed(fp_evm::ExitSucceed::Returned)
        );
        assert_eq!(execinfo.value, sp_io::hashing::sha2_256(input).to_vec());

        // BLS12-381 precompiles: adding the point at infinity to itself.
        let execinfo = delegate_call(hash(BLS12381_G1_ADD), &[0u8; 256]);
        assert_eq!(
            execinfo.exit_reason,
            fp_evm::ExitReason::Succeed(fp_evm::ExitSucceed::Returned)
        );
        assert_eq!(execinfo.value, vec![0u8; 128]);
    })
}
