use bridge_pot_currency_swap::{CurrencySwap, ExistenceRequired};
use sp_runtime::traits::Identity;

use crate::{
    parameter_types, AccountId, Balances, EvmAccountId, EvmBalances, EvmToNativeSwapBridgePot,
    NativeToEvmSwapBridgePot,
};

parameter_types! {
    pub NativeToEvmSwapBridgePotAccountId: AccountId = NativeToEvmSwapBridgePot::account_id();
    pub EvmToNativeSwapBridgePotAccountId: EvmAccountId = EvmToNativeSwapBridgePot::account_id();
}

pub type NativeToEvmOneToOne = CurrencySwap<NativeToEvmOneToOneConfig, ExistenceRequired>;

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

pub type EvmToNativeOneToOne = CurrencySwap<EvmToNativeOneToOneConfig, ExistenceRequired>;

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

pub struct EvmToNativeProxy;

impl primitives_currency_swap_proxy::Config for EvmToNativeProxy {
    type AccountIdFrom = EvmAccountId;
    type AccountIdTo = AccountId;
    type CurrencySwap = EvmToNativeOneToOne;
}
