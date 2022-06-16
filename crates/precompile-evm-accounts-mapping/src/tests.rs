use hex_literal::hex;

use crate::{mock::*, *};

// This test denies invalid evm address input.
#[test]
fn test_error_invalid_input() {
    new_test_ext().execute_with(|| {
        let mut mock_handle = MockPrecompileHandle::new();
        mock_handle.expect_record_cost().returning(|_| Ok(()));
        mock_handle.expect_input().return_const(vec![]);
        let handle = &mut mock_handle as _;

        let err = crate::EvmAccountsMapping::<Test>::execute(handle).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("input must be a valid evm address".into())
            }
        );
    })
}

// This test returns a corresponding native account for provided evm address.
#[test]
fn test_success_mapped_evm_address() {
    new_test_ext().execute_with(|| {
        // Test data.
        let evm_address = pallet_evm_accounts_mapping::EvmAddress::from(hex!(
            "6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"
        ));
        let native_account = <Test as frame_system::Config>::AccountId::from(hex!(
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        ));

        // Current [`Accounts`] storage map.
        pallet_evm_accounts_mapping::Accounts::<Test>::insert(evm_address, native_account.clone());

        let mut mock_handle = MockPrecompileHandle::new();
        mock_handle.expect_record_cost().returning(|_| Ok(()));
        mock_handle
            .expect_input()
            .return_const(evm_address.as_bytes().to_vec());
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

// This test returns an empty output for unmapped evm address.
#[test]
fn test_success_unmapped_evm_address() {
    new_test_ext().execute_with(|| {
        // Test data.
        let evm_address = pallet_evm_accounts_mapping::EvmAddress::from(hex!(
            "6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"
        ));

        let mut mock_handle = MockPrecompileHandle::new();
        mock_handle.expect_record_cost().returning(|_| Ok(()));
        mock_handle
            .expect_input()
            .return_const(evm_address.as_bytes().to_vec());
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
