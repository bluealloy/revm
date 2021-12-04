mod cmd;
mod debugger;
mod exec;
mod runner;
mod statetest;
use structopt::StructOpt;
mod cli_env;

pub fn main() {
    let cmd = cmd::MainCmd::from_args();
    cmd.run()
}
