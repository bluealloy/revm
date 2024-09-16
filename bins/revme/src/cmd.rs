pub mod bytecode;
pub mod eofvalidation;
pub mod evmrunner;
pub mod statetest;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(infer_subcommands = true)]
#[allow(clippy::large_enum_variant)]
pub enum MainCmd {
    /// Execute Ethereum state tests.
    Statetest(statetest::Cmd),
    /// Execute eof validation tests.
    EofValidation(eofvalidation::Cmd),
    /// Run arbitrary EVM bytecode.
    Evm(evmrunner::Cmd),
    /// Print the structure of an EVM bytecode.
    Bytecode(bytecode::Cmd),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Statetest(#[from] statetest::Error),
    #[error(transparent)]
    EvmRunnerErrors(#[from] evmrunner::Errors),
    #[error("Eof validation failed: {:?}/{total_tests}", total_tests-failed_test)]
    EofValidation {
        failed_test: usize,
        total_tests: usize,
    },
    #[error("Custom error: {0}")]
    Custom(&'static str),
}

impl MainCmd {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Self::Statetest(cmd) => cmd.run().map_err(Into::into),
            Self::EofValidation(cmd) => cmd.run().map_err(Into::into),
            Self::Evm(cmd) => cmd.run().map_err(Into::into),
            Self::Bytecode(cmd) => {
                cmd.run();
                Ok(())
            }
        }
    }
}
