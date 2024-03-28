use std::path::PathBuf;
use structopt::StructOpt;

/// Statetest command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// EOF bytecode in hex format. It bytes start with 0xFE it will be interpreted as a EOF.
    /// Otherwise, it will be interpreted as a EOF bytecode.
    #[structopt(required = true)]
    bytes: Vec<u8>,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), TestError> {
        
    }
}
