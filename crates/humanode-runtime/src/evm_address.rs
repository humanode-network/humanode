use sp_runtime::{traits::LookupError, MultiAddress};

use super::*;

/// The EVM-aware address lookup for [`MultiAddress`].
///
/// The [`MultiAddress`] is a type that supports both "native" account ID (as [`MultiAddress::Id`])
/// and, among other things, EVM-like 20-byte addresses (as [`MultiAddress::Address20`]).
///
/// We utilize this property to allow using both "native" and 20-byte addresses in our chain.
/// This type, being part of the Substrate itself, and not part of our custom codebase,
/// should be well supported by the node RPC API clients (like Polkadot.js API/Apps and subxt),
/// so we can expect good compatibility in the existing ecosystem (well, at least this is one of
/// the arguments for using this type in theory - we'll see how it plays out).
pub struct MultiLookup<T>(core::marker::PhantomData<T>);

impl<T> StaticLookup for MultiLookup<T>
where
    T: frame_system::Config + pallet_evm_accounts_mapping::Config,
{
    type Source = MultiAddress<T::AccountId, ()>;

    type Target = T::AccountId;

    fn lookup(s: Self::Source) -> Result<Self::Target, frame_support::error::LookupError> {
        match s {
            // Pass "native" address directly as-is.
            MultiAddress::Id(id) => Ok(id),
            // Map 20-byte address to a native address via ethereum address mapping pallet,
            // if the mapping exists.
            MultiAddress::Address20(ethereum_address) => {
                let ethereum_address = primitives_ethereum::EthereumAddress(ethereum_address);
                pallet_evm_accounts_mapping::Pallet::<T>::accounts(ethereum_address)
                    .ok_or(LookupError)
            }
            _ => Err(LookupError),
        }
    }

    fn unlookup(t: Self::Target) -> Self::Source {
        // We unlookup from a native address, so we can naturally always output a native address.
        // Note: this is only a good idea because we don't do the truncated EVM addresses.
        // If we were doing this truncated magic, we'd want to map those truncated native addresses
        // into their 20-byte form here.
        // Since we always have a real "native" address for each EVM address in our current design -
        // we just simply always prefer to unlookup into the "native" form.
        MultiAddress::Id(t)
    }
}

/// A [`pallet_evm::EnsureAddressOrigin`] implementation that performs the 20-byte address mapping
/// via [`frame_system::Config::Lookup`], requiring that it uses a [`MultiAddress`] as a source
/// and passing the [`MultiAddress::Address20`] as an input.
///
/// This way this implementation does not introduce anything new, but instead just relies on
/// however the lookup is implemented, reducing the complexity of the mental model.
pub struct SystemLookupAddressOrigin<T>(core::marker::PhantomData<T>);

impl<T> pallet_evm::EnsureAddressOrigin<T::RuntimeOrigin> for SystemLookupAddressOrigin<T>
where
    T: frame_system::Config + pallet_evm_accounts_mapping::Config,
    <T as frame_system::Config>::Lookup: StaticLookup<Source = MultiAddress<T::AccountId, ()>>,
{
    type Success = T::AccountId;

    fn try_address_origin(
        address: &H160,
        origin: T::RuntimeOrigin,
    ) -> Result<Self::Success, T::RuntimeOrigin> {
        <T::Lookup as StaticLookup>::lookup(MultiAddress::Address20(address.0)).map_err(|_| origin)
    }
}

/// A [`pallet_evm::AddressMapping`] implementation that performs the 20-byte address mapping
/// via [`frame_system::Config::Lookup`], requiring that it uses a [`MultiAddress`] as a source
/// and passing the [`MultiAddress::Address20`] as an input.
///
/// This way this implementation does not introduce anything new, but instead just relies on
/// however the lookup is implemented, reducing the complexity of the mental model.
///
/// ## Panics
///
/// Panics when the lookup fails.
/// This is a temporary solution for now as the EVM internal address mapping interface is
/// infallible. We will need to make it fallible to land a proper support.
pub struct SystemLookupAddressMapping<T>(core::marker::PhantomData<T>);

impl<T> pallet_evm::AddressMapping<T::AccountId> for SystemLookupAddressMapping<T>
where
    T: frame_system::Config + pallet_evm_accounts_mapping::Config,
    <T as frame_system::Config>::Lookup: StaticLookup<Source = MultiAddress<T::AccountId, ()>>,
    <T as frame_system::Config>::AccountId: From<AccountId32>,
{
    fn into_account_id(address: H160) -> T::AccountId {
        <T::Lookup as StaticLookup>::lookup(MultiAddress::Address20(address.0)).unwrap_or_else(|_| {
            // This panic can happen in practice, and it is not a bug!
            // If this happens, this means that the lookup has failed, and the address mapping must
            // fail. Unfortunately, the interface that we are implementing is infallible, so we must
            // panic to kill the whole EVM invocation.
            // Ideally we'd just return an error here instead, but the signature of this trait fn
            // does not allow it.
            sp_tracing::warn!(
                %address,
                "lookup failed for evm address {address}; this is not a bug, you are just accessing the unmapped evm address",
            );
            AccountId32::new([0u8; 32]).into()
        })
    }
}
