use structopt::{clap::AppSettings, StructOpt};

use crate::{debugger, runner, statetest};

#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::InferSubcommands)]
#[allow(clippy::large_enum_variant)]
pub enum MainCmd {
    Statetest(statetest::Cmd),
    Debug(debugger::Cmd),
    Run(runner::Cmd),
}

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Statetest: {0}")]
    Statetest(statetest::Error),
    #[error("Generic system error")]
    SystemError,
}

impl MainCmd {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Self::Statetest(cmd) => cmd.run().map_err(Error::Statetest),
            Self::Debug(cmd) => {
                cmd.run();
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
