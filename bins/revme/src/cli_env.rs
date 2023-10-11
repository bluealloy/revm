use revm::primitives::{Address, Bytes, Env, TransactTo, U256};
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug, PartialEq, Eq)]
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

impl From<CliEnv> for Env {
    fn from(from: CliEnv) -> Self {
        let mut env = Env::default();
        local_fill!(env.block.gas_limit, from.block.block_gas_limit, U256::from);
        local_fill!(env.block.number, from.block.number, U256::from);
        local_fill!(env.block.coinbase, from.block.coinbase);
        local_fill!(env.block.timestamp, from.block.timestamp, U256::from);
        local_fill!(env.block.difficulty, from.block.difficulty, U256::from);
        local_fill!(env.block.basefee, from.block.basefee, U256::from);

        local_fill!(env.tx.caller, from.tx.caller);
        local_fill!(env.tx.gas_limit, from.tx.tx_gas_limit);
        local_fill!(env.tx.value, from.tx.value, U256::from);
        local_fill!(env.tx.data, from.tx.data);
        env.tx.gas_priority_fee = from.tx.gas_priority_fee.map(U256::from);
        env.tx.chain_id = from.tx.chain_id;
        env.tx.nonce = from.tx.nonce;

        env.tx.transact_to = if let Some(to) = from.tx.transact_to {
            TransactTo::Call(to)
        } else {
            TransactTo::create()
        };
        //TODO tx access_list

        env
    }
}

#[derive(StructOpt, Clone, Debug, PartialEq, Eq)]
pub struct CliEnvBlock {
    #[structopt(long = "env.block.gas_limit")]
    pub block_gas_limit: Option<u64>,
    /// somebody call it nonce
    #[structopt(long = "env.block.number")]
    pub number: Option<u64>,
    /// Coinbase or miner or address that created and signed the block.
    /// Address where we are going to send gas spend
    #[structopt(long = "env.block.coinbase")]
    pub coinbase: Option<Address>,
    #[structopt(long = "env.block.timestamp")]
    pub timestamp: Option<u64>,
    #[structopt(long = "env.block.difficulty")]
    pub difficulty: Option<u64>,
    /// basefee is added in EIP1559 London upgrade
    #[structopt(long = "env.block.basefee")]
    pub basefee: Option<u64>,
}

#[derive(StructOpt, Clone, Debug, PartialEq, Eq)]
pub struct CliEnvTx {
    /// Caller or Author or tx signer
    #[structopt(long = "env.tx.caller")]
    pub caller: Option<Address>,
    #[structopt(long = "env.tx.gas_limit")]
    pub tx_gas_limit: Option<u64>,
    #[structopt(long = "env.tx.gas_price")]
    pub gas_price: Option<u64>,
    #[structopt(long = "env.tx.gas_priority_fee")]
    pub gas_priority_fee: Option<u64>,
    #[structopt(long = "env.tx.to")]
    pub transact_to: Option<Address>,
    #[structopt(long = "env.tx.value")]
    pub value: Option<u64>,
    #[structopt(long = "env.tx.data")]
    pub data: Option<Bytes>,
    #[structopt(long = "env.tx.chain_id")]
    pub chain_id: Option<u64>,
    #[structopt(long = "env.tx.nonce")]
    pub nonce: Option<u64>,
    //#[structopt(long = "env.")]
    //TODO pub access_list: Vec<(Address, Vec<U256>)>,
}
