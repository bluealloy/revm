use std::io;

use structopt::StructOpt;

use revm::{
    interpreter::{analysis::validate_eof_inner, opcode::eof_printer::print_eof_code},
    primitives::{Bytes, Eof},
};
use revm::interpreter::analysis::CodeType::ReturnOrStop;
use revm::interpreter::analysis::EofError;

/// Statetest command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// Bytecode in hex format. If bytes start with 0xFE it will be interpreted as a EOF.
    /// Otherwise, it will be interpreted as a EOF bytecode.
    /// If not provided, it will operate in interactive EOF validation mode.
    #[structopt(default_value = "")]
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
            loop {
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Input Error");
                if input.len() == 1 {
                    // just a newline, so exit
                    break;
                }
                let trimmed = input.trim().trim_start_matches("0x");
                let Ok(bytes) = hex::decode(trimmed) else {
                    println!("fail: Invalid hex string");
                    return;
                };
                let bytes: Bytes = bytes.into();
                match Eof::decode(bytes) {
                    Ok(eof) => {
                        match validate_eof_inner(&eof, Option::from(ReturnOrStop)) {
                            Ok(_) => {
                                println!("OK {}/{}/{}", eof.body.code_section.len(), eof.body.container_section.len(), eof.body.data_section.len());
                            }
                            Err(eof_error) => {
                                match eof_error {
                                    EofError::Decode(e) => println!("err decode: {}", e),
                                    EofError::Validation(e) => println!("err validation: {}", e),
                                }
                            }
                        }
                    }
                    Err(e) => println!("err: {:#?}", e),
                }
            }
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
