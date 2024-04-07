use revm::{
    interpreter::opcode::eof_printer::print_eof_code,
    primitives::{Bytes, Eof},
};
use structopt::StructOpt;

/// Statetest command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// EOF bytecode in hex format. It bytes start with 0xFE it will be interpreted as a EOF.
    /// Otherwise, it will be interpreted as a EOF bytecode.
    #[structopt(required = true)]
    bytes: String,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) {
        let trimmed = self.bytes.trim_start_matches("0x");
        let Ok(bytes) = hex::decode(trimmed) else {
            eprintln!("Invalid hex string");
            return;
        };
        let bytes: Bytes = bytes.into();
        if bytes.is_empty() {
            eprintln!("Empty hex string");
            return;
        }
        if bytes[0] == 0xEF {
            let Ok(eof) = Eof::decode(bytes) else {
                eprintln!("Invalid EOF bytecode");
                return;
            };
            println!("{:#?}", eof);
        } else {
            print_eof_code(&bytes)
        }
    }
}
