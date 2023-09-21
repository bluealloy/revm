use std::path::PathBuf;
use crate::{statetest, build::{generate_kzg_settings, KzgErrors}};
use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::InferSubcommands)]
#[allow(clippy::large_enum_variant)]
pub enum MainCmd {
    #[structopt(about = "Launch Ethereum state tests")]
    Statetest(statetest::Cmd),
    #[structopt(name = "generate-kzg-points", about = "Generate kzg settings from a trusted setup file (.txt)")]
    GenerateKzgPoints {
        #[structopt(parse(from_os_str), help = "Path of the trusted setup file (.txt).")]
        path: PathBuf,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Statetest(#[from] statetest::Error),
    #[error(transparent)]
    KzgErrors(#[from] KzgErrors),
}

impl MainCmd {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Self::Statetest(cmd) => cmd.run().map_err(Into::into),
            Self::GenerateKzgPoints { path } => generate_kzg_settings(path.as_path()).map_err(Into::into),
        }
    }
}
