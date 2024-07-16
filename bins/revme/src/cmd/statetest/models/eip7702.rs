use alloy_rlp::{Decodable, Error as RlpError, Header};
use revm::primitives::{AccessList, Bytes, Signature, SignedAuthorization, TxKind, U256};
use std::vec::Vec;

/// [EIP-7702 Set Code Transaction](https://eips.ethereum.org/EIPS/eip-7702)
///
/// Set EOA account code for one transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxEip7702 {
    /// Added as EIP-155: Simple replay attack protection
    pub chain_id: u64,
    /// A scalar value equal to the number of transactions sent by the sender; formally Tn.
    pub nonce: u64,
    /// A scalar value equal to the number of
    /// Wei to be paid per unit of gas for all computation
    /// costs incurred as a result of the execution of this transaction; formally Tp.
    ///
    /// As ethereum circulation is around 120mil eth as of 2022 that is around
    /// 120000000000000000000000000 wei we are safe to use u128 as its max number is:
    /// 340282366920938463463374607431768211455
    pub gas_limit: u64,
    /// A scalar value equal to the maximum
    /// amount of gas that should be used in executing
    /// this transaction. This is paid up-front, before any
    /// computation is done and may not be increased
    /// later; formally Tg.
    ///
    /// As ethereum circulation is around 120mil eth as of 2022 that is around
    /// 120000000000000000000000000 wei we are safe to use u128 as its max number is:
    /// 340282366920938463463374607431768211455
    ///
    /// This is also known as `GasFeeCap`
    pub max_fee_per_gas: u128,
    /// Max Priority fee that transaction is paying
    ///
    /// As ethereum circulation is around 120mil eth as of 2022 that is around
    /// 120000000000000000000000000 wei we are safe to use u128 as its max number is:
    /// 340282366920938463463374607431768211455
    ///
    /// This is also known as `GasTipCap`
    pub max_priority_fee_per_gas: u128,
    /// The 160-bit address of the message call’s recipient or, for a contract creation
    /// transaction, ∅, used here to denote the only member of B0 ; formally Tt.
    pub to: TxKind,
    /// A scalar value equal to the number of Wei to
    /// be transferred to the message call’s recipient or,
    /// in the case of contract creation, as an endowment
    /// to the newly created account; formally Tv.
    pub value: U256,
    /// The accessList specifies a list of addresses and storage keys;
    /// these addresses and storage keys are added into the `accessed_addresses`
    /// and `accessed_storage_keys` global sets (introduced in EIP-2929).
    /// A gas cost is charged, though at a discount relative to the cost of
    /// accessing outside the list.
    pub access_list: AccessList,
    /// Authorizations are used to temporarily set the code of its signer to
    /// the code referenced by `address`. These also include a `chain_id` (which
    /// can be set to zero and not evaluated) as well as an optional `nonce`.
    pub authorization_list: Vec<SignedAuthorization>,
    /// Input has two uses depending if the transaction `to` field is [`TxKind::Create`] or
    /// [`TxKind::Call`].
    ///
    /// Input as init code, or if `to` is [`TxKind::Create`]: An unlimited size byte array
    /// specifying the EVM-code for the account initialisation procedure `CREATE`
    ///
    /// Input as data, or if `to` is [`TxKind::Call`]: An unlimited size byte array specifying the
    /// input data of the message call, formally Td.
    pub input: Bytes,
    pub signature: Signature,
}

impl TxEip7702 {
    /// Decodes the inner [`TxEip7702`] fields from RLP bytes.
    ///
    /// NOTE: This assumes a RLP header has already been decoded, and _just_ decodes the following
    /// RLP fields in the following order:
    ///
    /// - `chain_id`
    /// - `nonce`
    /// - `gas_price`
    /// - `gas_limit`
    /// - `to`
    /// - `value`
    /// - `data` (`input`)
    /// - `access_list`
    /// - `authorization_list`
    fn decode_inner(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            chain_id: Decodable::decode(buf)?,
            nonce: Decodable::decode(buf)?,
            max_priority_fee_per_gas: Decodable::decode(buf)?,
            max_fee_per_gas: Decodable::decode(buf)?,
            gas_limit: Decodable::decode(buf)?,
            to: Decodable::decode(buf)?,
            value: Decodable::decode(buf)?,
            input: Decodable::decode(buf)?,
            access_list: Decodable::decode(buf)?,
            authorization_list: Decodable::decode(buf)?,
            signature: Signature::decode_rlp_vrs(buf)?,
        })
    }

    pub fn decode(data: &mut &[u8]) -> alloy_rlp::Result<Self> {
        // decode the list header for the rest of the transaction
        let header = Header::decode(data)?;
        if !header.list {
            return Err(RlpError::Custom(
                "typed tx fields must be encoded as a list",
            ));
        }
        let tx = TxEip7702::decode_inner(data)?;
        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_eip7702_tx() {
        let tx_bytes = hex::decode("f8c2018080078398968094a94f5374fce5edbc8e2a8697c15331677e6ebf0b8080c0f85df85b01940000000000000000000000000000000000001000c18001a08171c0ded912d4f458b8115618c18f3f430f414919c73b4daa693c47fd325414a0787741e1621bcb9cb58ece039ad73f41d9422aa259ed53c2b0bd30dc7ff09be780a00e6c8f4d73b175887e1f21cc00bf0f8243af18aed208ec0a4562ee60e7f85736a03f8e8f1b01fcd6d3a988877e80dc17fad16274447f4211ed74b41e8789ae70cd").unwrap();
        let tx = TxEip7702::decode(&mut tx_bytes.as_slice()).unwrap();
        assert_eq!(tx.authorization_list.len(), 1);
    }

    #[test]
    fn test_eip7702_tx() {
        let tx_bytes = hex::decode("f8c2018080078398968094a94f5374fce5edbc8e2a8697c15331677e6ebf0b8080c0f85df85b80940000000000000000000000000000000000001000c10180a09e833a19cf7ac609d713ffeb8d5cd327237ef5cb4ac9524c53195423e348629fa0632893e4b18b32faf56972dc3568c3a3869dcf9eb9c282a637173475d19e8d2f01a05d6eea7691335a6bb066613d5c33a27bd1cbc89feb472b6dd437aca6aec73282a013c492943ea0fce77a20b1d554eac087fee37fa27b0f8294b13fb3162a0fb175").unwrap();
        let tx = TxEip7702::decode(&mut tx_bytes.as_slice()).unwrap();
        assert_eq!(tx.authorization_list.len(), 1);
    }
}
