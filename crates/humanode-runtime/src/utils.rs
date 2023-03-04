use frame_support::traits::Get;
use sp_runtime::generic::Era;

/// Returns longest possible era for the number of block hashes that are cached by the runtime.
///
/// Take the biggest period possible, considering the number of cached block hashes.
/// In the case of overflow, we pass default (`0`) and let `Era::mortal`
/// clamp the value to the lower bound
pub fn longest_era_for_block_hashes<BlockHashCount: Get<u32>>(current_block: u64) -> Era {
    let period: u64 = BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or_default()
        .into();
    Era::mortal(period, current_block)
}
