use cmd::Error;
use structopt::StructOpt;

mod cli_env;
mod cmd;
mod statetest;
mod build;

pub fn main() -> Result<(), Error> {
    let cmd = cmd::MainCmd::from_args();
    cmd.run()
}
