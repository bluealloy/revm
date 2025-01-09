use crate::OpSpecId;
use maili_protocol::{
    calculate_tx_l1_cost_bedrock, calculate_tx_l1_cost_bedrock_empty_scalars,
    calculate_tx_l1_cost_ecotone, calculate_tx_l1_cost_fjord, calculate_tx_l1_cost_regolith,
    L1BlockInfoBedrock, L1BlockInfoEcotone, L1BlockInfoTx,
};
use revm::{
    database_interface::Database,
    primitives::{address, Address, Bytes, U256},
    specification::hardfork::SpecId,
};

use super::OpSpec;

pub const ZERO_BYTE_COST: u64 = 4;
pub const NON_ZERO_BYTE_COST: u64 = 16;

/// The two 4-byte Ecotone fee scalar values are packed into the same storage slot as the 8-byte sequence number.
/// Byte offset within the storage slot of the 4-byte baseFeeScalar attribute.
pub const BASE_FEE_SCALAR_OFFSET: usize = 16;
/// The two 4-byte Ecotone fee scalar values are packed into the same storage slot as the 8-byte sequence number.
/// Byte offset within the storage slot of the 4-byte blobBaseFeeScalar attribute.
pub const BLOB_BASE_FEE_SCALAR_OFFSET: usize = 20;

pub const L1_BASE_FEE_SLOT: U256 = U256::from_limbs([1u64, 0, 0, 0]);
pub const L1_OVERHEAD_SLOT: U256 = U256::from_limbs([5u64, 0, 0, 0]);
pub const L1_SCALAR_SLOT: U256 = U256::from_limbs([6u64, 0, 0, 0]);

/// [ECOTONE_L1_BLOB_BASE_FEE_SLOT] was added in the Ecotone upgrade and stores the L1 blobBaseFee attribute.
pub const ECOTONE_L1_BLOB_BASE_FEE_SLOT: U256 = U256::from_limbs([7u64, 0, 0, 0]);

/// As of the ecotone upgrade, this storage slot stores the 32-bit basefeeScalar and blobBaseFeeScalar attributes at
/// offsets [BASE_FEE_SCALAR_OFFSET] and [BLOB_BASE_FEE_SCALAR_OFFSET] respectively.
pub const ECOTONE_L1_FEE_SCALARS_SLOT: U256 = U256::from_limbs([3u64, 0, 0, 0]);

/// An empty 64-bit set of scalar values.
const EMPTY_SCALARS: [u8; 8] = [0u8; 8];

/// The address of L1 fee recipient.
pub const L1_FEE_RECIPIENT: Address = address!("420000000000000000000000000000000000001A");

/// The address of the base fee recipient.
pub const BASE_FEE_RECIPIENT: Address = address!("4200000000000000000000000000000000000019");

/// The address of the L1Block contract.
pub const L1_BLOCK_CONTRACT: Address = address!("4200000000000000000000000000000000000015");

/// Calculates the L1 cost of a transaction.
pub fn calculate_tx_l1_cost(block_info: &L1BlockInfoTx, tx: &Bytes, spec_id: OpSpec) -> U256 {
    if spec_id.is_enabled_in(OpSpecId::FJORD) {
        calculate_tx_l1_cost_fjord(
            &tx[..],
            block_info.l1_base_fee(),
            block_info.l1_fee_scalar(),
            block_info.blob_base_fee(),
            block_info.blob_base_fee_scalar(),
        )
    } else if spec_id.is_enabled_in(OpSpecId::ECOTONE) {
        if block_info.empty_scalars() {
            return calculate_tx_l1_cost_bedrock_empty_scalars(
                &tx[..],
                block_info.l1_fee_overhead(),
                block_info.l1_base_fee(),
                block_info.l1_fee_scalar(),
            );
        }
        calculate_tx_l1_cost_ecotone(
            &tx[..],
            block_info.l1_base_fee(),
            block_info.l1_fee_scalar(),
            block_info.blob_base_fee(),
            block_info.blob_base_fee_scalar(),
        )
    } else if spec_id.is_enabled_in(OpSpecId::REGOLITH) {
        calculate_tx_l1_cost_regolith(
            &tx[..],
            block_info.l1_fee_overhead(),
            block_info.l1_base_fee(),
            block_info.l1_fee_scalar(),
        )
    } else {
        calculate_tx_l1_cost_bedrock(
            &tx[..],
            block_info.l1_fee_overhead(),
            block_info.l1_base_fee(),
            block_info.l1_fee_scalar(),
        )
    }
}

