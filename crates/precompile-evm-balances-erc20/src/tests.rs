#![allow(clippy::integer_arithmetic)] // not a problem in tests

use precompile_utils::{testing::*, EvmDataWriter};

use crate::{mock::*, *};

fn precompiles() -> Precompiles<Test> {
    PrecompilesValue::get()
}

#[test]
fn name_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));

        let name_action = EvmDataWriter::new_with_selector(Action::Name).build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, name_action)
            .expect_cost(0)
            .expect_no_logs()
            .execute_returns(EvmDataWriter::new().write(Bytes::from(NAME)).build());
    });
}

#[test]
fn symbol_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));

        let symbol_action = EvmDataWriter::new_with_selector(Action::Symbol).build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, symbol_action)
            .expect_cost(0)
            .expect_no_logs()
            .execute_returns(EvmDataWriter::new().write(Bytes::from(SYMBOL)).build());
    });
}

#[test]
fn decimals_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));

        let decimals_action = EvmDataWriter::new_with_selector(Action::Decimals).build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, decimals_action)
            .expect_cost(0)
            .expect_no_logs()
            .execute_returns(EvmDataWriter::new().write(DECIMALS).build());
    });
}
