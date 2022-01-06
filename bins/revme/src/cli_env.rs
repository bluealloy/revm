use std::str::FromStr;

use bytes::Bytes;
use primitive_types::{H160, U256};
use revm::{Env, TransactTo};
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
pub struct CliEnv {
    #[structopt(flatten)]
    block: CliEnvBlock,
    #[structopt(flatten)]
    tx: CliEnvTx,
}

macro_rules! local_fill {
    ($left:expr, $right:expr, $fun:expr) => {
        if let Some(right) = $right {
            $left = $fun(right)
        }
    };
    ($left:expr, $right:expr) => {
        if let Some(right) = $right {
            $left = right
        }
    };
}

impl Into<Env> for CliEnv {
    fn into(self) -> Env {
        let mut env = Env::default();
        local_fill!(env.block.gas_limit, self.block.block_gas_limit, U256::from);
        local_fill!(env.block.number, self.block.number, U256::from);
        local_fill!(env.block.coinbase, self.block.coinbase);
        local_fill!(env.block.timestamp, self.block.timestamp, U256::from);
        local_fill!(env.block.difficulty, self.block.difficulty, U256::from);
        local_fill!(env.block.basefee, self.block.basefee, U256::from);

        local_fill!(env.tx.caller, self.tx.caller);
        local_fill!(env.tx.gas_limit, self.tx.tx_gas_limit);
        local_fill!(env.tx.value, self.tx.value, U256::from);
        local_fill!(env.tx.data, self.tx.data);
        env.tx.gas_priority_fee = self.tx.gas_priority_fee.map(U256::from);
        env.tx.chain_id = self.tx.chain_id;
        env.tx.nonce = self.tx.nonce;

        env.tx.transact_to = if let Some(to) = self.tx.transact_to {
            TransactTo::Call(to)
        } else {
            TransactTo::create()
        };
        //TODO tx access_list

        env
    }
}

#[derive(StructOpt, Clone, Debug)]
pub struct CliEnvBlock {
    #[structopt(long = "env.block.gas_limit")]
    pub block_gas_limit: Option<u64>,
    /// somebody call it nonce
    #[structopt(long = "env.block.number")]
    pub number: Option<u64>,
    /// Coinbase or miner or address that created and signed the block.
    /// Address where we are going to send gas spend
    #[structopt(long = "env.block.coinbase", parse(try_from_str = parse_h160))]
    pub coinbase: Option<H160>,
    #[structopt(long = "env.block.timestamp")]
    pub timestamp: Option<u64>,
    #[structopt(long = "env.block.difficulty")]
    pub difficulty: Option<u64>,
    /// basefee is added in EIP1559 London upgrade
    #[structopt(long = "env.block.basefee")]
    pub basefee: Option<u64>,
}

#[derive(StructOpt, Clone, Debug)]
pub struct CliEnvTx {
    /// Caller or Author or tx signer
    #[structopt(long = "env.tx.caller", parse(try_from_str = parse_h160))]
    pub caller: Option<H160>,
    #[structopt(long = "env.tx.gas_limit")]
    pub tx_gas_limit: Option<u64>,
    #[structopt(long = "env.tx.gas_price")]
    pub gas_price: Option<u64>,
    #[structopt(long = "env.tx.gas_priority_fee")]
    pub gas_priority_fee: Option<u64>,
    #[structopt(long = "env.tx.to", parse(try_from_str = parse_h160))]
    pub transact_to: Option<H160>,
    #[structopt(long = "env.tx.value")]
    pub value: Option<u64>,
    #[structopt(long = "env.tx.data", parse(try_from_str = parse_hex))]
    pub data: Option<Bytes>,
    #[structopt(long = "env.tx.chain_id")]
    pub chain_id: Option<u64>,
    #[structopt(long = "env.tx.nonce")]
    pub nonce: Option<u64>,
    //#[structopt(long = "env.")]
    //TODO pub access_list: Vec<(H160, Vec<U256>)>,
}

fn parse_hex(src: &str) -> Result<Bytes, hex::FromHexError> {
    Ok(Bytes::from(hex::decode(src)?))
}

pub fn parse_h160(input: &str) -> Result<H160, <H160 as FromStr>::Err> {
    H160::from_str(input)
}
