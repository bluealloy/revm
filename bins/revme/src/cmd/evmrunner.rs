use revm::{
    db::BenchmarkDB,
    inspector_handle_register,
    inspectors::TracerEip3155,
    primitives::{Address, Bytecode, TxKind},
    Evm,
};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::time::Duration;
use std::{borrow::Cow, fs};
use structopt::StructOpt;

extern crate alloc;

#[derive(Debug, thiserror::Error)]
pub enum Errors {
    #[error("The specified path does not exist")]
    PathNotExists,
    #[error("Invalid bytecode")]
    InvalidBytecode,
    #[error("Invalid input")]
    InvalidInput,
    #[error("EVM Error")]
    EVMError,
    #[error(transparent)]
    Io(IoError),
}

impl From<IoError> for Errors {
    fn from(e: IoError) -> Self {
        Errors::Io(e)
    }
}

/// Evm runner command allows running arbitrary evm bytecode.
/// Bytecode can be provided from cli or from file with --path option.
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// Bytecode to be executed.
    #[structopt(default_value = "")]
    bytecode: String,
    /// Path to file containing the evm bytecode.
    /// Overrides the bytecode option.
    #[structopt(long)]
    path: Option<PathBuf>,
    /// Run in benchmarking mode.
    #[structopt(long)]
    bench: bool,
    /// Input bytes.
    #[structopt(long, default_value = "")]
    input: String,
    /// Print the state.
    #[structopt(long)]
    state: bool,
    /// Print the trace.
    #[structopt(long)]
    trace: bool,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), Errors> {
        let bytecode_str: Cow<'_, str> = if let Some(path) = &self.path {
            // check if path exists.
            if !path.exists() {
                return Err(Errors::PathNotExists);
            }
            fs::read_to_string(path)?.to_owned().into()
        } else {
            self.bytecode.as_str().into()
        };

        let bytecode = hex::decode(bytecode_str.trim()).map_err(|_| Errors::InvalidBytecode)?;
        let input = hex::decode(self.input.trim())
            .map_err(|_| Errors::InvalidInput)?
            .into();
        // BenchmarkDB is dummy state that implements Database trait.
        // the bytecode is deployed at zero address.
        let mut evm = Evm::builder()
            .with_db(BenchmarkDB::new_bytecode(Bytecode::new_raw(
                bytecode.into(),
            )))
            .modify_tx_env(|tx| {
                // execution globals block hash/gas_limit/coinbase/timestamp..
                tx.caller = "0x0000000000000000000000000000000000000001"
                    .parse()
                    .unwrap();
                tx.transact_to = TxKind::Call(Address::ZERO);
                tx.data = input;
            })
            .build();

        if self.bench {
            // Microbenchmark
            let bench_options = microbench::Options::default().time(Duration::from_secs(3));

            microbench::bench(&bench_options, "Run bytecode", || {
                let _ = evm.transact().unwrap();
            });

            return Ok(());
        }

        let out = if self.trace {
            let mut evm = evm
                .modify()
                .reset_handler_with_external_context(TracerEip3155::new(
                    Box::new(std::io::stdout()),
                ))
                .append_handler_register(inspector_handle_register)
                .build();

            evm.transact().map_err(|_| Errors::EVMError)?
        } else {
            let out = evm.transact().map_err(|_| Errors::EVMError)?;
            println!("Result: {:#?}", out.result);
            out
        };

        if self.state {
            println!("State: {:#?}", out.state);
        }

        Ok(())
    }
}
