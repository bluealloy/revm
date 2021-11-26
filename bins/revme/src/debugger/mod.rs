use std::str::FromStr;

use primitive_types::{H160, U256};
use revm::{db::Web3DB, TransactTo, EVM};
use structopt::StructOpt;

mod cmd;
mod ctrl;

use ctrl::Controller;

#[derive(StructOpt, Debug)]
pub struct Cmd {
    #[structopt(long)] //#[structopt(short, long)]
    web3: String,
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

        //let input: Bytes = hex::decode(&args[1]).unwrap().into();
        println!("STATE OUT:{:?}", revm.inspect(Controller::new()));
    }
}
