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

impl BenchName {
    pub const ALL: &[BenchName] = &[
        BenchName::Analysis,
        BenchName::Burntpix,
        BenchName::Snailtracer,
        BenchName::Transfer,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            BenchName::Analysis => "analysis",
            BenchName::Burntpix => "burntpix",
            BenchName::Snailtracer => "snailtracer",
            BenchName::Transfer => "transfer",
        }
    }
}

/// `bytecode` subcommand
#[derive(Parser, Debug)]
pub struct Cmd {
    #[arg(value_enum)]
    pub name: BenchName,
}

impl Cmd {
    /// Runs bench command.
    pub fn run(&self) {
        match self.name {
            BenchName::Analysis => analysis::run(),
            BenchName::Burntpix => burntpix::run(),
            BenchName::Snailtracer => snailtracer::run(),
            BenchName::Transfer => transfer::run(),
        }
    }
}
