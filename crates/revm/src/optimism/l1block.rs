use crate::primitives::{address, db::Database, Address, SpecId, U256};
use core::ops::Mul;

const ZERO_BYTE_COST: u64 = 4;
const NON_ZERO_BYTE_COST: u64 = 16;

/// The two 4-byte Ecotone fee scalar values are packed into the same storage slot as the 8-byte sequence number.
/// Byte offset within the storage slot of the 4-byte baseFeeScalar attribute.
const BASE_FEE_SCALAR_OFFSET: usize = 16;
/// The two 4-byte Ecotone fee scalar values are packed into the same storage slot as the 8-byte sequence number.
/// Byte offset within the storage slot of the 4-byte blobBaseFeeScalar attribute.
const BLOB_BASE_FEE_SCALAR_OFFSET: usize = 20;

const L1_BASE_FEE_SLOT: U256 = U256::from_limbs([1u64, 0, 0, 0]);
const L1_OVERHEAD_SLOT: U256 = U256::from_limbs([5u64, 0, 0, 0]);
const L1_SCALAR_SLOT: U256 = U256::from_limbs([6u64, 0, 0, 0]);

/// [ECOTONE_L1_BLOB_BASE_FEE_SLOT] was added in the Ecotone upgrade and stores the L1 blobBaseFee attribute.
const ECOTONE_L1_BLOB_BASE_FEE_SLOT: U256 = U256::from_limbs([7u64, 0, 0, 0]);

/// As of the ecotone upgrade, this storage slot stores the 32-bit basefeeScalar and blobBaseFeeScalar attributes at
/// offsets [BASE_FEE_SCALAR_OFFSET] and [BLOB_BASE_FEE_SCALAR_OFFSET] respectively.
const ECOTONE_L1_FEE_SCALARS_SLOT: U256 = U256::from_limbs([3u64, 0, 0, 0]);

/// An empty 64-bit set of scalar values.
const EMPTY_SCALARS: [u8; 8] = [0u8; 8];

/// The address of L1 fee recipient.
pub const L1_FEE_RECIPIENT: Address = address!("420000000000000000000000000000000000001A");

/// The address of the base fee recipient.
pub const BASE_FEE_RECIPIENT: Address = address!("4200000000000000000000000000000000000019");

/// The address of the L1Block contract.
pub const L1_BLOCK_CONTRACT: Address = address!("4200000000000000000000000000000000000015");

/// L1 block info
///
/// We can extract L1 epoch data from each L2 block, by looking at the `setL1BlockValues`
/// transaction data. This data is then used to calculate the L1 cost of a transaction.
///
/// Here is the format of the `setL1BlockValues` transaction data:
///
/// setL1BlockValues(uint64 _number, uint64 _timestamp, uint256 _basefee, bytes32 _hash,
/// uint64 _sequenceNumber, bytes32 _batcherHash, uint256 _l1FeeOverhead, uint256 _l1FeeScalar)
///
/// For now, we only care about the fields necessary for L1 cost calculation.
#[derive(Clone, Debug, Default)]
pub struct L1BlockInfo {
    /// The base fee of the L1 origin block.
    pub l1_base_fee: U256,
    /// The current L1 fee overhead. None if Ecotone is activated.
    pub l1_fee_overhead: Option<U256>,
    /// The current L1 fee scalar.
    pub l1_base_fee_scalar: U256,
    /// The current L1 blob base fee. None if Ecotone is not activated, except if `empty_scalars` is `true`.
    pub l1_blob_base_fee: Option<U256>,
    /// The current L1 blob base fee scalar. None if Ecotone is not activated.
    pub l1_blob_base_fee_scalar: Option<U256>,
    /// True if Ecotone is activated, but the L1 fee scalars have not yet been set.
    pub(crate) empty_scalars: bool,
}

