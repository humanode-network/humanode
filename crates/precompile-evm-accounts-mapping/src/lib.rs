//! A precompile to check and return a proper native account for provided evm address.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use fp_evm::{
    ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
    PrecompileResult,
};
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

        let evm_address_bytes: [u8; 20] =
            handle
                .input()
                .try_into()
                .map_err(|_| PrecompileFailure::Error {
                    exit_status: ExitError::Other("input must be a valid evm address".into()),
                })?;

        let evm_address = pallet_evm_accounts_mapping::EvmAddress::from(evm_address_bytes);
        let native_account = pallet_evm_accounts_mapping::Accounts::<T>::get(evm_address)
            .ok_or_else(|| PrecompileFailure::Error {
                exit_status: ExitError::Other("evm address isn't mapped".into()),
            })?;

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: native_account.encode(),
        })
    }
}
