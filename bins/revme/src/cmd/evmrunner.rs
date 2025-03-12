use clap::Parser;
use database::BenchmarkDB;
use inspector::{inspectors::TracerEip3155, InspectEvm};
use revm::{
    bytecode::{Bytecode, BytecodeDecodeError},
    primitives::{address, hex, Address, TxKind},
    Context, Database, ExecuteEvm, MainBuilder, MainContext,
};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::time::Duration;
use std::{borrow::Cow, fs};

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
    Io(#[from] IoError),
    #[error(transparent)]
    BytecodeDecodeError(#[from] BytecodeDecodeError),
}

/// Evm runner command allows running arbitrary evm bytecode
///
/// Bytecode can be provided from cli or from file with `--path` option.
#[derive(Parser, Debug)]
pub struct Cmd {
    /// Hex-encoded EVM bytecode to be executed
    #[arg(required_unless_present = "path")]
    bytecode: Option<String>,
    /// Path to a file containing the hex-encoded EVM bytecode to be executed
    ///
    /// Overrides the positional `bytecode` argument.
    #[arg(long)]
    path: Option<PathBuf>,
    /// Whether to run in benchmarking mode
    #[arg(long)]
    bench: bool,
    /// Hex-encoded input/calldata bytes
    #[arg(long, default_value = "")]
    input: String,
    /// Whether to print the state
    #[arg(long)]
    state: bool,
    /// Whether to print the trace
    #[arg(long)]
    trace: bool,
}

impl Cmd {
    /// Runs evm runner command.
    pub fn run(&self) -> Result<(), Errors> {
        const CALLER: Address = address!("0000000000000000000000000000000000000001");

        let bytecode_str: Cow<'_, str> = if let Some(path) = &self.path {
            // Check if path exists.
            if !path.exists() {
                return Err(Errors::PathNotExists);
            }
            fs::read_to_string(path)?.into()
        } else if let Some(bytecode) = &self.bytecode {
            bytecode.as_str().into()
        } else {
            unreachable!()
        };

        let bytecode = hex::decode(bytecode_str.trim()).map_err(|_| Errors::InvalidBytecode)?;
        let input = hex::decode(self.input.trim())
            .map_err(|_| Errors::InvalidInput)?
            .into();

        let mut db = BenchmarkDB::new_bytecode(Bytecode::new_raw_checked(bytecode.into())?);

        let nonce = db.basic(CALLER).unwrap().map_or(0, |account| account.nonce);

        // BenchmarkDB is dummy state that implements Database trait.
        // The bytecode is deployed at zero address.
        let mut evm = Context::mainnet()
            .with_db(db)
            .modify_tx_chained(|tx| {
                tx.caller = CALLER;
                tx.kind = TxKind::Call(Address::ZERO);
                tx.data = input;
                tx.nonce = nonce;
            })
            .build_mainnet_with_inspector(TracerEip3155::new(Box::new(std::io::stdout())));

        if self.bench {
            // Microbenchmark
            let bench_options = microbench::Options::default().time(Duration::from_secs(3));

            microbench::bench(&bench_options, "Run bytecode", || {
                let _ = evm.replay().unwrap();
            });

            return Ok(());
        }

        let out = if self.trace {
            evm.inspect_replay().map_err(|_| Errors::EVMError)?
        } else {
            let out = evm.replay().map_err(|_| Errors::EVMError)?;
            println!("Result: {:#?}", out.result);
            out
        };

        if self.state {
            println!("State: {:#?}", out.state);
        }

        Ok(())
    }
}
