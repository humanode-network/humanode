//! A precompile to check and return a proper native account for provided ethereum address.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use fp_evm::{
    ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
    PrecompileResult,
};
use primitives_ethereum::EthereumAddress;
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The cost of the operation in gas.
// TODO(#378): implement proper dynamic gas cost estimation.
const GAS_COST: u64 = 200;

/// Exposes a proper evm accounts mapping to native accounts.
pub struct EvmAccountsMapping<Runtime>(PhantomData<Runtime>);

impl<T> Precompile for EvmAccountsMapping<T>
where
    T: pallet_evm_accounts_mapping::Config,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        handle.record_cost(GAS_COST)?;

        let ethereum_address_bytes: [u8; 20] =
            handle
                .input()
                .try_into()
                .map_err(|_| PrecompileFailure::Error {
                    exit_status: ExitError::Other("input must be a valid ethereum address".into()),
                })?;

        let ethereum_address = EthereumAddress(ethereum_address_bytes);
        let native_account = pallet_evm_accounts_mapping::Accounts::<T>::get(ethereum_address);

        let precompile_output = match native_account {
            Some(account) => account.encode(),
            None => sp_std::vec![],
        };

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: precompile_output,
        })
    }
}
