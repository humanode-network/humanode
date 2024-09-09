//! The tests for the pallet.

use crate::{
    mock::{new_test_ext, Test, TestExternalitiesExt},
    *,
};

/// This test verifies that basic setup works in the happy path.
#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {
        assert!(<BannedAccounts<Test>>::get().is_empty());
    });
}
