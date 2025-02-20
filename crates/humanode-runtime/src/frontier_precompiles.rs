use frame_support::traits::Currency;
use pallet_evm::{
    IsPrecompileResult, Precompile, PrecompileHandle, PrecompileResult, PrecompileSet,
};
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use precompile_bioauth::Bioauth;
use precompile_bls12381::{
    Bls12381G1Add, Bls12381G1Mul, Bls12381G1MultiExp, Bls12381G2Add, Bls12381G2Mul,
    Bls12381G2MultiExp, Bls12381MapG1, Bls12381MapG2, Bls12381Pairing,
};
use precompile_currency_swap::CurrencySwap;
use precompile_evm_accounts_mapping::EvmAccountsMapping;
use precompile_native_currency::NativeCurrency;
use precompile_utils::EvmData;
use sp_core::{H160, U256};
use sp_std::marker::PhantomData;

use crate::{currency_swap, AccountId, ConstU64, EvmAccountId};

/// A set of constant values used to indicate precompiles.
pub mod precompiles_constants {
    /// `ECRecover` precompile constant.
    pub const EC_RECOVERY: u64 = 1;
    /// `Sha256` precompile constant.
    pub const SHA_256: u64 = 2;
    /// `Ripemd160` precompile constant.
    pub const RIPEMD_160: u64 = 3;
    /// `Identity` precompile constant.
    pub const IDENTITY: u64 = 4;
    /// `Modexp` precompile constant.
    pub const MODEXP: u64 = 5;

    /// `Bn128Add` precompile constant.
    pub const BN128_ADD: u64 = 6;
    /// `Bn128Mul` precompile constant.
    pub const BN128_MUL: u64 = 7;
    /// `Bn128Pairing` precompile constant.
    pub const BN128_PAIRING: u64 = 8;

    /// `Blake2F` precompile constant.
    pub const BLAKE2F: u64 = 9;

    /// `Bls12381G1Add` precompile constant.
    pub const BLS12381_G1_ADD: u64 = 11;
    /// `Bls12381G1Mul` precompile constant.
    pub const BLS12381_G1_MUL: u64 = 12;
    /// `Bls12381G1MultiExp` precompile constant.
    pub const BLS12381_G1_MULTI_EXP: u64 = 13;
    /// `Bls12381G2Add` precompile constant.
    pub const BLS12381_G2_ADD: u64 = 14;
    /// `Bls12381G2Mul` precompile constant.
    pub const BLS12381_G2_MUL: u64 = 15;
    /// `Bls12381G2MultiExp` precompile constant.
    pub const BLS12381_G2_MULTI_EXP: u64 = 16;
    /// `Bls12381Pairing` precompile constant.
    pub const BLS12381_PAIRING: u64 = 17;
    /// `Bls12381MapG1` precompile constant.
    pub const BLS12381_MAP_G1: u64 = 18;
    /// `Bls12381MapG2` precompile constant.
    pub const BLS12381_MAP_G2: u64 = 19;

    /// `Sha3FIPS256` precompile constant.
    pub const SHA_3_FIPS256: u64 = 1024;
    /// `ECRecoverPublicKey` precompile constant.
    pub const EC_RECOVER_PUBLIC_KEY: u64 = 1025;

    /// `Bioauth` precompile constant.
    pub const BIOAUTH: u64 = 2048;
    /// `EvmAccountsMapping` precompile constant.
    pub const EVM_ACCOUNTS_MAPPING: u64 = 2049;
    /// `NativeCurrency` precompile constant.
    pub const NATIVE_CURRENCY: u64 = 2050;
    /// `CurrencySwap` precompile constant.
    pub const CURRENCY_SWAP: u64 = 2304;
}

