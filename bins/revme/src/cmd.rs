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

impl MainCmd {
    pub fn run(&self) {
        match self {
            Self::Statetest(cmd) => {
                let _ = cmd.run();
            }
            Self::Debug(cmd) => {
                cmd.run();
            }
            _ => (),
        }
    }
}
