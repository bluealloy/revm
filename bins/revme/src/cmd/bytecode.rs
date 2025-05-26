use bytecode::{validate_eof_inner, Eof};
use clap::Parser;
use revm::{
    bytecode::eof::{self, validate_raw_eof_inner, CodeType, EofError},
    primitives::{hex, Bytes},
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
    /// Bytecode in hex format string.
    ///
    /// - If bytes start with 0xFE it will be interpreted as a EOF.
    /// - Otherwise, it will be interpreted as a EOF bytecode.
    /// - If not provided, it will operate in interactive EOF validation mode.
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
    /// Runs statetest command.
    pub fn run(&self) {
        let container_kind = if self.eof_initcode {
            Some(CodeType::Initcode)
        } else if self.eof_runtime {
            Some(CodeType::Runtime)
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
                        match validate_eof_inner(&eof, container_kind) {
                            Ok(_) => {
                                println!("Validation: OK");
                            }
                            Err(eof_error) => {
                                eprintln!("Validation error: {}", eof_error);
                            }
                        }
                        println!("Validation: OK");
                    }
                    Err(eof_error) => {
                        eprintln!("Decoding error: {}", eof_error);
                    }
                }
            } else {
                eof::printer::print(&bytes)
            }
            return;
        }

        // Else run command in loop.
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Input Error");
            if input.len() == 1 {
                // Just a newline, so exit
                return;
            }
            let Some(bytes) = trim_decode(&input) else {
                return;
            };
            match validate_raw_eof_inner(bytes, container_kind) {
                Ok(eof) => {
                    println!(
                        "OK {}/{}/{}",
                        eof.body.code_section.len(),
                        eof.body.container_section.len(),
                        eof.body.data_section.len()
                    );
                }
                Err(eof_error) => {
                    if matches!(
                        eof_error,
                        EofError::Decode(eof::EofDecodeError::InvalidEOFSize)
                    ) {
                        continue;
                    }
                    println!("err: {}", eof_error);
                }
            }
        }
    }
}
