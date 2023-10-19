pub mod format_kzg_setup;
pub mod statetest;

use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::InferSubcommands)]
#[allow(clippy::large_enum_variant)]
pub enum MainCmd {
    #[structopt(about = "Launch Ethereum state tests")]
    Statetest(statetest::Cmd),
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
    KzgErrors(#[from] format_kzg_setup::KzgErrors),
}

impl MainCmd {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Self::Statetest(cmd) => cmd.run().map_err(Into::into),
            Self::FormatKzgSetup(cmd) => cmd.run().map_err(Into::into),
        }
    }
}
