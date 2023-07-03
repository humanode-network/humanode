use bridge_pot_currency_swap::{CurrencySwap, ExistenceRequired};
use sp_runtime::traits::Identity;

use crate::{
    parameter_types, AccountId, Balances, BalancesPot, EvmAccountId, EvmBalances, EvmBalancesPot,
};

parameter_types! {
    pub BalancesPotAccountId: AccountId = BalancesPot::account_id();
    pub EvmBalancesPotAccountId: EvmAccountId = EvmBalancesPot::account_id();
}

pub type NativeToEvmOneToOne = CurrencySwap<NativeToEvmOneToOneConfig, ExistenceRequired>;

pub struct NativeToEvmOneToOneConfig;

impl bridge_pot_currency_swap::Config for NativeToEvmOneToOneConfig {
    type AccountIdFrom = AccountId;
    type AccountIdTo = EvmAccountId;
    type CurrencyFrom = Balances;
    type CurrencyTo = EvmBalances;
    type BalanceConverter = Identity;
    type PotFrom = BalancesPotAccountId;
    type PotTo = EvmBalancesPotAccountId;
}

pub type EvmToNativeOneToOne = CurrencySwap<EvmToNativeOneToOneConfig, ExistenceRequired>;

pub struct EvmToNativeOneToOneConfig;

impl bridge_pot_currency_swap::Config for EvmToNativeOneToOneConfig {
    type AccountIdFrom = EvmAccountId;
    type AccountIdTo = AccountId;
    type CurrencyFrom = EvmBalances;
    type CurrencyTo = Balances;
    type BalanceConverter = Identity;
    type PotFrom = EvmBalancesPotAccountId;
    type PotTo = BalancesPotAccountId;
}

pub struct EvmToNativeProxy;

impl primitives_currency_swap_proxy::Config for EvmToNativeProxy {
    type AccountIdFrom = EvmAccountId;
    type AccountIdTo = AccountId;
    type CurrencySwap = EvmToNativeOneToOne;
}
