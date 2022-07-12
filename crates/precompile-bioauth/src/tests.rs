use frame_support::{traits::ConstU32, WeakBoundedVec};
use pallet_evm::ExitSucceed;
use precompile_utils::{Bytes, EvmDataWriter};

use crate::{mock::*, *};

type TestAuthentication = pallet_bioauth::Authentication<
    <Test as pallet_bioauth::Config>::ValidatorPublicKey,
    <Test as pallet_bioauth::Config>::Moment,
>;

fn make_bounded_authentications(
    authentications: Vec<pallet_bioauth::Authentication<ValidatorPublicKey, UnixMilliseconds>>,
) -> WeakBoundedVec<
    pallet_bioauth::Authentication<ValidatorPublicKey, UnixMilliseconds>,
    ConstU32<MAX_AUTHENTICATIONS>,
> {
    WeakBoundedVec::<_, ConstU32<MAX_AUTHENTICATIONS>>::try_from(authentications).unwrap()
}

#[test]
fn test_empty_selector() {
    new_test_ext().execute_with(|| {
        let mut mock_handle = MockPrecompileHandle::new();
        mock_handle.expect_record_cost().returning(|_| Ok(()));
        mock_handle.expect_input().return_const(vec![]);
        let handle = &mut mock_handle as _;

        let err = crate::Bioauth::<Test>::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("invalid function selector".into())
            }
        );
    })
}

#[test]
fn test_empty_input() {
    new_test_ext().execute_with(|| {
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated).build();

        let mut mock_handle = MockPrecompileHandle::new();
        mock_handle.expect_record_cost().returning(|_| Ok(()));
        mock_handle.expect_input().return_const(input);
        let handle = &mut mock_handle as _;

        let err = crate::Bioauth::<Test>::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly one arguement is expected".into())
            }
        );
    })
}

#[test]
fn test_invalid_input() {
    new_test_ext().execute_with(|| {
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated)
            .write(Bytes::from("invalid input"))
            .build();

        let mut mock_handle = MockPrecompileHandle::new();
        mock_handle.expect_record_cost().returning(|_| Ok(()));
        mock_handle.expect_input().return_const(input);
        let handle = &mut mock_handle as _;

        let err = crate::Bioauth::<Test>::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("input must be a valid account id".into())
            }
        );
    })
}

#[test]
fn test_authorized() {
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

        let mut mock_handle = MockPrecompileHandle::new();
        mock_handle.expect_record_cost().returning(|_| Ok(()));
        mock_handle.expect_input().return_const(input);
        let handle = &mut mock_handle as _;

        let val = crate::Bioauth::<Test>::execute(handle).unwrap();
        assert_eq!(
            val,
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: vec![1],
            }
        );
    })
}

#[test]
fn test_not_authorized() {
    new_test_ext().execute_with(|| {
        let input = EvmDataWriter::new_with_selector(Action::IsAuthenticated)
            .write(sp_core::H256::from([0; 32]))
            .build();

        pallet_bioauth::ActiveAuthentications::<Test>::put(make_bounded_authentications(vec![]));

        let mut mock_handle = MockPrecompileHandle::new();
        mock_handle.expect_record_cost().returning(|_| Ok(()));
        mock_handle.expect_input().return_const(input);
        let handle = &mut mock_handle as _;

        let val = crate::Bioauth::<Test>::execute(handle).unwrap();
        assert_eq!(
            val,
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: vec![0],
            }
        );
    })
}
