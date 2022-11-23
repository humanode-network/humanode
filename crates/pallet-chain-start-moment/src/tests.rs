use frame_support::traits::Hooks;

use crate::mock::*;

fn set_timestamp(inc: UnixMilliseconds) {
    Timestamp::set(RuntimeOrigin::none(), inc).unwrap();
}

fn switch_block() {
    if System::block_number() != 0 {
        Timestamp::on_finalize(System::block_number());
        ChainStartMoment::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
    }
    System::set_block_number(System::block_number() + 1);
    System::on_initialize(System::block_number());
    Timestamp::on_initialize(System::block_number());
    ChainStartMoment::on_initialize(System::block_number());
}

/// This test verifies that the chain start moment is not set at genesis.
#[test]
fn value_is_not_set_at_genesis() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Assert the state.
        assert_eq!(ChainStartMoment::chain_start(), None);
    })
}

/// This test verifies that the chain start moment is set at the very first block.
#[test]
fn value_is_set_at_first_block() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Block 0.
        // Ensure we don't have any timestamp set first.
        assert_eq!(ChainStartMoment::chain_start(), None);

        // Block 0 -> 1.
        switch_block();
        set_timestamp(100);
        assert_eq!(ChainStartMoment::chain_start(), None,);

        // Block 1 -> 2.
        switch_block();
        assert_eq!(
            ChainStartMoment::chain_start(),
            Some(100),
            "the chain start must be set correctly right after the first block has been finalized"
        );
    })
}

/// This test verifies that the chain start moment is not written to after the first block.
#[test]
fn value_does_not_get_written_after_the_first_block() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Block 0 -> 1.
        switch_block();

        set_timestamp(100);

        // Block 1 -> 2.
        switch_block();

        set_timestamp(106);

        // Block 2 -> 3.
        switch_block();

        // Assert the state.
        assert_eq!(
            ChainStartMoment::chain_start(),
            Some(100),
            "the chain start moment must've been recorded at the first block"
        );
    })
}

/// This test verifies that the chain start moment is valid when capture it.
#[test]
#[should_panic = "the chain start moment is zero, it is not right"]
fn value_is_properly_checked() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Block 0 -> 1.
        switch_block();

        set_timestamp(0);

        // Block 1 -> 2.
        switch_block();
    })
}