impl L1BlockInfo {
    /// Try to fetch the L1 block info from the database.
    pub fn try_fetch<DB: Database>(db: &mut DB, spec_id: SpecId) -> Result<L1BlockInfo, DB::Error> {
        // Ensure the L1 Block account is loaded into the cache after Ecotone. With EIP-4788, it is no longer the case
        // that the L1 block account is loaded into the cache prior to the first inquiry for the L1 block info.
        if spec_id.is_enabled_in(SpecId::CANCUN) {
            let _ = db.basic(L1_BLOCK_CONTRACT)?;
        }

        let l1_base_fee = db.storage(L1_BLOCK_CONTRACT, L1_BASE_FEE_SLOT)?;

        if !spec_id.is_enabled_in(SpecId::ECOTONE) {
            let l1_fee_overhead = db.storage(L1_BLOCK_CONTRACT, L1_OVERHEAD_SLOT)?;
            let l1_fee_scalar = db.storage(L1_BLOCK_CONTRACT, L1_SCALAR_SLOT)?;

            Ok(L1BlockInfo {
                l1_base_fee,
                l1_fee_overhead: Some(l1_fee_overhead),
                l1_base_fee_scalar: l1_fee_scalar,
                ..Default::default()
            })
        } else {
            let l1_blob_base_fee = db.storage(L1_BLOCK_CONTRACT, ECOTONE_L1_BLOB_BASE_FEE_SLOT)?;
            let l1_fee_scalars = db
                .storage(L1_BLOCK_CONTRACT, ECOTONE_L1_FEE_SCALARS_SLOT)?
                .to_be_bytes::<32>();

            let l1_base_fee_scalar = U256::from_be_slice(
                l1_fee_scalars[BASE_FEE_SCALAR_OFFSET..BASE_FEE_SCALAR_OFFSET + 4].as_ref(),
            );
            let l1_blob_base_fee_scalar = U256::from_be_slice(
                l1_fee_scalars[BLOB_BASE_FEE_SCALAR_OFFSET..BLOB_BASE_FEE_SCALAR_OFFSET + 4]
                    .as_ref(),
            );

            // Check if the L1 fee scalars are empty. If so, we use the Bedrock cost function. The L1 fee overhead is
            // only necessary if `empty_scalars` is true, as it was deprecated in Ecotone.
            let empty_scalars = l1_blob_base_fee == U256::ZERO
                && l1_fee_scalars[BASE_FEE_SCALAR_OFFSET..BLOB_BASE_FEE_SCALAR_OFFSET + 4]
                    == EMPTY_SCALARS;
            let l1_fee_overhead = empty_scalars
                .then(|| db.storage(L1_BLOCK_CONTRACT, L1_OVERHEAD_SLOT))
                .transpose()?;

            Ok(L1BlockInfo {
                l1_base_fee,
                l1_base_fee_scalar,
                l1_blob_base_fee: Some(l1_blob_base_fee),
                l1_blob_base_fee_scalar: Some(l1_blob_base_fee_scalar),
                empty_scalars,
                l1_fee_overhead,
            })
        }
    }

    /// Calculate the data gas for posting the transaction on L1. Calldata costs 16 gas per non-zero
    /// byte and 4 gas per zero byte.
    ///
    /// Prior to regolith, an extra 68 non-zero bytes were included in the rollup data costs to
    /// account for the empty signature.
    pub fn data_gas(&self, input: &[u8], spec_id: SpecId) -> U256 {
        let mut rollup_data_gas_cost = U256::from(input.iter().fold(0, |acc, byte| {
            acc + if *byte == 0x00 {
                ZERO_BYTE_COST
            } else {
                NON_ZERO_BYTE_COST
            }
        }));

        // Prior to regolith, an extra 68 non zero bytes were included in the rollup data costs.
        if !spec_id.is_enabled_in(SpecId::REGOLITH) {
            rollup_data_gas_cost += U256::from(NON_ZERO_BYTE_COST).mul(U256::from(68));
        }

        rollup_data_gas_cost
    }

