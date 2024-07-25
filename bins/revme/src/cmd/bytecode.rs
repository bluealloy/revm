use revm::{
    interpreter::{analysis::validate_eof_inner, opcode::eof_printer::print_eof_code},
    primitives::{Bytes, Eof},
};
use structopt::StructOpt;

/// Statetest command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// Bytecode in hex format. If bytes start with 0xFE it will be interpreted as a EOF.
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
            match Eof::decode(bytes) {
                Ok(eof) => {
                    println!("Decoding: {:#?}", eof);
                    let res = validate_eof_inner(&eof, None);
                    println!("Validation: {:#?}", res);
                }
                Err(e) => eprintln!("Decoding Error: {:#?}", e),
            }
        } else {
            print_eof_code(&bytes)
        }
    }
}
