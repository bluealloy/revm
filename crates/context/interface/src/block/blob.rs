//! Blob (EIP-4844) related functions and types. [`BlobExcessGasAndPrice`] is struct that helps with
//! calculating blob gas price and excess blob gas.
//!
//! See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers).
//!
//! [`calc_blob_gasprice`] and [`calc_excess_blob_gas`] are used to calculate the blob gas price and
//! excess blob gas.
//!
//! [`BlobExcessGasAndPrice`] is used to store the blob gas price and excess blob gas.s
use primitives::{
    eip4844::{
        BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN, BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE, GAS_PER_BLOB,
        MIN_BLOB_GASPRICE,
    },
    eip7918,
    hardfork::SpecId,
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
    /// `excess_blob_gas` is the excess blob gas of the block, it can be calculated with [`calc_excess_blob_gas`].
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

    /// Calculate this block excess gas and price from the parent excess gas and gas used
    /// and the target blob gas per block.
    ///
    /// These fields will be used to calculate `excess_blob_gas` with [`calc_excess_blob_gas`] func.
    #[deprecated(
        note = "Use `calc_excess_blob_gas` and `BlobExcessGasAndPrice::new` instead. Only works for forks before Osaka."
    )]
    pub fn from_parent_and_target(
        parent_excess_blob_gas: u64,
        parent_blob_gas_used: u64,
        parent_target_blob_gas_per_block: u64,
        blob_base_fee_update_fraction: u64,
    ) -> Self {
        Self::new(
            calc_excess_blob_gas(
                parent_excess_blob_gas,
                parent_blob_gas_used,
                parent_target_blob_gas_per_block,
            ),
            blob_base_fee_update_fraction,
        )
    }
}

/// Calculates the `excess_blob_gas` from the parent header's `blob_gas_used` and `excess_blob_gas`.
/// Uses [`calc_excess_blob_gas_osaka`] internally.
#[inline]
pub fn calc_excess_blob_gas(
    parent_excess_blob_gas: u64,
    parent_blob_gas_used: u64,
    parent_target_blob_gas_per_block: u64,
) -> u64 {
    calc_excess_blob_gas_osaka(
        parent_excess_blob_gas,
        parent_blob_gas_used,
        parent_target_blob_gas_per_block,
        false,
        0,
        0,
        0,
        0,
        0,
    )
}

/// Calculates the `excess_blob_gas` from the parent header's `blob_gas_used` and `excess_blob_gas`.
///
/// See also [the EIP-4844 helpers]<https://eips.ethereum.org/EIPS/eip-4844#helpers>
/// (`calc_excess_blob_gas`).
///
/// [EIP-7918: Blob base fee bounded by execution cost](https://eips.ethereum.org/EIPS/eip-7918)
///
/// `blob_base_cost` is introduced in EIP-7918 in Osaka fork. All fields after is_osaka input are not needed before Osaka.
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn calc_excess_blob_gas_osaka(
    parent_excess_blob_gas: u64,
    parent_blob_gas_used: u64,
    parent_target_blob_gas_per_block: u64,
    is_osaka: bool,
    parent_base_fee_per_gas: u64,
    parent_blob_base_fee_per_gas: u64,
    parent_blob_base_fee_update_fraction: u64,
    max_blob_count: u64,
    target_blob_count: u64,
) -> u64 {
    let excess_and_used = parent_excess_blob_gas.saturating_add(parent_blob_gas_used);

    if is_osaka {
        if excess_and_used < parent_target_blob_gas_per_block {
            return 0;
        }

        if (eip7918::BLOB_BASE_COST.saturating_mul(parent_base_fee_per_gas) as u128)
            > (GAS_PER_BLOB as u128).saturating_mul(get_base_fee_per_blob_gas(
                parent_blob_base_fee_per_gas,
                parent_blob_base_fee_update_fraction,
            ))
        {
            return excess_and_used.saturating_add(
                parent_blob_gas_used.saturating_mul(max_blob_count - target_blob_count)
                    / max_blob_count,
            );
        }
    }

    excess_and_used.saturating_sub(parent_target_blob_gas_per_block)
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
#[inline]
pub fn fake_exponential(factor: u64, numerator: u64, denominator: u64) -> u128 {
    assert_ne!(denominator, 0, "attempt to divide by zero");
    let factor = factor as u128;
    let numerator = numerator as u128;
    let denominator = denominator as u128;

    if denominator == 0 {
        return 0;
    }

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
    use primitives::eip4844::{
        self, BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN, GAS_PER_BLOB,
        TARGET_BLOB_GAS_PER_BLOCK_CANCUN as TARGET_BLOB_GAS_PER_BLOCK,
    };

    // https://github.com/ethereum/go-ethereum/blob/28857080d732857030eda80c69b9ba2c8926f221/consensus/misc/eip4844/eip4844_test.go#L27
    #[test]
    fn test_calc_excess_blob_gas() {
        for t @ &(excess, blobs, expected) in &[
            // The excess blob gas should not increase from zero if the used blob
            // slots are below - or equal - to the target.
            (0, 0, 0),
            (0, 1, 0),
            (0, TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB, 0),
            // If the target blob gas is exceeded, the excessBlobGas should increase
            // by however much it was overshot
            (
                0,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) + 1,
                GAS_PER_BLOB,
            ),
            (
                1,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) + 1,
                GAS_PER_BLOB + 1,
            ),
            (
                1,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) + 2,
                2 * GAS_PER_BLOB + 1,
            ),
            // The excess blob gas should decrease by however much the target was
            // under-shot, capped at zero.
            (
                TARGET_BLOB_GAS_PER_BLOCK,
                TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB,
                TARGET_BLOB_GAS_PER_BLOCK,
            ),
            (
                TARGET_BLOB_GAS_PER_BLOCK,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) - 1,
                TARGET_BLOB_GAS_PER_BLOCK - GAS_PER_BLOB,
            ),
            (
                TARGET_BLOB_GAS_PER_BLOCK,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) - 2,
                TARGET_BLOB_GAS_PER_BLOCK - (2 * GAS_PER_BLOB),
            ),
            (
                GAS_PER_BLOB - 1,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) - 1,
                0,
            ),
        ] {
            let actual = calc_excess_blob_gas(
                excess,
                blobs * GAS_PER_BLOB,
                eip4844::TARGET_BLOB_GAS_PER_BLOCK_CANCUN,
            );
            assert_eq!(actual, expected, "test: {t:?}");
        }
    }

    // https://github.com/ethereum/go-ethereum/blob/28857080d732857030eda80c69b9ba2c8926f221/consensus/misc/eip4844/eip4844_test.go#L60
    #[test]
    fn test_calc_blob_fee_cancun() {
        let blob_fee_vectors = &[
            (0, 1),
            (2314057, 1),
            (2314058, 2),
            (10 * 1024 * 1024, 23),
            // `calc_blob_gasprice` approximates `e ** (excess_blob_gas / BLOB_BASE_FEE_UPDATE_FRACTION)` using Taylor expansion
            //
            // to roughly find where boundaries will be hit:
            // 2 ** bits = e ** (excess_blob_gas / BLOB_BASE_FEE_UPDATE_FRACTION)
            // excess_blob_gas = ln(2 ** bits) * BLOB_BASE_FEE_UPDATE_FRACTION
            (148099578, 18446739238971471609), // output is just below the overflow
            (148099579, 18446744762204311910), // output is just after the overflow
            (161087488, 902580055246494526580),
        ];

        for &(excess, expected) in blob_fee_vectors {
            let actual = calc_blob_gasprice(excess, BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN);
            assert_eq!(actual, expected, "test: {excess}");
        }
    }

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
