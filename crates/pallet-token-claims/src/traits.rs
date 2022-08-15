//! Traits we use and expose.

use core::marker::PhantomData;

use frame_support::dispatch::DispatchResult;
use primitives_ethereum::{EcdsaSignature, EthereumAddress};

/// The verifier for the Ethereum signature.
///
/// The idea is we don't pass in the message we use for the verification, but instead we pass in
/// the message parameters.
///
/// This abstraction is built with EIP-712 in mind, but can also be implemented with any generic
/// ECDSA signature.
pub trait EthereumSignatureVerifier {
    /// The type describing the parameters used to construct a message.
    type MessageParams;

    /// Generate a message and verify the provided `signature` against the said message.
    /// Extract the [`EthereumAddress`] from the signature and return it.
    ///
    /// The caller should check that the extracted address matches what is expected, as successfull
    /// recovery does not necessarily guarantee the correctness of the signature - that can only
    /// be achieved with checking the recovered address against the expected one.
    fn recover_signer(
        signature: &EcdsaSignature,
        message_params: &Self::MessageParams,
    ) -> Option<EthereumAddress>;
}

/// Calls [`EthereumSignatureVerifier::recover_signer`] and then checks that the `signer`
/// matches the recovered address.
pub fn verify_ethereum_signature<T: EthereumSignatureVerifier>(
    signature: &EcdsaSignature,
    message_params: &T::MessageParams,
    signer: &EthereumAddress,
) -> bool {
    let recovered = match T::recover_signer(signature, message_params) {
        Some(recovered) => recovered,
        None => return false,
    };
    &recovered == signer
}

/// The interface to the vesting implementation.
pub trait VestingInterface {
    /// The Account ID to apply vesting to.
    type AccountId;
    /// The type of balance to lock under the vesting.
    type Balance;
    /// The vesting schedule configuration.
    type Schedule;

    /// Lock the specified amount of balance (`balance_to_lock`) on the given account (`account`)
    /// with the provided vesting schedule configuration (`schedule`).
    fn lock_under_vesting(
        account: &Self::AccountId,
        balance_to_lock: Self::Balance,
        schedule: Self::Schedule,
    ) -> DispatchResult;
}

/// A vesting interface that doesn't implement any vesting.
pub struct NoVesting<T>(PhantomData<T>);

impl<T: crate::Config> VestingInterface for NoVesting<T> {
    type AccountId = T::AccountId;
    type Balance = crate::BalanceOf<T>;
    type Schedule = ();

    fn lock_under_vesting(
        _account: &Self::AccountId,
        _balance_to_lock: Self::Balance,
        _schedule: Self::Schedule,
    ) -> DispatchResult {
        Ok(())
    }
}

/// A vesting interface that allows wrapping any mandatory vesting into an optional form.
pub struct OptionalVesting<T>(PhantomData<T>);

impl<T: VestingInterface> VestingInterface for OptionalVesting<T> {
    type AccountId = <T as VestingInterface>::AccountId;
    type Balance = <T as VestingInterface>::Balance;
    type Schedule = Option<<T as VestingInterface>::Schedule>;

    fn lock_under_vesting(
        account: &Self::AccountId,
        balance_to_lock: Self::Balance,
        schedule: Self::Schedule,
    ) -> DispatchResult {
        if let Some(schedule) = schedule {
            return T::lock_under_vesting(account, balance_to_lock, schedule);
        }
        Ok(())
    }
}
