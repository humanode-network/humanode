//! The tests for the pallet.

use primitives_ethereum::EthereumAddress;

use crate::{
    mock::{new_test_ext, Test},
    *,
};

#[test]
fn basic_setup_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Claims::<Test>::get(&EthereumAddress::default()), None);
    });
}
