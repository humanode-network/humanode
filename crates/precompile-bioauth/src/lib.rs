//! A precompile to check if an address has an active bioauth or not.

#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::{
    Context, ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileOutput,
    PrecompileResult,
};
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The cost of the operation in gas.
// TODO(#352): implement proper dynamic gas cost estimation.
const GAS_COST: u64 = 200;

/// Exposes the current bioauth status of an address to the EVM.
pub struct Bioauth<Runtime>(PhantomData<Runtime>);

impl<T> Precompile for Bioauth<T>
where
    T: pallet_bioauth::Config,
    T::ValidatorPublicKey: for<'a> TryFrom<&'a [u8]> + Eq,
{
    fn execute(
        input: &[u8],
        _target_gas: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> PrecompileResult {
        let account_id =
            T::ValidatorPublicKey::try_from(input).map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("input must be a valid account id".into()),
            })?;

        let is_authorized = pallet_bioauth::ActiveAuthentications::<T>::get()
            .iter()
            .any(|active_authetication| active_authetication.public_key == account_id);

        let bytes = if is_authorized { &[1] } else { &[0] };

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: GAS_COST,
            output: bytes.to_vec(),
            logs: Default::default(),
        })
    }
}