use precompiles_constants::*;

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
        sp_std::vec![
            EC_RECOVERY,
            SHA_256,
            RIPEMD_160,
            IDENTITY,
            MODEXP,
            BN128_ADD,
            BN128_MUL,
            BN128_PAIRING,
            BLAKE2F,
            BLS12381_G1_ADD,
            BLS12381_G1_MUL,
            BLS12381_G1_MULTI_EXP,
            BLS12381_G2_ADD,
            BLS12381_G2_MUL,
            BLS12381_G2_MULTI_EXP,
            BLS12381_PAIRING,
            BLS12381_MAP_G1,
            BLS12381_MAP_G2,
            SHA_3_FIPS256,
            EC_RECOVER_PUBLIC_KEY,
            BIOAUTH,
            EVM_ACCOUNTS_MAPPING,
            NATIVE_CURRENCY,
            CURRENCY_SWAP
        ]
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
    R: pallet_erc20_support::Config,
    <R as pallet_erc20_support::Config>::AccountId: From<H160>,
    <<R as pallet_erc20_support::Config>::Currency as Currency<
        <R as pallet_erc20_support::Config>::AccountId,
    >>::Balance: Into<U256> + TryFrom<U256>,
    <R as pallet_erc20_support::Config>::Allowance: TryFrom<U256> + EvmData,
    R::ValidatorPublicKey: for<'a> TryFrom<&'a [u8]> + Eq,
{
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
        match handle.code_address() {
            // Ethereum precompiles :
            a if a == hash(EC_RECOVERY) => Some(ECRecover::execute(handle)),
            a if a == hash(SHA_256) => Some(Sha256::execute(handle)),
            a if a == hash(RIPEMD_160) => Some(Ripemd160::execute(handle)),
            a if a == hash(IDENTITY) => Some(Identity::execute(handle)),
            a if a == hash(MODEXP) => Some(Modexp::execute(handle)),
            // Bn128 precompiles:
            a if a == hash(BN128_ADD) => Some(Bn128Add::execute(handle)),
            a if a == hash(BN128_MUL) => Some(Bn128Mul::execute(handle)),
            a if a == hash(BN128_PAIRING) => Some(Bn128Pairing::execute(handle)),
            // Blake-2 precompiles:
            a if a == hash(BLAKE2F) => Some(Blake2F::execute(handle)),
            // BLS12-381 precompiles:
            a if a == hash(BLS12381_G1_ADD) => Some(Bls12381G1Add::execute(handle)),
            a if a == hash(BLS12381_G1_MUL) => Some(Bls12381G1Mul::execute(handle)),
            a if a == hash(BLS12381_G1_MULTI_EXP) => Some(Bls12381G1MultiExp::execute(handle)),
            a if a == hash(BLS12381_G2_ADD) => Some(Bls12381G2Add::execute(handle)),
            a if a == hash(BLS12381_G2_MUL) => Some(Bls12381G2Mul::execute(handle)),
            a if a == hash(BLS12381_G2_MULTI_EXP) => Some(Bls12381G2MultiExp::execute(handle)),
            a if a == hash(BLS12381_PAIRING) => Some(Bls12381Pairing::execute(handle)),
            a if a == hash(BLS12381_MAP_G1) => Some(Bls12381MapG1::execute(handle)),
            a if a == hash(BLS12381_MAP_G2) => Some(Bls12381MapG2::execute(handle)),
            // Non-Frontier specific nor Ethereum precompiles :
            a if a == hash(SHA_3_FIPS256) => Some(Sha3FIPS256::execute(handle)),
            a if a == hash(EC_RECOVER_PUBLIC_KEY) => Some(ECRecoverPublicKey::execute(handle)),
            // Humanode precompiles:
            a if a == hash(BIOAUTH) => Some(Bioauth::<R>::execute(handle)),
            a if a == hash(EVM_ACCOUNTS_MAPPING) => Some(EvmAccountsMapping::<R>::execute(handle)),
            a if a == hash(NATIVE_CURRENCY) => {
                Some(NativeCurrency::<R, ConstU64<200>>::execute(handle))
            }
            a if a == hash(CURRENCY_SWAP) => {
                Some(CurrencySwap::<
                    currency_swap::EvmToNativeOneToOne,
                    EvmAccountId,
                    AccountId,
                    // TODO(#697): implement proper dynamic gas cost estimation.
                    ConstU64<200>,
                >::execute(handle))
            }
            // Fallback
            _ => None,
        }
    }

    fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
        IsPrecompileResult::Answer {
            is_precompile: Self::used_addresses().contains(&address),
            extra_cost: 0,
        }
    }
}

pub struct FrontierPrecompilesAddresses<R>(PhantomData<R>);

impl<R: pallet_evm::Config> sp_core::TypedGet for FrontierPrecompilesAddresses<R> {
    type Type = sp_std::vec::Vec<H160>;

    fn get() -> Self::Type {
        FrontierPrecompiles::<R>::used_addresses()
    }
}

pub fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}
