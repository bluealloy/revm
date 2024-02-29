use revm::{
    db::BenchmarkDB,
    primitives::{Bytecode, TransactTo},
    Evm,
};
use std::time::Duration;
use std::fs;
use std::path::PathBuf;
use core::fmt::Display;
use structopt::StructOpt;

extern crate alloc;

#[derive(Debug)]
pub enum Errors {
    PathNotExists,
    InvalidFile,
    EVMError,
}

impl std::error::Error for Errors {}
 
impl Display for Errors {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Errors::PathNotExists => write!(f, "The specified path does not exist"),
            Errors::InvalidFile => write!(f, "Invalid EVM script"),
            Errors::EVMError => write!(f, "VM error"),
        }
    }
}

/// EvmRunner command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// Path to file containing the evm script.
    #[structopt(required = true)]
    path: PathBuf,
    /// Run in benchmarking mode
    #[structopt(long)]
    bench: bool,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), Errors> {
        // check if path exists.
        if !self.path.exists() {
            return Err(Errors::PathNotExists);
        }
 

        let contents = fs::read_to_string(&self.path).map_err(|_| Errors::InvalidFile)?;
        let contents_str = contents.to_string();
        let bytecode = hex::decode(contents_str.trim()).map_err(|_| Errors::InvalidFile)?;

        let zero_address = "0x0000000000000000000000000000000000000000";

        // BenchmarkDB is dummy state that implements Database trait.
        // the bytecode is deployed at zero address.
        let mut evm = Evm::builder()
            .with_db(BenchmarkDB::new_bytecode(Bytecode::new_raw(bytecode.into())))
            .modify_tx_env(|tx| {
                // execution globals block hash/gas_limit/coinbase/timestamp..
                tx.caller = "0x0000000000000000000000000000000000000001"
                    .parse()
                    .unwrap();
                tx.transact_to = TransactTo::Call(
                    zero_address
                        .parse()
                        .unwrap(),
                );
            })
            .build();

        if self.bench {
            // Microbenchmark
            let bench_options = microbench::Options::default().time(Duration::from_secs(3));

            microbench::bench(&bench_options, "Run bytecode", || {
                let _ = evm.transact().unwrap();
            });
        } else {
            evm.transact().map_err(|_| Errors::EVMError)?;
            // TODO: print the result
        }
        Ok(())
    }
}
