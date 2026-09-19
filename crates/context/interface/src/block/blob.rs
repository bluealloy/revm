//! Blob (EIP-4844) related functions and types. [`BlobExcessGasAndPrice`] is struct that helps with
//! calculating blob gas price and excess blob gas.
//!
//! See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers).
//!
//!
//! [`BlobExcessGasAndPrice`] is used to store the blob gas price and excess blob gas.s
use primitives::{
    eip4844::{
        BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN, BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE,
        MIN_BLOB_GASPRICE,
    },
    hardfork::SpecId,
    ruint::aliases::U512,
};

/// Structure holding block blob excess gas and it calculates blob fee
///
/// Incorporated as part of the Cancun upgrade via [EIP-4844].
///
/// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlobExcessGasAndPrice {
    /// The excess blob gas of the block
    pub excess_blob_gas: u64,
    /// The calculated blob gas price based on the `excess_blob_gas`
    ///
    /// See [calc_blob_gasprice]
    pub blob_gasprice: u128,
}

impl BlobExcessGasAndPrice {
    /// Creates a new instance by calculating the blob gas price with [`calc_blob_gasprice`].
    ///
    /// `excess_blob_gas` is the excess blob gas of the block, it can be calculated with `calc_excess_blob_gas` function from alloy-eips.
    pub fn new(excess_blob_gas: u64, blob_base_fee_update_fraction: u64) -> Self {
        let blob_gasprice = calc_blob_gasprice(excess_blob_gas, blob_base_fee_update_fraction);
        Self {
            excess_blob_gas,
            blob_gasprice,
        }
    }

    /// Creates a new instance by calculating the blob gas price based on the spec.
    pub fn new_with_spec(excess_blob_gas: u64, spec: SpecId) -> Self {
        Self::new(
            excess_blob_gas,
            if spec.is_enabled_in(SpecId::PRAGUE) {
                BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE
            } else {
                BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN
            },
        )
    }
}

/// Calculates the blob gas price from the header's excess blob gas field.
///
/// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers)
/// (`get_blob_gasprice`).
#[inline]
pub fn calc_blob_gasprice(excess_blob_gas: u64, blob_base_fee_update_fraction: u64) -> u128 {
    fake_exponential(
        MIN_BLOB_GASPRICE,
        excess_blob_gas,
        blob_base_fee_update_fraction,
    )
}

/// Calculates the base fee per blob gas. Calls [`calc_blob_gasprice`] internally.
/// Name of the function is aligned with EIP-4844 spec.
pub fn get_base_fee_per_blob_gas(excess_blob_gas: u64, blob_base_fee_update_fraction: u64) -> u128 {
    calc_blob_gasprice(excess_blob_gas, blob_base_fee_update_fraction)
}

/// Approximates `factor * e ** (numerator / denominator)` using Taylor expansion.
///
/// This is used to calculate the blob price.
///
/// The result is exact for every value that fits in a `u128`; if the mathematical
/// result does not fit, the function saturates to `u128::MAX` instead of panicking
/// or wrapping around.
///
/// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers)
/// (`fake_exponential`).
///
/// # Panics
///
/// Panics if `denominator` is zero.
#[inline]
pub fn fake_exponential(factor: u64, numerator: u64, denominator: u64) -> u128 {
    assert_ne!(denominator, 0, "attempt to divide by zero");
    // Fast path: `u128` intermediates are enough for any excess blob gas a real
    // chain can reach (the series only overflows above ~192M excess blob gas).
    if let Some(output) = fake_exponential_u128(factor, numerator, denominator) {
        return output;
    }
    fake_exponential_u512(factor, numerator, denominator)
}

/// Evaluates the series with `u128` intermediates.
///
/// Returns `None` if any intermediate value overflows.
#[inline]
fn fake_exponential_u128(factor: u64, numerator: u64, denominator: u64) -> Option<u128> {
    let numerator = numerator as u128;
    let denominator = denominator as u128;

    let mut i = 1;
    let mut output: u128 = 0;
    let mut numerator_accum = factor as u128 * denominator;
    while numerator_accum > 0 {
        output = output.checked_add(numerator_accum)?;
        // Denominator is asserted as not zero by the caller.
        numerator_accum = numerator_accum.checked_mul(numerator)? / (denominator * i);
        i += 1;
    }
    Some(output / denominator)
}

