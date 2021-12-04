use std::{fs::OpenOptions, path::PathBuf};

use primitive_types::{H160, U256};
use revm::{db::Web3DB, TransactTo, EVM};
use std::str::FromStr;
use structopt::StructOpt;

use super::ctrl::Controller;

#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// specify web3 interface that we will fetch data from
    #[structopt(long)]
    web3: String,
    #[structopt(long)]
    block: Option<u64>,
    /// File where CLI history is going to be saved. if not set history will not be flushed to file.
    #[structopt(long, parse(from_os_str))]
    history: Option<PathBuf>,
}

impl Cmd {
    pub fn run(&self) {
        //https://mainnet.infura.io/v3/0954246eab5544e89ac236b668980810
        let db = Web3DB::new(&self.web3, self.block).unwrap();

        let mut revm = EVM::new();
        revm.database(db);
        revm.env.cfg.perf_all_precompiles_have_balance = true;
        // https://etherscan.io/tx/0x868942b2ba5dcb1e8fbb016d59b1ec1a3acab132d55a48212ba36d91f0c1bae6
        revm.env.tx.caller = H160::from_str("0xee0235eb8602dac2830a878593c29a954aa617a0").unwrap();
        revm.env.tx.value = U256::from(100000000);
        revm.env.tx.transact_to =
            TransactTo::Call(H160::from_str("0x7a250d5630b4cf539739df2c5dacb4c659f2488d").unwrap());
        revm.env.tx.data = hex::decode("7ff36ab50000000000000000000000000000000000000000000000bf09b200842a36c90d0000000000000000000000000000000000000000000000000000000000000080000000000000000000000000ee0235eb8602dac2830a878593c29a954aa617a00000000000000000000000000000000000000000000000000000000061aab4dd0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000ca7b3ba66556c4da2e2a9afef9c64f909a59430a").unwrap().into();

        // touch history file
        if let Some(ref history) = self.history {
            match OpenOptions::new().create(true).write(true).open(history) {
                Ok(_) => (),
                Err(e) => panic!("History file ({:?}) coudn't be touched", e),
            }
        }

        //let input: Bytes = hex::decode(&args[1]).unwrap().into();
        println!(
            "STATE OUT:{:?}",
            revm.inspect(Controller::new(self.history.clone()))
        );
    }
}
