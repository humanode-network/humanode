//! Traits that we use.

use core::marker::PhantomData;

/// An error that can happen at [`FracScale`].
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum FracScaleError {
    /// An overflow occured.
    #[error("overflow")]
    Overflow,
    /// A division by zero occured.
    #[error("division by zero")]
    DivisionByZero,
    /// Convertion from the internal computations type to the value type failed.
    #[error("type conversion")]
    Conversion,
}

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
    ) -> Result<Self::Value, FracScaleError>;
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
    ) -> Result<Self::Value, FracScaleError> {
        let value = (*value).into();
        let nom = (*nom).into();

        let upscaled = value.checked_mul(&nom).ok_or(FracScaleError::Overflow)?;
        if upscaled.is_zero() {
            return Ok(num_traits::Zero::zero());
        }

        let denom = (*denom).into();
        let downscaled = upscaled
            .checked_div(&denom)
            .ok_or(FracScaleError::DivisionByZero)?;
        downscaled
            .try_into()
            .map_err(|_| FracScaleError::Conversion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_frac_scaler_logic_same_size() {
        let max = u8::MAX;
        let tests = [
            // Ok
            // - value bounds
            (0, 1, 1, Ok(0)),
            (max, 1, 1, Ok(max)),
            (0, max, max, Ok(0)),
            // - samples
            (100, 0, 100, Ok(0)),
            (100, 0, 0, Ok(0)),
            (0xff, 1, 1, Ok(0xff)),
            (0xff, 1, 2, Ok(127)),
            (2, 1, 2, Ok(1)),
            (2, 1, 3, Ok(0)),
            // Errors
            // - the denom is zero, we get what we asked for
            (10, 10, 0, Err(FracScaleError::DivisionByZero)),
            // - the 0xff * 0xff > 0xff and we are at u8, so we get an overflow
            (max, max, max, Err(FracScaleError::Overflow)),
            // - the 0xff * 2 > 0xff, with u8 this is an overflow
            (max, 2, 1, Err(FracScaleError::Overflow)),
            // - the 2 * 0xff > 0xff, u8, so overflow again
            (2, max, 1, Err(FracScaleError::Overflow)),
        ];

        for (value, nom, denom, expected) in tests {
            let actual = <SimpleFracScaler<u8, u8, u8>>::frac_scale(&value, &nom, &denom);
            assert_eq!(actual, expected, "u8 {} {} {}", value, nom, denom);
        }
    }

    #[test]
    fn simple_frac_scaler_logic_u8_to_u16() {
        let max = u8::MAX;
        let tests = [
            // Ok
            // - value bounds
            (0, 1, 1, Ok(0)),
            (max, 1, 1, Ok(max)),
            (0, max, max, Ok(0)),
            // - samples
            (100, 0, 100, Ok(0)),
            (100, 0, 0, Ok(0)),
            (0xff, 1, 1, Ok(0xff)),
            (0xff, 1, 2, Ok(127)),
            (2, 1, 2, Ok(1)),
            (2, 1, 3, Ok(0)),
            // - the 0xff * 0xff < 0xffff and we are at u16, so we are good
            (max, max, max, Ok(max)),
            // Errors
            // - the denom is zero, we get what we asked for
            (10, 10, 0, Err(FracScaleError::DivisionByZero)),
            // - the 0xff * 2 > 0xff < 0xffff, with u16 we are good with overflow but fail at conversion
            (max, 2, 1, Err(FracScaleError::Conversion)),
            // - the 2 * 0xff > 0xff < 0xffff, u16, again good with overflow but fail at conversion
            (2, max, 1, Err(FracScaleError::Conversion)),
        ];

        for (value, nom, denom, expected) in tests {
            let actual = <SimpleFracScaler<u16, u8, u8>>::frac_scale(&value, &nom, &denom);
            assert_eq!(actual, expected, "u16 u8 {} {} {}", value, nom, denom);
        }
    }

    #[test]
    fn simple_frac_scaler_logic_biguint_u128_u64() {
        let tests = [
            // Ok
            (u128::MAX, u64::MAX, u64::MAX, Ok(u128::MAX)),
            (0, u64::MAX, u64::MAX, Ok(0)),
            (1, u64::MAX, u64::MAX, Ok(1)),
            (1, u64::MAX / 2, u64::MAX, Ok(0)),
            (1, u64::MAX - 1, u64::MAX, Ok(0)),
            (2, u64::MAX - 1, u64::MAX, Ok(1)),
            (2, u64::MAX, u64::MAX, Ok(2)),
            // Err
            (u128::MAX, u64::MAX, 0, Err(FracScaleError::DivisionByZero)),
            (u128::MAX, u64::MAX, 1, Err(FracScaleError::Conversion)),
            (u128::MAX, 2, 1, Err(FracScaleError::Conversion)),
        ];

        for (value, nom, denom, expected) in tests {
            let actual =
                <SimpleFracScaler<num::BigUint, u128, u64>>::frac_scale(&value, &nom, &denom);
            assert_eq!(
                actual, expected,
                "BigUint u128 u64 {} {} {}",
                value, nom, denom
            );
        }
    }
}