/// Try to fetch the L1 block info from the database.
pub fn try_fetch<DB: Database>(db: &mut DB, spec_id: OpSpec) -> Result<L1BlockInfoTx, DB::Error> {
    // Ensure the L1 Block account is loaded into the cache after Ecotone. With EIP-4788, it is no longer the case
    // that the L1 block account is loaded into the cache prior to the first inquiry for the L1 block info.
    if spec_id.is_enabled_in(SpecId::CANCUN) {
        let _ = db.basic(L1_BLOCK_CONTRACT)?;
    }

    let l1_base_fee =
        TryInto::<u64>::try_into(db.storage(L1_BLOCK_CONTRACT, L1_BASE_FEE_SLOT)?).unwrap();

    if !spec_id.is_enabled_in(OpSpecId::ECOTONE) {
        let l1_fee_overhead = db.storage(L1_BLOCK_CONTRACT, L1_OVERHEAD_SLOT)?;
        let l1_fee_scalar = db.storage(L1_BLOCK_CONTRACT, L1_SCALAR_SLOT)?;

        Ok(L1BlockInfoTx::Bedrock(L1BlockInfoBedrock {
            base_fee: l1_base_fee,
            l1_fee_scalar,
            l1_fee_overhead,
            ..Default::default()
        }))
    } else {
        let l1_blob_base_fee = TryInto::<u128>::try_into(
            db.storage(L1_BLOCK_CONTRACT, ECOTONE_L1_BLOB_BASE_FEE_SLOT)?,
        )
        .unwrap();
        let l1_fee_scalars = db
            .storage(L1_BLOCK_CONTRACT, ECOTONE_L1_FEE_SCALARS_SLOT)?
            .to_be_bytes::<32>();

        let l1_base_fee_scalar = TryInto::<u32>::try_into(U256::from_be_slice(
            l1_fee_scalars[BASE_FEE_SCALAR_OFFSET..BASE_FEE_SCALAR_OFFSET + 4].as_ref(),
        ))
        .unwrap();
        let l1_blob_base_fee_scalar = TryInto::<u32>::try_into(U256::from_be_slice(
            l1_fee_scalars[BLOB_BASE_FEE_SCALAR_OFFSET..BLOB_BASE_FEE_SCALAR_OFFSET + 4].as_ref(),
        ))
        .unwrap();

        // Check if the L1 fee scalars are empty. If so, we use the Bedrock cost function.
        // The L1 fee overhead is only necessary if `empty_scalars` is true, as it was deprecated in Ecotone.
        let empty_scalars = l1_blob_base_fee == 0
            && l1_fee_scalars[BASE_FEE_SCALAR_OFFSET..BLOB_BASE_FEE_SCALAR_OFFSET + 4]
                == EMPTY_SCALARS;
        let l1_fee_overhead = empty_scalars
            .then(|| db.storage(L1_BLOCK_CONTRACT, L1_OVERHEAD_SLOT))
            .transpose()?;

        Ok(L1BlockInfoTx::Ecotone(L1BlockInfoEcotone {
            base_fee: l1_base_fee,
            base_fee_scalar: l1_base_fee_scalar,
            blob_base_fee: l1_blob_base_fee,
            blob_base_fee_scalar: l1_blob_base_fee_scalar,
            empty_scalars,
            l1_fee_overhead: l1_fee_overhead.unwrap_or_default(),
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maili_protocol::{data_gas_bedrock, data_gas_fjord, data_gas_regolith};
    use revm::primitives::{bytes, hex};

    #[test]
    fn test_data_gas_non_zero_bytes() {
        // 0xFACADE = 6 nibbles = 3 bytes
        // 0xFACADE = 1111 1010 . 1100 1010 . 1101 1110

        // Pre-regolith (ie bedrock) has an extra 68 non-zero bytes
        // gas cost = 3 non-zero bytes * NON_ZERO_BYTE_COST + NON_ZERO_BYTE_COST * 68
        // gas cost = 3 * 16 + 68 * 16 = 1136
        let input = bytes!("FACADE");
        let bedrock_data_gas = data_gas_bedrock(&input);
        assert_eq!(bedrock_data_gas, U256::from(1136));

        // Regolith has no added 68 non zero bytes
        // gas cost = 3 * 16 = 48
        let regolith_data_gas = data_gas_regolith(&input);
        assert_eq!(regolith_data_gas, U256::from(48));

        // Fjord has a minimum compressed size of 100 bytes
        // gas cost = 100 * 16 = 1600
        let fjord_data_gas = data_gas_fjord(&input);
        assert_eq!(fjord_data_gas, U256::from(1600));
    }

    #[test]
    fn test_data_gas_zero_bytes() {
        // 0xFA00CA00DE = 10 nibbles = 5 bytes
        // 0xFA00CA00DE = 1111 1010 . 0000 0000 . 1100 1010 . 0000 0000 . 1101 1110

        // Pre-regolith (ie bedrock) has an extra 68 non-zero bytes
        // gas cost = 3 non-zero * NON_ZERO_BYTE_COST + 2 * ZERO_BYTE_COST + NON_ZERO_BYTE_COST * 68
        // gas cost = 3 * 16 + 2 * 4 + 68 * 16 = 1144
        let input = bytes!("FA00CA00DE");
        let bedrock_data_gas = data_gas_bedrock(&input);
        assert_eq!(bedrock_data_gas, U256::from(1144));

        // Regolith has no added 68 non zero bytes
        // gas cost = 3 * 16 + 2 * 4 = 56
        let regolith_data_gas = data_gas_regolith(&input);
        assert_eq!(regolith_data_gas, U256::from(56));

        // Fjord has a minimum compressed size of 100 bytes
        // gas cost = 100 * 16 = 1600
        let fjord_data_gas = data_gas_fjord(&input);
        assert_eq!(fjord_data_gas, U256::from(1600));
    }

    #[test]
    fn test_calculate_tx_l1_cost() {
        let l1_block_info = L1BlockInfoTx::Bedrock(L1BlockInfoBedrock {
            base_fee: 1_000,
            l1_fee_overhead: U256::from(1_000),
            l1_fee_scalar: U256::from(1_000),
            ..Default::default()
        });

        let input = bytes!("FACADE");
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::REGOLITH.into());
        assert_eq!(gas_cost, U256::from(1048));

        // Zero rollup data gas cost should result in zero
        let input = bytes!("");
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::REGOLITH.into());
        assert_eq!(gas_cost, U256::ZERO);

        // Deposit transactions with the EIP-2718 type of 0x7F should result in zero
        let input = bytes!("7FFACADE");
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::REGOLITH.into());
        assert_eq!(gas_cost, U256::ZERO);
    }

    #[test]
    fn test_calculate_tx_l1_cost_ecotone() {
        let mut l1_block_info = L1BlockInfoTx::Ecotone(L1BlockInfoEcotone {
            base_fee: 1_000,
            base_fee_scalar: 1_000,
            blob_base_fee: 1_000,
            blob_base_fee_scalar: 1_000,
            empty_scalars: false,
            l1_fee_overhead: U256::from(1_000),
            ..Default::default()
        });

        // calldataGas * (l1BaseFee * 16 * l1BaseFeeScalar + l1BlobBaseFee * l1BlobBaseFeeScalar) / (16 * 1e6)
        // = (16 * 3) * (1000 * 16 * 1000 + 1000 * 1000) / (16 * 1e6)
        // = 51
        let input = bytes!("FACADE");
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::ECOTONE.into());
        assert_eq!(gas_cost, U256::from(51));

        // Zero rollup data gas cost should result in zero
        let input = bytes!("");
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::ECOTONE.into());
        assert_eq!(gas_cost, U256::ZERO);

        // Deposit transactions with the EIP-2718 type of 0x7F should result in zero
        let input = bytes!("7FFACADE");
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::ECOTONE.into());
        assert_eq!(gas_cost, U256::ZERO);

        // If the scalars are empty, the bedrock cost function should be used.
        l1_block_info = L1BlockInfoTx::Ecotone(L1BlockInfoEcotone {
            base_fee: 1_000,
            base_fee_scalar: 1_000,
            blob_base_fee: 1_000,
            blob_base_fee_scalar: 1_000,
            empty_scalars: true,
            l1_fee_overhead: U256::from(1_000),
            ..Default::default()
        });
        let input = bytes!("FACADE");
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::ECOTONE.into());
        assert_eq!(gas_cost, U256::from(1048));
    }

    #[test]
    fn calculate_tx_l1_cost_ecotone() {
        // rig

        // l1 block info for OP mainnet ecotone block 118024092
        // 1710374401 (ecotone timestamp)
        // 1711603765 (block 118024092 timestamp)
        // 1720627201 (fjord timestamp)
        // <https://optimistic.etherscan.io/block/118024092>
        // decoded from
        let l1_block_info = L1BlockInfoTx::Ecotone(L1BlockInfoEcotone {
            base_fee: 47_036_678_951_u64,
            base_fee_scalar: 1_368_u32,
            blob_base_fee: 57_422_457_042_u128,
            blob_base_fee_scalar: 810_949_u32,
            ..Default::default()
        });

        // second tx in OP mainnet ecotone block 118024092
        // <https://optimistic.etherscan.io/tx/0xa75ef696bf67439b4d5b61da85de9f3ceaa2e145abe982212101b244b63749c2>
        const TX: &[u8] = &hex!("02f8b30a832253fc8402d11f39842c8a46398301388094dc6ff44d5d932cbd77b52e5612ba0529dc6226f180b844a9059cbb000000000000000000000000d43e02db81f4d46cdf8521f623d21ea0ec7562a50000000000000000000000000000000000000000000000008ac7230489e80000c001a02947e24750723b48f886931562c55d9e07f856d8e06468e719755e18bbc3a570a0784da9ce59fd7754ea5be6e17a86b348e441348cd48ace59d174772465eadbd1");

        // l1 gas used for tx and l1 fee for tx, from OP mainnet block scanner
        // <https://optimistic.etherscan.io/tx/0xa75ef696bf67439b4d5b61da85de9f3ceaa2e145abe982212101b244b63749c2>
        let expected_l1_gas_used = U256::from(2456);
        let expected_l1_fee = U256::from_be_bytes(hex!(
            "000000000000000000000000000000000000000000000000000006a510bd7431" // 7306020222001 wei
        ));

        // test

        let gas_used = data_gas_regolith(TX);

        assert_eq!(gas_used, expected_l1_gas_used);

        let l1_fee =
            calculate_tx_l1_cost(&l1_block_info, &Bytes::from(TX), OpSpecId::ECOTONE.into());

        assert_eq!(l1_fee, expected_l1_fee)
    }

    #[test]
    fn test_calculate_tx_l1_cost_fjord() {
        // l1FeeScaled = baseFeeScalar*l1BaseFee*16 + blobFeeScalar*l1BlobBaseFee
        //             = 1000 * 1000 * 16 + 1000 * 1000
        //             = 17e6
        let l1_block_info = L1BlockInfoTx::Ecotone(L1BlockInfoEcotone {
            base_fee: 1_000,
            base_fee_scalar: 1_000,
            blob_base_fee: 1_000,
            blob_base_fee_scalar: 1_000,
            ..Default::default()
        });

        // fastLzSize = 4
        // estimatedSize = max(minTransactionSize, intercept + fastlzCoef*fastlzSize)
        //               = max(100e6, 836500*4 - 42585600)
        //               = 100e6
        let input = bytes!("FACADE");
        // l1Cost = estimatedSize * l1FeeScaled / 1e12
        //        = 100e6 * 17 / 1e6
        //        = 1700
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::FJORD.into());
        assert_eq!(gas_cost, U256::from(1700));

        // fastLzSize = 202
        // estimatedSize = max(minTransactionSize, intercept + fastlzCoef*fastlzSize)
        //               = max(100e6, 836500*202 - 42585600)
        //               = 126387400
        let input = bytes!("02f901550a758302df1483be21b88304743f94f80e51afb613d764fa61751affd3313c190a86bb870151bd62fd12adb8e41ef24f3f000000000000000000000000000000000000000000000000000000000000006e000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831000000000000000000000000000000000000000000000000000000000003c1e5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000148c89ed219d02f1a5be012c689b4f5b731827bebe000000000000000000000000c001a033fd89cb37c31b2cba46b6466e040c61fc9b2a3675a7f5f493ebd5ad77c497f8a07cdf65680e238392693019b4092f610222e71b7cec06449cb922b93b6a12744e");
        // l1Cost = estimatedSize * l1FeeScaled / 1e12
        //        = 126387400 * 17 / 1e6
        //        = 2148
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::FJORD.into());
        assert_eq!(gas_cost, U256::from(2148));

        // Zero rollup data gas cost should result in zero
        let input = bytes!("");
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::FJORD.into());
        assert_eq!(gas_cost, U256::ZERO);

        // Deposit transactions with the EIP-2718 type of 0x7F should result in zero
        let input = bytes!("7FFACADE");
        let gas_cost = calculate_tx_l1_cost(&l1_block_info, &input, OpSpecId::FJORD.into());
        assert_eq!(gas_cost, U256::ZERO);
    }

    #[test]
    fn calculate_tx_l1_cost_fjord() {
        // rig

        // L1 block info for OP mainnet fjord block 124665056
        // <https://optimistic.etherscan.io/block/124665056>
        let l1_block_info = L1BlockInfoTx::Ecotone(L1BlockInfoEcotone {
            base_fee: 1055991687,
            base_fee_scalar: 5227,
            blob_base_fee_scalar: 1014213,
            blob_base_fee: 1,
            ..Default::default() // L1 fee overhead (l1 gas used) deprecated since Fjord
        });

        // Second tx in OP mainnet Fjord block 124665056
        // <https://optimistic.etherscan.io/tx/0x1059e8004daff32caa1f1b1ef97fe3a07a8cf40508f5b835b66d9420d87c4a4a>
        const TX: &[u8] = &hex!("02f904940a8303fba78401d6d2798401db2b6d830493e0943e6f4f7866654c18f536170780344aa8772950b680b904246a761202000000000000000000000000087000a300de7200382b55d40045000000e5d60e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a0000000000000000000000000000000000000000000000000000000000000022482ad56cb0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000120000000000000000000000000dc6ff44d5d932cbd77b52e5612ba0529dc6226f1000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000044095ea7b300000000000000000000000021c4928109acb0659a88ae5329b5374a3024694c0000000000000000000000000000000000000000000000049b9ca9a6943400000000000000000000000000000000000000000000000000000000000000000000000000000000000021c4928109acb0659a88ae5329b5374a3024694c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000024b6b55f250000000000000000000000000000000000000000000000049b9ca9a694340000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000415ec214a3950bea839a7e6fbb0ba1540ac2076acd50820e2d5ef83d0902cdffb24a47aff7de5190290769c4f0a9c6fabf63012986a0d590b1b571547a8c7050ea1b00000000000000000000000000000000000000000000000000000000000000c080a06db770e6e25a617fe9652f0958bd9bd6e49281a53036906386ed39ec48eadf63a07f47cf51a4a40b4494cf26efc686709a9b03939e20ee27e59682f5faa536667e");

        // L1 gas used for tx and L1 fee for tx, from OP mainnet block scanner
        // https://optimistic.etherscan.io/tx/0x1059e8004daff32caa1f1b1ef97fe3a07a8cf40508f5b835b66d9420d87c4a4a
        let expected_data_gas = U256::from(4471);
        let expected_l1_fee = U256::from_be_bytes(hex!(
            "00000000000000000000000000000000000000000000000000000005bf1ab43d"
        ));

        // test

        let data_gas = data_gas_fjord(TX);

        assert_eq!(data_gas, expected_data_gas);

        let l1_fee = calculate_tx_l1_cost(&l1_block_info, &Bytes::from(TX), OpSpecId::FJORD.into());

        assert_eq!(l1_fee, expected_l1_fee);
    }
}
