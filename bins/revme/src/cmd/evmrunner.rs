use clap::Parser;
use revm::{
    bytecode::{Bytecode, BytecodeDecodeError},
    context::TxEnv,
    database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET},
    inspector::{inspectors::TracerEip3155, InspectEvm},
    primitives::{hex, TxKind},
    Context, Database, ExecuteEvm, MainBuilder, MainContext,
};
use std::{borrow::Cow, fs, io::Error as IoError, path::PathBuf, time::Instant};

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
    /// Gas limit
    #[arg(long, default_value = "1000000000")]
    gas_limit: u64,

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

        let bytecode = hex::decode(bytecode_str.trim().trim_start_matches("0x"))
            .map_err(|_| Errors::InvalidBytecode)?;
        let input = hex::decode(self.input.trim().trim_start_matches("0x"))
            .map_err(|_| Errors::InvalidInput)?
            .into();

        let mut db = BenchmarkDB::new_bytecode(Bytecode::new_raw_checked(bytecode.into())?);

        let nonce = db
            .basic(BENCH_CALLER)
            .unwrap()
            .map_or(0, |account| account.nonce);

        // BenchmarkDB is dummy state that implements Database trait.
        // The bytecode is deployed at zero address.
        let mut evm = Context::mainnet()
            .with_db(db)
            .build_mainnet_with_inspector(TracerEip3155::new(Box::new(std::io::stdout())));

        let tx = TxEnv::builder()
            .caller(BENCH_CALLER)
            .kind(TxKind::Call(BENCH_TARGET))
            .data(input)
            .nonce(nonce)
            .gas_limit(self.gas_limit)
            .build()
            .unwrap();

        if self.bench {
            let mut criterion = criterion::Criterion::default()
                .warm_up_time(std::time::Duration::from_millis(300))
                .measurement_time(std::time::Duration::from_secs(2))
                .without_plots();
            let mut criterion_group = criterion.benchmark_group("revme");
            criterion_group.bench_function("evm", |b| {
                b.iter_batched(
                    || tx.clone(),
                    |input| evm.transact(input).unwrap(),
                    criterion::BatchSize::SmallInput,
                );
            });
            criterion_group.finish();

            return Ok(());
        }

        let time = Instant::now();
        let r = if self.trace {
            evm.inspect_tx(tx)
        } else {
            evm.transact(tx)
        }
        .map_err(|_| Errors::EVMError)?;
        let time = time.elapsed();

        println!("Result: {:#?}", r.result);
        if self.state {
            println!("State: {:#?}", r.state);
        }

        println!("Elapsed: {time:?}");
        Ok(())
    }
}
