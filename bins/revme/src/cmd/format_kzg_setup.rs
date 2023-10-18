pub use revm::primitives::kzg::KzgErrors as Error;
pub use revm::primitives::kzg::{format_kzg_settings, G1Points, G2Points};

use std::path::PathBuf;
use structopt::StructOpt;

/// Statetest command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// Input path to the kzg trusted setup file.
    #[structopt(required = true)]
    path: PathBuf,
    /// path to output g1 point in binary format.
    #[structopt(short = "g1", long)]
    g1: bool,
    /// Path to output g2 point in binary format.
    #[structopt(long)]
    g2: bool,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), Error> {
        Ok(())
    }
}
