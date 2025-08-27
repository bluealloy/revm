//! Blob (EIP-4844) related functions and types. [`BlobExcessGasAndPrice`] is struct that helps with
//! calculating blob gas price and excess blob gas.
//!
//! See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers).
//!
//!
//! [`BlobExcessGasAndPrice`] is used to store the blob gas price and excess blob gas.s
use primitives::eip4844::MIN_BLOB_GASPRICE;

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
/// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers)
/// (`fake_exponential`).
///
/// # Panics
///
/// This function panics if `denominator` is zero.
#[inline]
pub fn fake_exponential(factor: u64, numerator: u64, denominator: u64) -> u128 {
    assert_ne!(denominator, 0, "attempt to divide by zero");
    let factor = factor as u128;
    let numerator = numerator as u128;
    let denominator = denominator as u128;

    let mut i = 1;
    let mut output = 0;
    let mut numerator_accum = factor * denominator;
    while numerator_accum > 0 {
        output += numerator_accum;

        // Denominator is asserted as not zero at the start of the function.
        numerator_accum = (numerator_accum * numerator) / (denominator * i);
        i += 1;
    }
    output / denominator
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
}
