mod cmd;
mod exec;
mod runner;
mod statetest;
pub mod tracer_eip3155;
use cmd::Error;
use structopt::StructOpt;
mod cli_env;

pub fn main() -> Result<(), Error> {
    let cmd = cmd::MainCmd::from_args();
    cmd.run()
}
