//! A precompile to check if an address has an active bioauth or not.

#![cfg_attr(not(feature = "std"), no_std)]

use pallet_evm::{ExitError, Precompile, PrecompileFailure, PrecompileOutput};
use precompile_utils::{succeed, EvmResult, PrecompileHandleExt};
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
    fn execute(handle: &mut impl pallet_evm::PrecompileHandle) -> pallet_evm::PrecompileResult {
        handle.record_cost(GAS_COST)?;

        let selector = handle.read_selector()?;

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
    fn is_authenticated(
        handle: &mut impl pallet_evm::PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        input
            .expect_arguments(1)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("input must be a valid account id".into()),
            })?;

        let account_id = T::ValidatorPublicKey::try_from(input.read_till_end()?).map_err(|_| {
            PrecompileFailure::Error {
                exit_status: ExitError::Other("input must be a valid account id".into()),
            }
        })?;

        let is_authenticated = pallet_bioauth::ActiveAuthentications::<T>::get()
            .iter()
            .any(|active_authetication| active_authetication.public_key == account_id);

        let bytes = if is_authenticated { &[1] } else { &[0] };

        Ok(succeed(bytes))
    }
}
