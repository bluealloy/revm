pub mod analysis;
pub mod burntpix;
pub mod snailtracer;
pub mod transfer;

use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum BenchName {
    Analysis,
    Burntpix,
    Snailtracer,
    Transfer,
}

/// `bytecode` subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[arg(value_enum)]
    name: BenchName,
}

impl Cmd {
    /// Run bench command.
    pub fn run(&self) {
        match self.name {
            BenchName::Analysis => analysis::run(),
            BenchName::Burntpix => burntpix::run(),
            BenchName::Snailtracer => snailtracer::run(),
            BenchName::Transfer => transfer::run(),
        }
    }
}
