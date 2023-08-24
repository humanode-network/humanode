use pallet_evm::{Precompile, PrecompileHandle, PrecompileResult, PrecompileSet};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use precompile_bioauth::Bioauth;
use precompile_currency_swap::CurrencySwap;
use precompile_evm_accounts_mapping::EvmAccountsMapping;
use precompile_evm_balances_erc20::EvmBalancesErc20;
use sp_core::{H160, U256};
use sp_std::marker::PhantomData;

use crate::{currency_swap, AccountId, ConstU64, EvmAccountId};

pub struct FrontierPrecompiles<R>(PhantomData<R>);

impl<R> Default for FrontierPrecompiles<R> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<R> FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
{
    pub fn used_addresses() -> sp_std::vec::Vec<H160> {
        sp_std::vec![1_u64, 2, 3, 4, 5, 1024, 1025, 2048, 2049, 2050, 2304]
            .into_iter()
            .map(hash)
            .collect()
    }
}

impl<R> PrecompileSet for FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
    R: pallet_bioauth::Config,
    R: pallet_evm_accounts_mapping::Config,
    R: pallet_evm_balances::Config,
    <R as pallet_evm_balances::Config>::Balance: Into<U256>,
    <R as pallet_evm_balances::Config>::AccountId: From<H160>,
    R::ValidatorPublicKey: for<'a> TryFrom<&'a [u8]> + Eq,
{
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
        match handle.code_address() {
            // Ethereum precompiles :
            a if a == hash(1) => Some(ECRecover::execute(handle)),
            a if a == hash(2) => Some(Sha256::execute(handle)),
            a if a == hash(3) => Some(Ripemd160::execute(handle)),
            a if a == hash(4) => Some(Identity::execute(handle)),
            a if a == hash(5) => Some(Modexp::execute(handle)),
            // Non-Frontier specific nor Ethereum precompiles :
            a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
            a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
            // Humanode precompiles:
            a if a == hash(2048) => Some(Bioauth::<R>::execute(handle)),
            a if a == hash(2049) => Some(EvmAccountsMapping::<R>::execute(handle)),
            a if a == hash(2050) => Some(EvmBalancesErc20::<R, ConstU64<200>>::execute(handle)),
            a if a == hash(2304) => Some(CurrencySwap::<
                currency_swap::EvmToNativeOneToOne,
                EvmAccountId,
                AccountId,
                // TODO(#697): implement proper dynamic gas cost estimation.
                ConstU64<200>,
            >::execute(handle)),
            // Fallback
            _ => None,
        }
    }

    fn is_precompile(&self, address: H160) -> bool {
        Self::used_addresses().contains(&address)
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}
