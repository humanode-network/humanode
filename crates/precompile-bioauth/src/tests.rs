use frame_support::{traits::ConstU32, BoundedVec};
use mockall::{predicate::eq, Sequence};
use pallet_evm::ExitSucceed;
use precompile_utils::{Bytes, EvmDataWriter};

use crate::{mock::*, *};

type TestAuthentication = pallet_bioauth::Authentication<
    <Test as pallet_bioauth::Config>::ValidatorPublicKey,
    <Test as pallet_bioauth::Config>::Moment,
>;

fn make_bounded_authentications(
    authentications: Vec<pallet_bioauth::Authentication<ValidatorPublicKey, UnixMilliseconds>>,
) -> BoundedVec<
    pallet_bioauth::Authentication<ValidatorPublicKey, UnixMilliseconds>,
    ConstU32<MAX_AUTHENTICATIONS>,
> {
    BoundedVec::<_, ConstU32<MAX_AUTHENTICATIONS>>::try_from(authentications).unwrap()
}

/// The address the precompile is reached at in the tests.
fn precompile_address() -> sp_core::H160 {
    sp_core::H160::from_low_u64_be(2048)
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

#[test]
fn test_delegate_call() {
    new_test_ext().execute_with(|| {
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated)
            .write(sp_core::H256::from([0; 32]))
            .build();

        // Under `DELEGATECALL` the context address is the delegating contract.
        let mut mock_handle = mock_handle(
            input,
            fp_evm::Context {
                address: sp_core::H160::from_low_u64_be(0xf0f0f0),
                ..regular_context()
            },
        );
        let handle = &mut mock_handle as _;

        let err = BioauthPrecompile::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
            }
        );
    })
}

#[test]
fn test_value() {
    new_test_ext().execute_with(|| {
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated)
            .write(sp_core::H256::from([0; 32]))
            .build();

        let mut mock_handle = mock_handle(
            input,
            fp_evm::Context {
                apparent_value: 1.into(),
                ..regular_context()
            },
        );
        let handle = &mut mock_handle as _;

        let err = BioauthPrecompile::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: b"function is not payable".to_vec(),
            }
        );
    })
}

#[test]
fn test_empty_selector() {
    new_test_ext().execute_with(|| {
        let mut mock_handle = mock_handle(vec![], regular_context());
        let handle = &mut mock_handle as _;

        let err = BioauthPrecompile::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("invalid function selector".into())
            }
        );
    })
}

#[test]
fn test_is_authenticated_empty_input() {
    new_test_ext().execute_with(|| {
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated).build();

        let mut mock_handle = mock_handle(input, regular_context());
        let handle = &mut mock_handle as _;

        let err = BioauthPrecompile::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly one argument is expected".into())
            }
        );
    })
}

#[test]
fn test_is_authenticated_invalid_input() {
    new_test_ext().execute_with(|| {
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated)
            .write(Bytes::from("invalid input"))
            .build();

        let mut mock_handle = mock_handle(input, regular_context());
        let handle = &mut mock_handle as _;

        let err = BioauthPrecompile::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("input must be a valid account id".into())
            }
        );
    })
}

#[test]
fn test_is_authenticated_true() {
    new_test_ext().execute_with(|| {
        let sample_key = [0; 32];
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated)
            .write(sp_core::H256::from(sample_key))
            .build();

        pallet_bioauth::ActiveAuthentications::<Test>::put(make_bounded_authentications(vec![
            TestAuthentication {
                public_key: sample_key,
                expires_at: 1,
            },
        ]));

        let mut mock_handle = mock_handle(input, regular_context());
        let handle = &mut mock_handle as _;

        let val = BioauthPrecompile::execute(handle).unwrap();
        assert_eq!(
            val,
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: EvmDataWriter::new().write(true).build(),
            }
        );
    })
}

/// This test verifies the gas costs of the call: the list read is charged before the read, and
/// the scan is charged for the actual list size before the scan.
#[test]
fn test_gas_costs() {
    new_test_ext().execute_with(|| {
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated)
            .write(sp_core::H256::from([0; 32]))
            .build();

        pallet_bioauth::ActiveAuthentications::<Test>::put(make_bounded_authentications(
            (1..=3)
                .map(|i| TestAuthentication {
                    public_key: [i; 32],
                    expires_at: 1,
                })
                .collect(),
        ));

        let mut mock_handle = MockPrecompileHandle::new();
        let mut seq = Sequence::new();
        mock_handle
            .expect_record_cost()
            .with(eq(READ_GAS_COST))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));
        mock_handle
            .expect_record_cost()
            .with(eq(3 * SCAN_GAS_COST_PER_AUTHENTICATION))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));
        mock_handle.expect_input().return_const(input);
        mock_handle
            .expect_code_address()
            .return_const(precompile_address());
        mock_handle.expect_context().return_const(regular_context());
        let handle = &mut mock_handle as _;

        let val = BioauthPrecompile::execute(handle).unwrap();
        assert_eq!(
            val,
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: EvmDataWriter::new().write(false).build(),
            }
        );

        assert_eq!(BioauthPrecompile::read_gas_cost(), READ_GAS_COST);
        assert_eq!(
            BioauthPrecompile::scan_gas_cost(1),
            SCAN_GAS_COST_PER_AUTHENTICATION
        );
        assert_eq!(
            BioauthPrecompile::max_gas_cost(),
            READ_GAS_COST + u64::from(MAX_AUTHENTICATIONS) * SCAN_GAS_COST_PER_AUTHENTICATION
        );
    })
}

#[test]
fn test_is_authenticated_false() {
    new_test_ext().execute_with(|| {
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated)
            .write(sp_core::H256::from([0; 32]))
            .build();

        pallet_bioauth::ActiveAuthentications::<Test>::put(make_bounded_authentications(vec![]));

        let mut mock_handle = mock_handle(input, regular_context());
        let handle = &mut mock_handle as _;

        let val = BioauthPrecompile::execute(handle).unwrap();
        assert_eq!(
            val,
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: EvmDataWriter::new().write(false).build(),
            }
        );
    })
}
