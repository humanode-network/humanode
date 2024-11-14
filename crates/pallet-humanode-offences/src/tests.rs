//! The tests for the pallet.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use crate::{
    mock::{new_test_ext, TestExternalitiesExt},
    *,
};

/// This test verifies that basic setup works in the happy path.
#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {});
}
