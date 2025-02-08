//! Imbalances implementation.

use frame_support::traits::{SameOrOther, TryDrop};
use sp_std::{cmp::Ordering, mem};

use super::*;

/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been created without any equal and opposite accounting.
#[must_use]
#[derive(RuntimeDebug, PartialEq, Eq)]
pub struct PositiveImbalance<T: Config<I>, I: 'static = ()>(T::Balance);

impl<T: Config<I>, I: 'static> PositiveImbalance<T, I> {
    /// Create a new positive imbalance from a balance.
    pub fn new(amount: T::Balance) -> Self {
        PositiveImbalance(amount)
    }
}

/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been destroyed without any equal and opposite accounting.
#[must_use]
#[derive(RuntimeDebug, PartialEq, Eq)]
pub struct NegativeImbalance<T: Config<I>, I: 'static = ()>(T::Balance);

impl<T: Config<I>, I: 'static> NegativeImbalance<T, I> {
    /// Create a new negative imbalance from a balance.
    pub fn new(amount: T::Balance) -> Self {
        NegativeImbalance(amount)
    }
}

impl<T: Config<I>, I: 'static> TryDrop for PositiveImbalance<T, I> {
    fn try_drop(self) -> result::Result<(), Self> {
        self.drop_zero()
    }
}

impl<T: Config<I>, I: 'static> Default for PositiveImbalance<T, I> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<T: Config<I>, I: 'static> Imbalance<T::Balance> for PositiveImbalance<T, I> {
    type Opposite = NegativeImbalance<T, I>;

    fn zero() -> Self {
        Self(Zero::zero())
    }

    fn drop_zero(self) -> result::Result<(), Self> {
        if self.0.is_zero() {
            Ok(())
        } else {
            Err(self)
        }
    }

    fn split(self, amount: T::Balance) -> (Self, Self) {
        let first = self.0.min(amount);
        let second = self.0 - first;

        mem::forget(self);
        (Self(first), Self(second))
    }

    fn merge(mut self, other: Self) -> Self {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);

        self
    }

    fn subsume(&mut self, other: Self) {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);
    }

    fn offset(self, other: Self::Opposite) -> SameOrOther<Self, Self::Opposite> {
        let (a, b) = (self.0, other.0);
        mem::forget((self, other));

        match a.cmp(&b) {
            Ordering::Greater => SameOrOther::Same(Self(a - b)),
            Ordering::Less => SameOrOther::Other(NegativeImbalance::new(b - a)),
            Ordering::Equal => SameOrOther::None,
        }
    }

    fn peek(&self) -> T::Balance {
        self.0
    }
}

impl<T: Config<I>, I: 'static> TryDrop for NegativeImbalance<T, I> {
    fn try_drop(self) -> result::Result<(), Self> {
        self.drop_zero()
    }
}

impl<T: Config<I>, I: 'static> Default for NegativeImbalance<T, I> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<T: Config<I>, I: 'static> Imbalance<T::Balance> for NegativeImbalance<T, I> {
    type Opposite = PositiveImbalance<T, I>;

    fn zero() -> Self {
        Self(Zero::zero())
    }

    fn drop_zero(self) -> result::Result<(), Self> {
        if self.0.is_zero() {
            Ok(())
        } else {
            Err(self)
        }
    }

    fn split(self, amount: T::Balance) -> (Self, Self) {
        let first = self.0.min(amount);
        let second = self.0 - first;

        mem::forget(self);
        (Self(first), Self(second))
    }

    fn merge(mut self, other: Self) -> Self {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);

        self
    }

    fn subsume(&mut self, other: Self) {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);
    }

    fn offset(self, other: Self::Opposite) -> SameOrOther<Self, Self::Opposite> {
        let (a, b) = (self.0, other.0);
        mem::forget((self, other));

        match a.cmp(&b) {
            Ordering::Greater => SameOrOther::Same(Self(a - b)),
            Ordering::Less => SameOrOther::Other(PositiveImbalance::new(b - a)),
            Ordering::Equal => SameOrOther::None,
        }
    }

    fn peek(&self) -> T::Balance {
        self.0
    }
}

impl<T: Config<I>, I: 'static> Drop for PositiveImbalance<T, I> {
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {
        TotalIssuance::<T, I>::mutate(|v| *v = v.saturating_add(self.0));
    }
}

impl<T: Config<I>, I: 'static> Drop for NegativeImbalance<T, I> {
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {
        TotalIssuance::<T, I>::mutate(|v| *v = v.saturating_sub(self.0));
    }
}
