use revme::cmd::MainCmd;
use structopt::StructOpt;

pub fn main() {
    let cmd = MainCmd::from_args();
    if let Err(e) = cmd.run() {
        println!("{}", e)
    }
}
