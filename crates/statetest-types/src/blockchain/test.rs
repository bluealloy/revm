#[cfg(test)]
mod tests {
    use crate::blockchain::BlockchainTest;

    #[test]
    fn test_blockchain_test() {
        let sample = include_str!("sample.json");
        println!("{}", sample);
        let _: BlockchainTest = serde_json::from_str(sample).unwrap();
    }

    #[test]
    fn test_env_conversions() {
        let sample = include_str!("sample.json");
        let test: BlockchainTest = serde_json::from_str(sample).unwrap();

        // Get the first test case
        let (_, test_case) = test.0.iter().next().unwrap();

        // Test genesis block conversion
        let genesis_env = test_case.genesis_block_env();
        assert_eq!(genesis_env.number.to::<u64>(), 0);
        assert_eq!(genesis_env.timestamp.to::<u64>(), 0);
        assert_eq!(genesis_env.gas_limit, 72_000_000);
        assert_eq!(genesis_env.basefee, 7);

        // Test chain ID extraction
        let chain_id = test_case.chain_id();
        assert_eq!(chain_id, 1);

        // Test transaction conversion from first block
        if let Some(first_block) = test_case.blocks.first() {
            if let Some(transactions) = &first_block.transactions {
                if let Some(first_tx) = transactions.first() {
                    let tx_env = first_tx.to_tx_env().unwrap();
                    assert_eq!(tx_env.tx_type, 0); // Legacy transaction
                    assert_eq!(tx_env.nonce, 0);
                    assert_eq!(tx_env.gas_limit, 30_000_001);
                    assert_eq!(tx_env.gas_price, 10);
                    assert_eq!(tx_env.value.to::<u64>(), 0);
                }
            }
        }
    }
}
