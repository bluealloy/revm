pub trait Cfg {
    fn chain_id(&self) -> u64;

    fn max_code_size(&self) -> usize;

    fn is_eip3607_disabled(&self) -> bool;

    fn is_balance_check_disabled(&self) -> bool;

    fn is_gas_refund_disabled(&self) -> bool;

    fn is_block_gas_limit_disabled(&self) -> bool;

    fn is_nonce_check_disabled(&self) -> bool;

    fn is_base_fee_check_disabled(&self) -> bool;
}
