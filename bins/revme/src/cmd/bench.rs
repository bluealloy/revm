pub mod analysis;
pub mod burntpix;
pub mod evm_build;
pub mod snailtracer;
pub mod transfer;

use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum BenchName {
    Analysis,
    Burntpix,
    Snailtracer,
    Transfer,
    EvmBuild,
}

impl BenchName {
    pub const ALL: &[BenchName] = &[
        BenchName::Analysis,
        BenchName::Burntpix,
        BenchName::Snailtracer,
        BenchName::Transfer,
        BenchName::EvmBuild,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            BenchName::Analysis => "analysis",
            BenchName::Burntpix => "burntpix",
            BenchName::Snailtracer => "snailtracer",
            BenchName::Transfer => "transfer",
            BenchName::EvmBuild => "evm-build",
        }
    }
}

/// `bytecode` subcommand
#[derive(Parser, Debug)]
pub struct Cmd {
    #[arg(value_enum)]
    pub name: BenchName,
    /// Warmup represents warm up time for benchmarks ran
    #[arg(short = 'w', long)]
    pub warmup: Option<f64>,
    /// Samples represents default measurement time for benchmarks ran
    #[arg(short = 'm', long)]
    pub time: Option<f64>,
    /// Samples represents size of the sample for benchmarks ran
    #[arg(short = 's', long)]
    pub samples: Option<usize>,
}

impl Cmd {
    /// Runs bench command.
    pub fn run(&self) {
        let mut criterion = criterion::Criterion::default()
            .warm_up_time(std::time::Duration::from_secs_f64(
                self.warmup.unwrap_or(0.5),
            ))
            // Measurement_time of 0.1 will get 500+ iterations for analysis and transfer and will be extended if needed in order to test the given sample size (minimum sample size is 10 per criterion documentation) as is the case with burntpix and snailtracer benchmark tests
            .measurement_time(std::time::Duration::from_secs_f64(self.time.unwrap_or(1.5)))
            .sample_size(self.samples.unwrap_or(10));

        match self.name {
            BenchName::Analysis => {
                analysis::run(&mut criterion);
            }
            BenchName::Burntpix => {
                burntpix::run(&mut criterion);
            }
            BenchName::Snailtracer => {
                snailtracer::run(&mut criterion);
            }
            BenchName::Transfer => {
                transfer::run(&mut criterion);
            }
            BenchName::EvmBuild => {
                evm_build::run(&mut criterion);
            }
        }
    }
}
