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
    T: num_traits::CheckedMul + num_traits::CheckedDiv + num_traits::Zero,
    Value: Into<T> + Copy + num_traits::Zero,
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
        let value = (*value).into();
        let nom = (*nom).into();

        let upscaled = value.checked_mul(&nom)?;
        if upscaled.is_zero() {
            return Some(num_traits::Zero::zero());
        }

        let denom = (*denom).into();
        let downscaled = upscaled.checked_div(&denom)?;
        downscaled.try_into().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_frac_scaler() {
        assert_eq!(
            <SimpleFracScaler<u8, u8, u8>>::frac_scale(&0, &100, &100),
            Some(0)
        );
    }
}
