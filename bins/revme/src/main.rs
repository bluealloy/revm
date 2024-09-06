use clap::Parser;
use revme::cmd::{Error, MainCmd};

fn main() -> Result<(), Error> {
    MainCmd::parse().run().inspect_err(|e| eprintln!("{e:?}"))
}
