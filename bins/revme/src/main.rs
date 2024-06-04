use revme::cmd::{Error, MainCmd};
use structopt::StructOpt;

#[cfg(feature = "revm-rwasm")]
pub extern crate revm_fluent as revm;

pub fn main() -> Result<(), Error> {
    let cmd = MainCmd::from_args();
    if let Err(e) = cmd.run() {
        println!("{:?}", e);
        return Err(e);
    }
    Ok(())
}