    /// Calculate the gas cost of a transaction based on L1 block data posted on L2, depending on the [SpecId] passed.
    pub fn calculate_tx_l1_cost(&self, input: &[u8], spec_id: SpecId) -> U256 {
        // If the input is a deposit transaction or empty, the default value is zero.
        if input.is_empty() || input.first() == Some(&0x7F) {
            return U256::ZERO;
        }

        if spec_id.is_enabled_in(SpecId::ECOTONE) {
            self.calculate_tx_l1_cost_ecotone(input, spec_id)
        } else {
            self.calculate_tx_l1_cost_bedrock(input, spec_id)
        }
    }

    /// Calculate the gas cost of a transaction based on L1 block data posted on L2, pre-Ecotone.
    fn calculate_tx_l1_cost_bedrock(&self, input: &[u8], spec_id: SpecId) -> U256 {
        let rollup_data_gas_cost = self.data_gas(input, spec_id);
        rollup_data_gas_cost
            .saturating_add(self.l1_fee_overhead.unwrap_or_default())
            .saturating_mul(self.l1_base_fee)
            .saturating_mul(self.l1_base_fee_scalar)
            .wrapping_div(U256::from(1_000_000))
    }

    /// Calculate the gas cost of a transaction based on L1 block data posted on L2, post-Ecotone.
    ///
    /// [SpecId::ECOTONE] L1 cost function:
    /// `(calldataGas/16)*(l1BaseFee*16*l1BaseFeeScalar + l1BlobBaseFee*l1BlobBaseFeeScalar)/1e6`
    ///
    /// We divide "calldataGas" by 16 to change from units of calldata gas to "estimated # of bytes when compressed".
    /// Known as "compressedTxSize" in the spec.
    ///
    /// Function is actually computed as follows for better precision under integer arithmetic:
    /// `calldataGas*(l1BaseFee*16*l1BaseFeeScalar + l1BlobBaseFee*l1BlobBaseFeeScalar)/16e6`
    fn calculate_tx_l1_cost_ecotone(&self, input: &[u8], spec_id: SpecId) -> U256 {
        // There is an edgecase where, for the very first Ecotone block (unless it is activated at Genesis), we must
        // use the Bedrock cost function. To determine if this is the case, we can check if the Ecotone parameters are
        // unset.
        if self.empty_scalars {
            return self.calculate_tx_l1_cost_bedrock(input, spec_id);
        }

        let rollup_data_gas_cost = self.data_gas(input, spec_id);
        let calldata_cost_per_byte = self
            .l1_base_fee
            .saturating_mul(U256::from(16))
            .saturating_mul(self.l1_base_fee_scalar);
        let blob_cost_per_byte = self
            .l1_blob_base_fee
            .unwrap_or_default()
            .saturating_mul(self.l1_blob_base_fee_scalar.unwrap_or_default());

        calldata_cost_per_byte
            .saturating_add(blob_cost_per_byte)
            .saturating_mul(rollup_data_gas_cost)
            .wrapping_div(U256::from(1_000_000 * 16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::bytes;

    #[test]
    fn test_data_gas_non_zero_bytes() {
        let l1_block_info = L1BlockInfo {
            l1_base_fee: U256::from(1_000_000),
            l1_fee_overhead: Some(U256::from(1_000_000)),
            l1_base_fee_scalar: U256::from(1_000_000),
            ..Default::default()
        };

        // 0xFACADE = 6 nibbles = 3 bytes
        // 0xFACADE = 1111 1010 . 1100 1010 . 1101 1110

        // Pre-regolith (ie bedrock) has an extra 68 non-zero bytes
        // gas cost = 3 non-zero bytes * NON_ZERO_BYTE_COST + NON_ZERO_BYTE_COST * 68
        // gas cost = 3 * 16 + 68 * 16 = 1136
        let input = bytes!("FACADE");
        let bedrock_data_gas = l1_block_info.data_gas(&input, SpecId::BEDROCK);
        assert_eq!(bedrock_data_gas, U256::from(1136));

        // Regolith has no added 68 non zero bytes
        // gas cost = 3 * 16 = 48
        let regolith_data_gas = l1_block_info.data_gas(&input, SpecId::REGOLITH);
        assert_eq!(regolith_data_gas, U256::from(48));
    }

    #[test]
    fn test_data_gas_zero_bytes() {
        let l1_block_info = L1BlockInfo {
            l1_base_fee: U256::from(1_000_000),
            l1_fee_overhead: Some(U256::from(1_000_000)),
            l1_base_fee_scalar: U256::from(1_000_000),
            ..Default::default()
        };

        // 0xFA00CA00DE = 10 nibbles = 5 bytes
        // 0xFA00CA00DE = 1111 1010 . 0000 0000 . 1100 1010 . 0000 0000 . 1101 1110

        // Pre-regolith (ie bedrock) has an extra 68 non-zero bytes
        // gas cost = 3 non-zero * NON_ZERO_BYTE_COST + 2 * ZERO_BYTE_COST + NON_ZERO_BYTE_COST * 68
        // gas cost = 3 * 16 + 2 * 4 + 68 * 16 = 1144
        let input = bytes!("FA00CA00DE");
        let bedrock_data_gas = l1_block_info.data_gas(&input, SpecId::BEDROCK);
        assert_eq!(bedrock_data_gas, U256::from(1144));

        // Regolith has no added 68 non zero bytes
        // gas cost = 3 * 16 + 2 * 4 = 56
        let regolith_data_gas = l1_block_info.data_gas(&input, SpecId::REGOLITH);
        assert_eq!(regolith_data_gas, U256::from(56));
    }

    #[test]
    fn test_calculate_tx_l1_cost() {
        let l1_block_info = L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        };

        let input = bytes!("FACADE");
        let gas_cost = l1_block_info.calculate_tx_l1_cost(&input, SpecId::REGOLITH);
        assert_eq!(gas_cost, U256::from(1048));

        // Zero rollup data gas cost should result in zero
        let input = bytes!("");
        let gas_cost = l1_block_info.calculate_tx_l1_cost(&input, SpecId::REGOLITH);
        assert_eq!(gas_cost, U256::ZERO);

        // Deposit transactions with the EIP-2718 type of 0x7F should result in zero
        let input = bytes!("7FFACADE");
        let gas_cost = l1_block_info.calculate_tx_l1_cost(&input, SpecId::REGOLITH);
        assert_eq!(gas_cost, U256::ZERO);
    }

    #[test]
    fn test_calculate_tx_l1_cost_ecotone() {
        let mut l1_block_info = L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_base_fee_scalar: U256::from(1_000),
            l1_blob_base_fee: Some(U256::from(1_000)),
            l1_blob_base_fee_scalar: Some(U256::from(1_000)),
            l1_fee_overhead: Some(U256::from(1_000)),
            ..Default::default()
        };

        // calldataGas * (l1BaseFee * 16 * l1BaseFeeScalar + l1BlobBaseFee * l1BlobBaseFeeScalar) / (16 * 1e6)
        // = (16 * 3) * (1000 * 16 * 1000 + 1000 * 1000) / (16 * 1e6)
        // = 51
        let input = bytes!("FACADE");
        let gas_cost = l1_block_info.calculate_tx_l1_cost(&input, SpecId::ECOTONE);
        assert_eq!(gas_cost, U256::from(51));

        // Zero rollup data gas cost should result in zero
        let input = bytes!("");
        let gas_cost = l1_block_info.calculate_tx_l1_cost(&input, SpecId::ECOTONE);
        assert_eq!(gas_cost, U256::ZERO);

        // Deposit transactions with the EIP-2718 type of 0x7F should result in zero
        let input = bytes!("7FFACADE");
        let gas_cost = l1_block_info.calculate_tx_l1_cost(&input, SpecId::ECOTONE);
        assert_eq!(gas_cost, U256::ZERO);

        // If the scalars are empty, the bedrock cost function should be used.
        l1_block_info.empty_scalars = true;
        let input = bytes!("FACADE");
        let gas_cost = l1_block_info.calculate_tx_l1_cost(&input, SpecId::ECOTONE);
        assert_eq!(gas_cost, U256::from(1048));
    }
}
