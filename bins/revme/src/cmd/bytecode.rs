use clap::Parser;
use revm::{
    bytecode::Eof,
    interpreter::{
        analysis::{validate_eof_inner, CodeType, EofError},
        opcode::eof_printer::print_eof_code,
    },
    primitives::{Bytes, MAX_INITCODE_SIZE},
};
use std::io;

/// `bytecode` subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    /// Is EOF code in INITCODE mode.
    #[arg(long)]
    eof_initcode: bool,
    /// Is EOF code in RUNTIME mode.
    #[arg(long)]
    eof_runtime: bool,
    /// Bytecode in hex format. If bytes start with 0xFE it will be interpreted as a EOF.
    /// Otherwise, it will be interpreted as a EOF bytecode.
    /// If not provided, it will operate in interactive EOF validation mode.
    #[arg()]
    bytes: Option<String>,
}

#[inline]
fn trim_decode(input: &str) -> Option<Bytes> {
    let trimmed = input.trim().trim_start_matches("0x");
    let decoded = hex::decode(trimmed).ok().map(Into::into);
    if decoded.is_none() {
        eprintln!("Invalid hex string");
        return None;
    }
    decoded
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) {
        let container_kind = if self.eof_initcode {
            Some(CodeType::ReturnContract)
        } else if self.eof_runtime {
            Some(CodeType::ReturnOrStop)
        } else {
            None
        };

        if let Some(input_bytes) = &self.bytes {
            let Some(bytes) = trim_decode(input_bytes) else {
                return;
            };

            if bytes[0] == 0xEF {
                match Eof::decode(bytes) {
                    Ok(eof) => {
                        println!("Decoding: {:#?}", eof);
                        let res = validate_eof_inner(&eof, container_kind);
                        println!("Validation: {:#?}", res);
                    }
                    Err(e) => eprintln!("Decoding Error: {:#?}", e),
                }
            } else {
                print_eof_code(&bytes)
            }
            return;
        }

        // else run command in loop.
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Input Error");
            if input.len() == 1 {
                // just a newline, so exit
                return;
            }
            let Some(bytes) = trim_decode(&input) else {
                return;
            };

            if bytes.len() > MAX_INITCODE_SIZE {
                println!(
                    "err: bytes exceeds max code size {} > {}",
                    bytes.len(),
                    MAX_INITCODE_SIZE
                );
                continue;
            }
            match Eof::decode(bytes) {
                Ok(eof) => match validate_eof_inner(&eof, container_kind) {
                    Ok(_) => {
                        println!(
                            "OK {}/{}/{}",
                            eof.body.code_section.len(),
                            eof.body.container_section.len(),
                            eof.body.data_section.len()
                        );
                    }
                    Err(eof_error) => match eof_error {
                        EofError::Decode(e) => println!("err decode: {}", e),
                        EofError::Validation(e) => println!("err validation: {}", e),
                    },
                },
                Err(e) => println!("err: {:#?}", e),
            }
        }
    }
}
