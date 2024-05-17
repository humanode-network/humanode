//! A precompile to check if an address has an active bioauth or not.

#![cfg_attr(not(feature = "std"), no_std)]

use pallet_evm::{
    ExitError, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use precompile_utils::{succeed, EvmDataWriter, EvmResult, PrecompileHandleExt};
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The cost of the operation in gas.
// TODO(#352): implement proper dynamic gas cost estimation.
const GAS_COST: u64 = 200;

/// Possible actions for this interface.
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Check if an address has been authenticated.
    IsAuthenticated = "isAuthenticated(bytes32)",
}

/// Exposes the current bioauth status of an address to the EVM.
pub struct Bioauth<Runtime>(PhantomData<Runtime>);

impl<T> Precompile for Bioauth<T>
where
    T: pallet_bioauth::Config,
    T::ValidatorPublicKey: for<'a> TryFrom<&'a [u8]> + Eq,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        handle.record_cost(GAS_COST)?;

        let selector = handle
            .read_selector()
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("invalid function selector".into()),
            })?;

        match selector {
            Action::IsAuthenticated => Self::is_authenticated(handle),
        }
    }
}

impl<T> Bioauth<T>
where
    T: pallet_bioauth::Config,
    T::ValidatorPublicKey: for<'a> TryFrom<&'a [u8]> + Eq,
{
    /// Check if input address is authenticated.
    fn is_authenticated(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        input
            .expect_arguments(1)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly one argument is expected".into()),
            })?;

        let account_id = T::ValidatorPublicKey::try_from(input.read_till_end()?).map_err(|_| {
            PrecompileFailure::Error {
                exit_status: ExitError::Other("input must be a valid account id".into()),
            }
        })?;

        let is_authenticated = pallet_bioauth::ActiveAuthentications::<T>::get()
            .iter()
            .any(|active_authentication| active_authentication.public_key == account_id);

        Ok(succeed(
            EvmDataWriter::new().write(is_authenticated).build(),
        ))
    }
}
