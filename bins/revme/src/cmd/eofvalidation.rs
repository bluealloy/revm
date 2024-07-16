use crate::cmd::Error;
use std::path::PathBuf;
use structopt::StructOpt;

/// Statetest command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// Input path to the kzg trusted setup file.
    #[structopt(required = true)]
    path: PathBuf,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), Error> {
        // check if path exists.
        if !self.path.exists() {
            return Err(Error::Custom("The specified path does not exist"));
        }
        Ok(())
    }
}
