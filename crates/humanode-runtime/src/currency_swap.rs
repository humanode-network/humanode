use bridge_pot_currency_swap::ExistenceRequired;
use sp_runtime::traits::Identity;

use crate::{
    parameter_types, AccountId, Balances, EvmAccountId, EvmBalances, EvmToNativeSwapBridgePot,
    NativeToEvmSwapBridgePot, PotInstanceEvmToNativeSwapBridge, PotInstanceFees,
    PotInstanceTreasury, Runtime,
};

parameter_types! {
    pub NativeToEvmSwapBridgePotAccountId: AccountId = NativeToEvmSwapBridgePot::account_id();
    pub EvmToNativeSwapBridgePotAccountId: EvmAccountId = EvmToNativeSwapBridgePot::account_id();
}

pub type NativeToEvmOneToOne =
    bridge_pot_currency_swap::CurrencySwap<NativeToEvmOneToOneConfig, ExistenceRequired>;

pub struct NativeToEvmOneToOneConfig;

impl bridge_pot_currency_swap::Config for NativeToEvmOneToOneConfig {
    type AccountIdFrom = AccountId;
    type AccountIdTo = EvmAccountId;
    type CurrencyFrom = Balances;
    type CurrencyTo = EvmBalances;
    type BalanceConverter = Identity;
    type PotFrom = NativeToEvmSwapBridgePotAccountId;
    type PotTo = EvmToNativeSwapBridgePotAccountId;
}

pub type EvmToNativeOneToOne =
    bridge_pot_currency_swap::CurrencySwap<EvmToNativeOneToOneConfig, ExistenceRequired>;

pub struct EvmToNativeOneToOneConfig;

impl bridge_pot_currency_swap::Config for EvmToNativeOneToOneConfig {
    type AccountIdFrom = EvmAccountId;
    type AccountIdTo = AccountId;
    type CurrencyFrom = EvmBalances;
    type CurrencyTo = Balances;
    type BalanceConverter = Identity;
    type PotFrom = EvmToNativeSwapBridgePotAccountId;
    type PotTo = NativeToEvmSwapBridgePotAccountId;
}

pub struct EvmToNativeProxyConfig;

impl primitives_currency_swap_proxy::Config for EvmToNativeProxyConfig {
    type AccountIdFrom = EvmAccountId;
    type AccountIdTo = AccountId;
    type CurrencySwap = EvmToNativeOneToOne;
}

pub type FeesPotProxy = primitives_currency_swap_proxy::SwapUnbalanced<
    EvmToNativeProxyConfig,
    pallet_pot::DepositUnbalancedCurrency<Runtime, PotInstanceFees>,
    pallet_pot::DepositUnbalancedCurrency<Runtime, PotInstanceEvmToNativeSwapBridge>,
>;

pub type TreasuryPotProxy = primitives_currency_swap_proxy::SwapUnbalanced<
    EvmToNativeProxyConfig,
    pallet_pot::DepositUnbalancedCurrency<Runtime, PotInstanceTreasury>,
    pallet_pot::DepositUnbalancedCurrency<Runtime, PotInstanceEvmToNativeSwapBridge>,
>;
