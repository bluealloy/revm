use cmd::Error;
use structopt::StructOpt;

mod build;
mod cmd;

pub fn main() -> Result<(), Error> {
    let cmd = cmd::MainCmd::from_args();
    cmd.run()
}