/// Evaluates the series with 512-bit intermediates, so the result stays exact as
/// long as it fits in a `u128`, and saturates to `u128::MAX` beyond that.
#[cold]
fn fake_exponential_u512(factor: u64, numerator: u64, denominator: u64) -> u128 {
    let numerator = U512::from(numerator);
    let denominator = U512::from(denominator);

    let mut i = U512::from(1);
    let mut output = U512::ZERO;
    let mut numerator_accum = U512::from(factor) * denominator;
    while !numerator_accum.is_zero() {
        let Some(next_output) = output.checked_add(numerator_accum) else {
            return u128::MAX;
        };
        output = next_output;

        let Some(next_accum) = numerator_accum.checked_mul(numerator) else {
            return u128::MAX;
        };
        numerator_accum = next_accum / (denominator * i);
        i += U512::from(1);
    }

    (output / denominator).try_into().unwrap_or(u128::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN;

    // https://github.com/ethereum/go-ethereum/blob/28857080d732857030eda80c69b9ba2c8926f221/consensus/misc/eip4844/eip4844_test.go#L78
    #[test]
    fn fake_exp() {
        for t @ &(factor, numerator, denominator, expected) in &[
            (1u64, 0u64, 1u64, 1u128),
            (38493, 0, 1000, 38493),
            (0, 1234, 2345, 0),
            (1, 2, 1, 6), // approximate 7.389
            (1, 4, 2, 6),
            (1, 3, 1, 16), // approximate 20.09
            (1, 6, 2, 18),
            (1, 4, 1, 49), // approximate 54.60
            (1, 8, 2, 50),
            (10, 8, 2, 542), // approximate 540.598
            (11, 8, 2, 596), // approximate 600.58
            (1, 5, 1, 136),  // approximate 148.4
            (1, 5, 2, 11),   // approximate 12.18
            (2, 5, 2, 23),   // approximate 24.36
            (1, 50000000, 2225652, 5709098764),
            (1, 380928, BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN, 1),
        ] {
            let actual = fake_exponential(factor, numerator, denominator);
            assert_eq!(actual, expected, "test: {t:?}");
        }
    }

    /// Large `excess_blob_gas` values used to overflow the `u128` intermediate
    /// `numerator_accum * numerator` product (panic in debug, wrap-around and a
    /// very long loop in release) long before the final result stopped fitting
    /// in a `u128`. The series must stay exact for every representable result
    /// and saturate to `u128::MAX` beyond that.
    #[test]
    fn fake_exp_large_inputs_are_exact_or_saturate() {
        // Reference values computed with arbitrary-precision integers from the
        // EIP-4844 Python `fake_exponential`.
        for (numerator, denominator, expected) in [
            // last value before the old u128 intermediate product overflowed
            (
                192_204_552,
                BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN,
                10_079_293_834_132_079_738_693_097u128,
            ),
            // first value that overflowed the old u128 intermediate product
            (
                192_204_553,
                BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN,
                10_079_296_854_086_811_361_005_191,
            ),
            (
                250_000_000,
                BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN,
                332_584_186_920_530_080_845_367_541_284_883,
            ),
            // largest value whose result still fits in a u128
            (
                296_199_157,
                BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN,
                340_282_290_560_605_955_201_531_563_932_614_965_989,
            ),
            (
                444_298_780,
                BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE,
                340_282_299_990_988_936_369_579_685_675_873_524_818,
            ),
        ] {
            assert_eq!(
                fake_exponential(MIN_BLOB_GASPRICE, numerator, denominator),
                expected,
                "numerator={numerator} denominator={denominator}"
            );
        }

        // Results past u128::MAX saturate instead of panicking or wrapping.
        for (numerator, denominator) in [
            (296_199_158, BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN),
            (444_298_781, BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE),
            (u64::MAX, BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN),
            (u64::MAX, 1),
        ] {
            assert_eq!(
                fake_exponential(MIN_BLOB_GASPRICE, numerator, denominator),
                u128::MAX,
                "numerator={numerator} denominator={denominator}"
            );
        }
    }
}
