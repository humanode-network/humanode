use frame_support::traits::fungible::{conformance_tests, Inspect, Mutate};
use paste::paste;

use crate::{mock::*, *};

macro_rules! run_tests {
    ($path:path, $($name:ident),*) => {
		$(
			paste! {
				#[test]
				fn [< $name _dust_trap_on >]() {
					let trap_account = <Test as Config>::AccountId::from(65174286u64);
					new_test_ext().execute_with_ext(|_| {
						EvmBalances::set_balance(&trap_account, EvmBalances::minimum_balance());
						$path::$name::<
							EvmBalances,
							<Test as Config>::AccountId,
						>(Some(trap_account));
					});
				}

				#[test]
				fn [< $name _dust_trap_off >]() {
					new_test_ext().execute_with_ext(|_| {
						$path::$name::<
							EvmBalances,
							<Test as Config>::AccountId,
						>(None);
					});
				}
			}
		)*
	};
	($path:path) => {
		run_tests!(
			$path,
			mint_into_success,
			mint_into_overflow,
			mint_into_below_minimum,
			burn_from_exact_success,
			burn_from_best_effort_success,
			burn_from_exact_insufficient_funds,
			restore_success,
			restore_overflow,
			restore_below_minimum,
			shelve_success,
			shelve_insufficient_funds,
			transfer_success,
			transfer_expendable_all,
			transfer_expendable_dust,
			transfer_protect_preserve,
			set_balance_mint_success,
			set_balance_burn_success,
			can_deposit_success,
			can_deposit_below_minimum,
			can_deposit_overflow,
			can_withdraw_success,
			can_withdraw_reduced_to_zero,
			can_withdraw_balance_low,
			reducible_balance_expendable,
			reducible_balance_protect_preserve
		);
	};
}

run_tests!(conformance_tests::inspect_mutate);
