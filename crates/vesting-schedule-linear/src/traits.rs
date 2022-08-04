//! Traits that we use.

use core::marker::PhantomData;

/// Fractional scaler.
///
/// Effectively represent multiplication of the value to a fraction operation: x * (a/b).
pub trait FracScale {
    /// The value type to scale.
    type Value;
    /// The type used for the fraction nominator and denominator.
    type FracPart;

    /// Compute `value` * (`nom` / `denom`).
    fn frac_scale(
        value: &Self::Value,
        nom: &Self::FracPart,
        denom: &Self::FracPart,
    ) -> Option<Self::Value>;
}

/// Not super precise or safe, but generic scaler.
pub struct SimpleFracScaler<T, Value, FracPart>(PhantomData<(T, Value, FracPart)>);

impl<T, Value, FracPart> FracScale for SimpleFracScaler<T, Value, FracPart>
where
    T: num_traits::CheckedMul + num_traits::CheckedDiv,
    Value: Into<T> + Copy,
    FracPart: Into<T> + Copy,
    T: TryInto<Value>,
{
    type Value = Value;
    type FracPart = FracPart;

    fn frac_scale(
        value: &Self::Value,
        nom: &Self::FracPart,
        denom: &Self::FracPart,
    ) -> Option<Self::Value> {
        let x = (*value).into();
        let x = x.checked_mul(&(*nom).into())?;
        let x = x.checked_div(&(*denom).into())?;
        x.try_into().ok()
    }
}
