use crate::OpSpecId;
use revm::{
    database_interface::Database,
    primitives::{address, Address, U256},
    specification::hardfork::SpecId,
};
use maili_protocol::L1BlockInfoTx;

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

/// <https://github.com/ethereum-optimism/op-geth/blob/647c346e2bef36219cc7b47d76b1cb87e7ca29e4/core/types/rollup_cost.go#L79>
const L1_COST_FASTLZ_COEF: u64 = 836_500;

/// <https://github.com/ethereum-optimism/op-geth/blob/647c346e2bef36219cc7b47d76b1cb87e7ca29e4/core/types/rollup_cost.go#L78>
/// Inverted to be used with `saturating_sub`.
const L1_COST_INTERCEPT: u64 = 42_585_600;

/// <https://github.com/ethereum-optimism/op-geth/blob/647c346e2bef36219cc7b47d76b1cb87e7ca29e4/core/types/rollup_cost.go#82>
const MIN_TX_SIZE_SCALED: u64 = 100 * 1_000_000;

/// Try to fetch the L1 block info from the database.
pub fn try_fetch<DB: Database>(db: &mut DB, spec_id: OpSpec) -> Result<L1BlockInfoTx, DB::Error> {
    // Ensure the L1 Block account is loaded into the cache after Ecotone. With EIP-4788, it is no longer the case
    // that the L1 block account is loaded into the cache prior to the first inquiry for the L1 block info.
    if spec_id.is_enabled_in(SpecId::CANCUN) {
        let _ = db.basic(L1_BLOCK_CONTRACT)?;
    }

    let l1_base_fee = db.storage(L1_BLOCK_CONTRACT, L1_BASE_FEE_SLOT)?;

    if !spec_id.is_enabled_in(OpSpecId::ECOTONE) {
        let l1_fee_overhead = db.storage(L1_BLOCK_CONTRACT, L1_OVERHEAD_SLOT)?;
        let l1_fee_scalar = db.storage(L1_BLOCK_CONTRACT, L1_SCALAR_SLOT)?;

        Ok(L1BlockInfoTx {
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

        // Check if the L1 fee scalars are empty. If so, we use the Bedrock cost function.
        // The L1 fee overhead is only necessary if `empty_scalars` is true, as it was deprecated in Ecotone.
        let empty_scalars = l1_blob_base_fee.is_zero()
            && l1_fee_scalars[BASE_FEE_SCALAR_OFFSET..BLOB_BASE_FEE_SCALAR_OFFSET + 4]
                == EMPTY_SCALARS;
        let l1_fee_overhead = empty_scalars
            .then(|| db.storage(L1_BLOCK_CONTRACT, L1_OVERHEAD_SLOT))
            .transpose()?;

        Ok(L1BlockInfoTx {
            l1_base_fee,
            l1_base_fee_scalar,
            l1_blob_base_fee: Some(l1_blob_base_fee),
            l1_blob_base_fee_scalar: Some(l1_blob_base_fee_scalar),
            empty_scalars,
            l1_fee_overhead,
        })
    }
}
