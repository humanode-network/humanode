use hex_literal::hex;
use primitives_ethereum::EthereumAddress;

use crate::{mock::*, *};

/// The address the precompile is reached at in the tests.
fn precompile_address() -> sp_core::H160 {
    sp_core::H160::from_low_u64_be(2049)
}

/// The context of a regular `CALL` to the precompile without value.
fn regular_context() -> fp_evm::Context {
    fp_evm::Context {
        address: precompile_address(),
        caller: sp_core::H160::from_low_u64_be(0xa11ce),
        apparent_value: 0.into(),
    }
}

/// Build a mock handle for a call to the precompile with the given `input` and `context`.
fn mock_handle(input: Vec<u8>, context: fp_evm::Context) -> MockPrecompileHandle {
    let mut mock_handle = MockPrecompileHandle::new();
    mock_handle.expect_record_cost().returning(|_| Ok(()));
    mock_handle.expect_input().return_const(input);
    mock_handle
        .expect_code_address()
        .return_const(precompile_address());
    mock_handle.expect_context().return_const(context);
    mock_handle
}

// This test denies the call made via `DELEGATECALL`.
#[test]
fn test_error_delegate_call() {
    new_test_ext().execute_with(|| {
        let ethereum_address = EthereumAddress(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"));

        // Under `DELEGATECALL` the context address is the delegating contract.
        let mut mock_handle = mock_handle(
            ethereum_address.0.to_vec(),
            fp_evm::Context {
                address: sp_core::H160::from_low_u64_be(0xf0f0f0),
                ..regular_context()
            },
        );
        let handle = &mut mock_handle as _;

        let err = crate::EvmAccountsMapping::<Test>::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
            }
        );
    })
}

// This test denies the call with value.
#[test]
fn test_error_value() {
    new_test_ext().execute_with(|| {
        let ethereum_address = EthereumAddress(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"));

        let mut mock_handle = mock_handle(
            ethereum_address.0.to_vec(),
            fp_evm::Context {
                apparent_value: 1.into(),
                ..regular_context()
            },
        );
        let handle = &mut mock_handle as _;

        let err = crate::EvmAccountsMapping::<Test>::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: b"function is not payable".to_vec(),
            }
        );
    })
}

// This test denies invalid ethereum address input.
#[test]
fn test_error_invalid_input() {
    new_test_ext().execute_with(|| {
        let mut mock_handle = mock_handle(vec![], regular_context());
        let handle = &mut mock_handle as _;

        let err = crate::EvmAccountsMapping::<Test>::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("input must be a valid ethereum address".into())
            }
        );
    })
}

// This test returns a corresponding native account for provided ethereum address.
#[test]
fn test_success_mapped_ethereum_address() {
    new_test_ext().execute_with(|| {
        // Test data.
        let ethereum_address = EthereumAddress(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"));
        let native_account = <Test as frame_system::Config>::AccountId::from(hex!(
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        ));

        // Current [`Accounts`] storage map.
        pallet_evm_accounts_mapping::Accounts::<Test>::insert(
            ethereum_address,
            native_account.clone(),
        );

        let mut mock_handle = mock_handle(ethereum_address.0.to_vec(), regular_context());
        let handle = &mut mock_handle as _;

        let val = crate::EvmAccountsMapping::<Test>::execute(handle).unwrap();
        assert_eq!(
            val,
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: native_account.encode(),
            }
        );
    })
}

// This test returns an empty output for unmapped ethereum address.
#[test]
fn test_success_unmapped_ethereum_address() {
    new_test_ext().execute_with(|| {
        // Test data.
        let ethereum_address = EthereumAddress(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"));

        let mut mock_handle = mock_handle(ethereum_address.0.to_vec(), regular_context());
        let handle = &mut mock_handle as _;

        let val = crate::EvmAccountsMapping::<Test>::execute(handle).unwrap();
        assert_eq!(
            val,
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: vec![],
            }
        );
    })
}
