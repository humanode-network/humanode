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

        let input: [u8; 32] = sample_key;

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

        let input: [u8; 32] = sample_key;

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
