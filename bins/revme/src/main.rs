use revme::cmd::MainCmd;
use structopt::StructOpt;

#[cfg(feature = "fluent_revm")]
pub extern crate revm_fluent as revm;

pub fn main() {
    let cmd = MainCmd::from_args();
    if let Err(e) = cmd.run() {
        println!("{}", e)
    }
}
