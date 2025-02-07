//! Account balances logic.

use frame_support::traits::WithdrawReasons;

use super::*;

/// All balance information for an account.
#[derive(
	Encode,
	Decode,
	Clone,
	PartialEq,
	Eq,
	Default,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo
)]
pub struct AccountData<Balance> {
	/// Non-reserved part of the balance. There may still be restrictions on this, but it is the
	/// total pool what may in principle be transferred, reserved and used for tipping.
	///
	/// This is the only balance that matters in terms of most operations on tokens. It
	/// alone is used to determine the balance when in the contract execution environment.
	pub free: Balance,
}

impl<Balance: Copy> AccountData<Balance> {
	/// The total balance in this account.
	pub(crate) fn total(&self) -> Balance {
		self.free
	}
}

/// Simplified reasons for withdrawing balance.
#[derive(
	Encode,
	Decode,
	Clone,
	Copy,
	PartialEq,
	Eq,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo
)]
pub enum Reasons {
	/// Paying system transaction fees.
	Fee = 0,
	/// Any reason other than paying system transaction fees.
	Misc = 1,
	/// Any reason at all.
	All = 2,
}

impl From<WithdrawReasons> for Reasons {
	fn from(r: WithdrawReasons) -> Reasons {
		if r == WithdrawReasons::TRANSACTION_PAYMENT {
			Reasons::Fee
		} else if r.contains(WithdrawReasons::TRANSACTION_PAYMENT) {
			Reasons::All
		} else {
			Reasons::Misc
		}
	}
}
