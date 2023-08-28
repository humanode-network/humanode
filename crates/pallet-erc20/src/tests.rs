use crate::{mock::*, Approvals};

/// This test verifies basic approvals related flow.
#[test]
fn approvals_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42_u64;
        let bob = 52_u64;
        let approved_balance = 999;

        // Check test preconditions.
        assert_eq!(<Approvals<Test>>::get(alice, bob), None);

        // Store alice-bob approval.
        <Approvals<Test>>::insert(alice, bob, approved_balance);

        // Verify alice-bob approval existence.
        assert_eq!(<Approvals<Test>>::get(alice, bob), Some(approved_balance));
    })
}
