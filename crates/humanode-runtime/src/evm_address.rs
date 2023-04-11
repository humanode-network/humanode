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
    T: frame_system::Config<AccountId = AccountId32> + pallet_evm_accounts_mapping::Config,
{
    type Source = MultiAddress<T::AccountId, ()>;

    type Target = T::AccountId;

    fn lookup(s: Self::Source) -> Result<Self::Target, frame_support::error::LookupError> {
        match s {
            // Pass "native" address directly as-is.
            MultiAddress::Id(id) => Ok(id),
            // Map 20-byte address to a native address via ethereum address mapping pallet,
            // if the mapping exists.
            // If the mapping does not exist, we still want to proceed without an error and map the
            // address to something. Well, the way we do this now is invoking
            // the [`pallet_evm::HashedAddressMapping::<BlakeTwo256>`].
            MultiAddress::Address20(ethereum_address) => {
                let ethereum_address = primitives_ethereum::EthereumAddress(ethereum_address);
                match pallet_evm_accounts_mapping::Pallet::<T>::accounts(ethereum_address) {
                    Some(mapped) => Ok(mapped),
                    None => {
                        let mapped = wrapped_eth_addr::to_native(&ethereum_address.0);
                        Ok(mapped)
                    }
                }
            }
            _ => Err(LookupError),
        }
    }

    fn unlookup(t: Self::Target) -> Self::Source {
        // We try to detect if the address is, in fact, a wrapped ethereum address.
        // If it is we emit is as 20-byte address.
        // If not - then we simply passthrough the address as native.
        match wrapped_eth_addr::from_native(&t) {
            Some(ethereum_address) => MultiAddress::Address20(ethereum_address),
            None => MultiAddress::Id(t),
        }
    }
}

mod wrapped_eth_addr {
    //! A logic to wrap an Ethereum 20-byte address within the "native" [`AccountId32`].

    use super::*;

    /// Encode the 20-byte address into a "native" address.
    pub fn to_native(address: &[u8; 20]) -> AccountId32 {
        let mut data = [0u8; 32];
        data[0..4].copy_from_slice(b"evm:");
        data[4..24].copy_from_slice(address);
        AccountId32::new(data)
    }

    /// Check that the provided "native" address is the wrapped 20-byte address, and decode it if
    /// it is.
    pub fn from_native(address: &AccountId32) -> Option<[u8; 20]> {
        let data: &[u8; 32] = address.as_ref();
        if &data[0..4] == b"evm:" && data[24..] == [0u8; 8][..] {
            let mut buf = [0u8; 20];
            buf.copy_from_slice(&data[4..24]);
            return Some(buf);
        }
        None
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
/// Takes a fallback to allow tweaking what happens when the lookup fails.
/// This is required because the [`pallet_evm::AddressMapping::into_account_id`] is infallible, but
/// the lookup is not.
pub struct SystemLookupAddressMapping<T, Fallback>(core::marker::PhantomData<(T, Fallback)>);

impl<T, Fallback> pallet_evm::AddressMapping<T::AccountId>
    for SystemLookupAddressMapping<T, Fallback>
where
    T: frame_system::Config + pallet_evm_accounts_mapping::Config,
    <T as frame_system::Config>::Lookup: StaticLookup<Source = MultiAddress<T::AccountId, ()>>,
    <T as frame_system::Config>::AccountId: From<AccountId32>,
    Fallback: pallet_evm::AddressMapping<T::AccountId>,
{
    fn into_account_id(address: H160) -> T::AccountId {
        <T::Lookup as StaticLookup>::lookup(MultiAddress::Address20(address.0))
            .unwrap_or_else(|_| Fallback::into_account_id(address))
    }
}

/// A [`pallet_evm::AddressMapping`] implementation that always panics.
///
/// Usable as a fallback in composition with other address mapping implementations.
pub struct PanicAddressMapping<T>(core::marker::PhantomData<T>);

impl<T> pallet_evm::AddressMapping<T::AccountId> for PanicAddressMapping<T>
where
    T: frame_system::Config,
{
    fn into_account_id(address: H160) -> T::AccountId {
        // This panic can happen in practice, and it is not a bug!
        // If this happens, this means that the lookup has failed, and the address mapping must
        // fail. Unfortunately, the interface that we are implementing is infallible, so we must
        // panic to kill the whole EVM invocation.
        // Ideally we'd just return an error here instead, but the signature of this trait fn
        // does not allow it.
        panic!(
            "lookup failed for evm address {address}; this is not a bug, you are just accessing the unmapped evm address",
        );
    }
}

/// A [`pallet_evm::AddressMapping`] implementation that logs a warining and always returns
/// a default (typically zero) account.
///
/// Doesn't make very much sense, and might even be dangerous to use in production.
///
/// Usable as a fallback in composition with other address mapping implementations.
pub struct StaticAddressMapping<T, Value>(core::marker::PhantomData<(T, Value)>);

impl<T, Value> pallet_evm::AddressMapping<T::AccountId> for StaticAddressMapping<T, Value>
where
    T: frame_system::Config,
    Value: Get<T::AccountId>,
{
    fn into_account_id(address: H160) -> T::AccountId {
        sp_tracing::warn!(
            message = "lookup failed for an evm address",
            %address,
        );
        Value::get()
    }
}
