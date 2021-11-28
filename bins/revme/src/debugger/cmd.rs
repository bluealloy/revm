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
    /// File where CLI history is going to be saved. if not set history will not be flushed to file.
    #[structopt(long, parse(from_os_str))]
    history: Option<PathBuf>,
}

impl Cmd {
    pub fn run(&self) {
        //https://mainnet.infura.io/v3/0954246eab5544e89ac236b668980810
        let db = Web3DB::new(&self.web3, None).unwrap();

        let mut revm = EVM::new();
        revm.database(db);
        revm.env.cfg.perf_all_precompiles_have_balance = true;
        revm.env.tx.caller = H160::from_str("0x393616975ff5A88AAB4568983C1dcE96FBb5b67a").unwrap();
        revm.env.tx.value = U256::from(11234);
        revm.env.tx.transact_to =
            TransactTo::Call(H160::from_str("0x393616975ff5A88AAB4568983C1dcE96FBb5b67b").unwrap());
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
