mod cmd;
mod debugger;
mod exec;
mod runner;
mod statetest;
use structopt::StructOpt;

pub fn main() {
    // TODO
    // full env should be cfg

    let cmd = cmd::MainCmd::from_args();
    println!("args:{:?}", cmd);
    cmd.run()
}
