pub mod bench;
pub mod blockchaintest;
pub mod bytecode;
pub mod evmrunner;
pub mod statetest;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(infer_subcommands = true)]
#[allow(clippy::large_enum_variant)]
pub enum MainCmd {
    /// Execute Ethereum state tests.
    Statetest(statetest::Cmd),
    /// Execute Ethereum state tests.
    Stest(statetest::Cmd),
    /// Run arbitrary EVM bytecode.
    Evm(evmrunner::Cmd),
    /// Print the structure of an EVM bytecode.
    Bytecode(bytecode::Cmd),
    /// Run bench from specified list.
    Bench(bench::Cmd),
    /// Execute Ethereum blockchain tests.
    Blockchaintest(blockchaintest::Cmd),
    /// Execute Ethereum blockchain tests.
    Btest(blockchaintest::Cmd),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Statetest(#[from] statetest::Error),
    #[error(transparent)]
    Blockchaintest(#[from] blockchaintest::Error),
    #[error(transparent)]
    EvmRunnerErrors(#[from] evmrunner::Errors),
    #[error("Custom error: {0}")]
    Custom(&'static str),
}

impl MainCmd {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Self::Statetest(cmd) | Self::Stest(cmd) => cmd.run()?,
            Self::Evm(cmd) => cmd.run()?,
            Self::Bytecode(cmd) => {
                cmd.run()?;
            }
            Self::Bench(cmd) => {
                cmd.run();
            }
            Self::Blockchaintest(cmd) | Self::Btest(cmd) => cmd.run()?,
        }
        Ok(())
    }
}
