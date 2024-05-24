use revme::cmd::{Error, MainCmd};
use structopt::StructOpt;

pub fn main() -> Result<(), Error> {
    let cmd = MainCmd::from_args();
    if let Err(e) = cmd.run() {
        return Err(e);
    }
    return Ok(());
}
