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
    #[arg(short, long)]
    pub warmup: Option<f64>,
    #[arg(short, long)]
    pub measurement_time: Option<f64>,
}

impl Cmd {
    /// Runs bench command.
    pub fn run(&self) {
        let mut criterion = criterion::Criterion::default()
            .warm_up_time(std::time::Duration::from_secs_f64(
                self.warmup.unwrap_or(10.0),
            ))
            .measurement_time(std::time::Duration::from_secs_f64(
                self.measurement_time.unwrap_or(1.0),
            ));

        println!("{:?}", self.warmup);
        println!("{:?}", self.measurement_time);

        match self.name {
            BenchName::Analysis => {
                println!("also {:?}", self.warmup);
                let mut criterion_group = criterion.benchmark_group("revme");
                analysis::run(&mut criterion_group);
                println!("also {:?}", self.warmup);
                criterion_group.finish();
                println!("this {:?}", self.warmup);
            }
            BenchName::Burntpix => {
                let mut criterion_group = criterion.benchmark_group("revme");
                burntpix::run(&mut criterion_group);
                criterion_group.finish();
            }
            BenchName::Snailtracer => {
                let mut criterion_group = criterion.benchmark_group("revme");
                snailtracer::run(&mut criterion_group);
                criterion_group.finish();
                println!("{:?}", self.warmup);
            }
            BenchName::Transfer => {
                let mut criterion_group = criterion.benchmark_group("revme");
                transfer::run(&mut criterion_group);
                criterion_group.finish();
                println!("{:?}", self.warmup);
            }
        }
        println!("{:?}", self.warmup);
        println!("{:?}", self.measurement_time);
    }
}
