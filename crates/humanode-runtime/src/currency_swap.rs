use pallet_bridge_pot_currency_swap::ExistenceRequired;

use crate::{
    AccountId, EvmAccountId, EvmToNativeSwapBridge, EvmToNativeSwapBridgePot, FeesPot,
    NativeToEvmSwapBridge, TreasuryPot,
};

pub type NativeToEvmOneToOne =
    pallet_bridge_pot_currency_swap::CurrencySwap<NativeToEvmSwapBridge, ExistenceRequired>;

pub type EvmToNativeOneToOne =
    pallet_bridge_pot_currency_swap::CurrencySwap<EvmToNativeSwapBridge, ExistenceRequired>;

pub struct GenesisVerifier;

impl pallet_bridge_pot_currency_swap::GenesisVerifier for GenesisVerifier {
    fn verify() -> bool {
        true
    }
}

pub struct EvmToNativeProxyConfig;

impl primitives_currency_swap_proxy::Config for EvmToNativeProxyConfig {
    type AccountIdFrom = EvmAccountId;
    type AccountIdTo = AccountId;
    type CurrencySwap = EvmToNativeOneToOne;
}

pub type FeesPotProxy = primitives_currency_swap_proxy::SwapUnbalanced<
    EvmToNativeProxyConfig,
    FeesPot,
    EvmToNativeSwapBridgePot,
>;

pub type TreasuryPotProxy = primitives_currency_swap_proxy::SwapUnbalanced<
    EvmToNativeProxyConfig,
    TreasuryPot,
    EvmToNativeSwapBridgePot,
>;
