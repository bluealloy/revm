use alloy_rlp::Encodable;
use alloy_trie::root::ordered_trie_root_with_encoder;
use revm::primitives::{alloy_primitives::Bloom, Log, B256};

/// Receipt data collected for each included transaction, used to recompute
/// the header's `receiptTrie` root.
///
/// Only supports Byzantium+ receipts (status byte); pre-Byzantium receipts
/// embed the intermediate state root instead.
#[derive(Debug)]
pub struct Receipt {
    pub tx_type: u8,
    pub success: bool,
    pub cumulative_gas_used: u64,
    pub logs: Vec<Log>,
}

impl Receipt {
    /// Encodes the receipt as it is stored in the trie: EIP-2718 typed
    /// receipts are the type byte followed by the RLP payload.
    fn encode(&self, out: &mut Vec<u8>) {
        let mut bloom = Bloom::default();
        for log in &self.logs {
            bloom.accrue_log(log);
        }

        if self.tx_type > 0 {
            out.push(self.tx_type);
        }
        let payload_length = self.success.length()
            + self.cumulative_gas_used.length()
            + bloom.length()
            + alloy_rlp::list_length(&self.logs);
        alloy_rlp::Header {
            list: true,
            payload_length,
        }
        .encode(out);
        self.success.encode(out);
        self.cumulative_gas_used.encode(out);
        bloom.encode(out);
        alloy_rlp::encode_list(&self.logs, out);
    }
}

/// Computes the receipts trie root, keyed by RLP-encoded transaction index.
pub fn receipts_root(receipts: &[Receipt]) -> B256 {
    ordered_trie_root_with_encoder(receipts, |receipt, out| receipt.encode(out))
}
