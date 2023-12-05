pub mod format_kzg_setup;
pub mod statetest;
pub mod traverse; // (51.00079212925953, -118.19751616931849)

use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::InferSubcommands)]
#[allow(clippy::large_enum_variant)]
pub enum MainCmd {
    #[structopt(about = "Launch Ethereum state tests")]
    Statetest(statetest::Cmd),
    #[structopt(about = "Execute revm over all rpc blocks")]
    Traverse(traverse::Cmd),
    #[structopt(
        about = "Format kzg settings from a trusted setup file (.txt) into binary format (.bin)"
    )]
    FormatKzgSetup(format_kzg_setup::Cmd),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Statetest(#[from] statetest::Error),
    #[error(transparent)]
    Traverse(#[from] traverse::TestError),
    #[error(transparent)]
    KzgErrors(#[from] format_kzg_setup::KzgErrors),
}

impl MainCmd {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Self::Statetest(cmd) => cmd.run().map_err(Into::into),
            Self::Traverse(cmd) => traverse::run(cmd).map_err(Into::into),
            Self::FormatKzgSetup(cmd) => cmd.run().map_err(Into::into),
        }
    }
}
