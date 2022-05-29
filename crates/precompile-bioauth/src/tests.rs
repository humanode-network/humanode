use std::io::Write;

use frame_support::WeakBoundedVec;

use crate::{mock::*, *};

type TestAuthentication = pallet_bioauth::Authentication<
    <Test as pallet_bioauth::Config>::ValidatorPublicKey,
    <Test as pallet_bioauth::Config>::Moment,
>;

fn make_bounded_authentications(
    authentications: Vec<pallet_bioauth::Authentication<ValidatorPublicKey, UnixMilliseconds>>,
) -> WeakBoundedVec<
    pallet_bioauth::Authentication<ValidatorPublicKey, UnixMilliseconds>,
    MaxAuthentications,
> {
    WeakBoundedVec::<_, MaxAuthentications>::try_from(authentications).unwrap()
}

fn make_selector(selector: u32) -> [u8; 4] {
    selector.to_be_bytes()
}

#[test]
fn test_empty_input() {
    new_test_ext().execute_with(|| {
        let input: [u8; 0] = [];

        let cost: u64 = 1;

        let context: Context = Context {
            address: Default::default(),
            caller: Default::default(),
            apparent_value: From::from(0),
        };

        let err = crate::Bioauth::<Test>::execute(&input, Some(cost), &context, false).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("unable to read the function selector".into())
            }
        );
    })
}

#[test]
fn test_invalid_selector() {
    new_test_ext().execute_with(|| {
        let mut input = Vec::new();
        input.write_all(&make_selector(0x00000000)).unwrap();

        let cost: u64 = 1;

        let context: Context = Context {
            address: Default::default(),
            caller: Default::default(),
            apparent_value: From::from(0),
        };

        let err = crate::Bioauth::<Test>::execute(&input, Some(cost), &context, false).unwrap_err();
        assert_eq!(
            err,
            PrecompileFailure::Error {
                exit_status: ExitError::Other("unable to read the function selector".into())
            }
        );
    })
}

#[test]
fn test_bad_argument() {
    new_test_ext().execute_with(|| {
        let mut input = Vec::new();
        input.write_all(&make_selector(0xe3c90bb9)).unwrap();

        let cost: u64 = 1;

        let context: Context = Context {
            address: Default::default(),
            caller: Default::default(),
            apparent_value: From::from(0),
        };

        let err = crate::Bioauth::<Test>::execute(&input, Some(cost), &context, false).unwrap_err();
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

        pallet_bioauth::ActiveAuthentications::<Test>::put(make_bounded_authentications(vec![
            TestAuthentication {
                public_key: sample_key,
                expires_at: 1,
            },
        ]));

        let mut input = Vec::new();
        input.write_all(&make_selector(0xe3c90bb9)).unwrap();
        input.write_all(&sample_key).unwrap();

        let cost: u64 = 1;

        let context: Context = Context {
            address: Default::default(),
            caller: Default::default(),
            apparent_value: From::from(0),
        };

        let val = crate::Bioauth::<Test>::execute(&input, Some(cost), &context, false).unwrap();
        assert_eq!(
            val,
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                cost: 200,
                output: vec![1],
                logs: vec![],
            }
        );
    })
}

#[test]
fn test_not_authorized() {
    new_test_ext().execute_with(|| {
        let sample_key = [0; 32];

        pallet_bioauth::ActiveAuthentications::<Test>::put(make_bounded_authentications(vec![]));

        let mut input = Vec::new();
        input.write_all(&make_selector(0xe3c90bb9)).unwrap();
        input.write_all(&sample_key).unwrap();

        let cost: u64 = 1;

        let context: Context = Context {
            address: Default::default(),
            caller: Default::default(),
            apparent_value: From::from(0),
        };

        let val = crate::Bioauth::<Test>::execute(&input, Some(cost), &context, false).unwrap();
        assert_eq!(
            val,
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                cost: 200,
                output: vec![0],
                logs: vec![],
            }
        );
    })
}
