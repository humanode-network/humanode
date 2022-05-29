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

/// A list of supported contract functions.
enum ContractFunction {
    /// Signature: `isAuthenticated(bytes32)`
    /// Selector:  `e3c90bb9`
    IsAuthenticated,
}

/// Read four (4) bytes from the input and parse them into a [`u32`] representing a selector.
fn read_selector_bytes(selector: &[u8]) -> Option<u32> {
    let selector = selector.get(0..4)?;
    let mut buffer = [0; 4];
    buffer.copy_from_slice(selector);
    let selector = u32::from_be_bytes(buffer);
    Some(selector)
}

impl ContractFunction {
    /// Determine a contract function from a selector.
    fn from_selector(selector: &[u8]) -> Option<ContractFunction> {
        let selector = read_selector_bytes(selector)?;
        match selector {
            0xe3c90bb9 => Some(Self::IsAuthenticated),
            _ => None,
        }
    }
}

impl<T> Precompile for Bioauth<T>
where
    T: pallet_bioauth::Config,
    T::ValidatorPublicKey: for<'a> TryFrom<&'a [u8]> + Eq,
{
    fn execute(
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
        is_static: bool,
    ) -> PrecompileResult {
        let funtion =
            ContractFunction::from_selector(input).ok_or_else(|| PrecompileFailure::Error {
                exit_status: ExitError::Other("unable to read the function selector".into()),
            })?;

        let data = &input[4..];

        match funtion {
            ContractFunction::IsAuthenticated => {
                Self::is_authenticated(data, target_gas, context, is_static)
            }
        }
    }
}

impl<T> Bioauth<T>
where
    T: pallet_bioauth::Config,
    T::ValidatorPublicKey: for<'a> TryFrom<&'a [u8]> + Eq,
{
    /// Implements [`ContractFunction::IsAuthenticated`].
    fn is_authenticated(
        data: &[u8],
        _target_gas: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> PrecompileResult {
        let account_id =
            T::ValidatorPublicKey::try_from(data).map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("input must be a valid account id".into()),
            })?;

        let is_authenticated = pallet_bioauth::ActiveAuthentications::<T>::get()
            .iter()
            .any(|active_authetication| active_authetication.public_key == account_id);

        let bytes = if is_authenticated { &[1] } else { &[0] };

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: GAS_COST,
            output: bytes.to_vec(),
            logs: Default::default(),
        })
    }
}
