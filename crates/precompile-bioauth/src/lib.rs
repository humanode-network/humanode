//! A precompile to check if an address has an active bioauth or not.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::MaxEncodedLen;
use frame_support::traits::Get;
use pallet_evm::{
    ExitError, ExitRevert, GasWeightMapping, Precompile, PrecompileFailure, PrecompileHandle,
    PrecompileOutput, PrecompileResult,
};
use precompile_utils::{succeed, EvmDataWriter, EvmResult, PrecompileHandleExt};
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Possible actions for this interface.
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Check if an address has been authenticated.
    IsAuthenticated = "isAuthenticated(bytes32)",
}

/// Exposes the current bioauth status of an address to the EVM.
///
/// The calls are priced through the `GasWeightMapping`, see [`Bioauth::read_gas_cost`] and
/// [`Bioauth::scan_gas_cost`].
pub struct Bioauth<Runtime, GasWeightMapping>(PhantomData<(Runtime, GasWeightMapping)>);

impl<T, G> Precompile for Bioauth<T, G>
where
    T: pallet_bioauth::Config,
    T::ValidatorPublicKey: for<'a> TryFrom<&'a [u8]> + Eq,
    G: GasWeightMapping,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        ensure_not_delegate_call(handle)?;
        ensure_no_value(handle)?;

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

/// The gas costs of the EVM operations the active authentications scan is priced after.
///
/// Ref: <https://github.com/rust-blockchain/evm/blob/master/gasometer/src/consts.rs>.
mod evm_gas {
    /// The cost of copying a 32-byte word of data (the `*COPY` opcodes).
    pub const G_COPY: u64 = 3;
    /// The cost of the cheapest opcodes, the comparisons (`EQ`) among them.
    pub const G_VERYLOW: u64 = 3;
}

impl<T, G> Bioauth<T, G>
where
    T: pallet_bioauth::Config,
    T::ValidatorPublicKey: for<'a> TryFrom<&'a [u8]> + Eq,
    G: GasWeightMapping,
{
    /// The gas cost of reading the active authentications list from the storage.
    ///
    /// A single storage read, priced from the runtime DB weights through the gas-to-weight
    /// mapping, the same way the EVM opcodes are priced.
    pub fn read_gas_cost() -> u64 {
        G::weight_to_gas(<T as frame_system::Config>::DbWeight::get().reads(1))
    }

    /// The gas cost of scanning through `authentications` active authentications.
    ///
    /// Each authentication is decoded and compared against the requested public key, which is
    /// priced the same way the EVM prices copying the encoded authentication and comparing it:
    /// the copy cost per 32-byte word of the encoded authentication, plus the comparison cost.
    pub fn scan_gas_cost(authentications: u64) -> u64 {
        let encoded_len =
            pallet_bioauth::Authentication::<T::ValidatorPublicKey, T::Moment>::max_encoded_len()
                as u64;
        let words = encoded_len.div_ceil(32);
        let cost_per_authentication = words
            .saturating_mul(evm_gas::G_COPY)
            .saturating_add(evm_gas::G_VERYLOW);
        authentications.saturating_mul(cost_per_authentication)
    }

    /// The worst-case gas cost of a call to the precompile: a storage read and a scan through
    /// the maximum amount of active authentications.
    pub fn max_gas_cost() -> u64 {
        Self::read_gas_cost()
            .saturating_add(Self::scan_gas_cost(u64::from(T::MaxAuthentications::get())))
    }

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

        handle.record_cost(Self::read_gas_cost())?;
        let active_authentications = pallet_bioauth::ActiveAuthentications::<T>::get();

        handle.record_cost(Self::scan_gas_cost(active_authentications.len() as u64))?;
        let is_authenticated = active_authentications
            .iter()
            .any(|active_authentication| active_authentication.public_key == account_id);

        Ok(succeed(
            EvmDataWriter::new().write(is_authenticated).build(),
        ))
    }
}

/// Rejects the call when it is made via `DELEGATECALL` or `CALLCODE`.
///
/// Under those the context address belongs to the calling frame and differs from the precompile
/// code address. The runtime precompile set guards against this too; this check keeps
/// the precompile safe on its own, regardless of how it is wired.
fn ensure_not_delegate_call(handle: &impl PrecompileHandle) -> EvmResult {
    if handle.code_address() != handle.context().address {
        return Err(PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
        });
    }
    Ok(())
}

/// Rejects the call when value is sent along with it.
///
/// The precompile is read-only and makes no use of the value, which would otherwise end up stuck
/// at the precompile address, as nothing can ever move it out from there.
fn ensure_no_value(handle: &impl PrecompileHandle) -> EvmResult {
    if !handle.context().apparent_value.is_zero() {
        return Err(PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: b"function is not payable".to_vec(),
        });
    }
    Ok(())
}
