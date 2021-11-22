use structopt::{clap::AppSettings, StructOpt};

use crate::{debugger, runner};//statetest};

#[derive(StructOpt, Debug)]
// https://docs.rs/clap/2/clap/enum.AppSettings.html#variant.InferSubcommands
#[structopt(setting = AppSettings::InferSubcommands)]
pub enum MainCmd {
    //Statetest(statetest::Cmd),
    Debug(debugger::Cmd),
    Run(runner::Cmd),
}

impl MainCmd {
    pub fn run(&self) {
        match self {
            // Self::Statetest(cmd) => {
            //     let _ = cmd.run();
            // }
            Self::Debug(cmd) => {
                cmd.run();
            }
            _ => (),
        }
    }
}
