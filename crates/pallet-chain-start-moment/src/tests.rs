use frame_support::traits::Hooks;

use crate::mock::*;

fn set_initial_timestamp(val: UnixMilliseconds) {
    assert!(!<pallet_timestamp::Now<Test>>::exists());
    <pallet_timestamp::Now<Test>>::put(val);
}

fn get_current_timestamp_checked() -> UnixMilliseconds {
    assert!(<pallet_timestamp::Now<Test>>::exists());
    <pallet_timestamp::Now<Test>>::get()
}

fn run_to_block(n: u64, time_per_block: UnixMilliseconds) {
    while System::block_number() < n {
        Timestamp::set(
            Origin::none(),
            get_current_timestamp_checked() + time_per_block,
        )
        .unwrap();
        Timestamp::on_finalize(System::block_number());
        ChainStartMoment::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Timestamp::on_initialize(System::block_number());
        ChainStartMoment::on_initialize(System::block_number());
    }
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
        set_initial_timestamp(100);

        // Ensure we don't have any timestamp set first.
        assert_eq!(ChainStartMoment::chain_start(), None);

        // Run for one block.
        run_to_block(1, 6);

        // Assert the state.
        assert_eq!(
            ChainStartMoment::chain_start(),
            Some(106),
            "the chain start must be set correctly right after the first block has initalized"
        );
    })
}

/// This test verifies that the chain start moment is not written to after the first block.
#[test]
fn value_does_not_get_written_after_the_first_block() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        set_initial_timestamp(100);

        // Run for two block.
        run_to_block(2, 6);

        // Assert the state.
        assert_eq!(
            ChainStartMoment::chain_start(),
            Some(106),
            "the chain start moment must've been recorded at the first block"
        );
    })
}
