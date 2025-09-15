use clap::Parser;
use revm::primitives::{hex, Bytes};

/// `bytecode` subcommand - simplified to handle legacy bytecode only.
#[derive(Parser, Debug)]
pub struct Cmd {
    /// Bytecode in hex format string.
    #[arg()]
    bytes: Option<String>,
}

#[inline]
fn trim_decode(input: &str) -> Option<Bytes> {
    let trimmed = input.trim().trim_start_matches("0x");
    hex::decode(trimmed).ok().map(Into::into)
}

impl Cmd {
    /// Runs bytecode command.
    pub fn run(&self) -> Result<(), super::Error> {
        if let Some(input_bytes) = &self.bytes {
            let Some(bytes) = trim_decode(input_bytes) else {
                // Fail on invalid hex to propagate a non-zero exit code
                return Err(super::Error::Custom("Invalid hex string"));
            };

            if bytes.starts_with(&[0xEF, 0x00]) {
                // Fail on EOF bytecode as it's not supported
                return Err(super::Error::Custom(
                    "EOF bytecode is not supported - EOF has been removed from ethereum plan.",
                ));
            }

            println!("Legacy bytecode:");
            println!("  Length: {} bytes", bytes.len());
            println!("  Hex: 0x{}", hex::encode(&bytes));

            // Basic analysis
            let mut opcodes = Vec::new();
            let mut i = 0;
            while i < bytes.len() {
                let opcode = bytes[i];
                opcodes.push(format!("{opcode:02x}"));

                // Skip immediate bytes for PUSH instructions
                if (0x60..=0x7f).contains(&opcode) {
                    let push_size = (opcode - 0x5f) as usize;
                    i += push_size;
                }
                i += 1;
            }

            println!("  Opcodes: {}", opcodes.join(" "));
        } else {
            println!("No bytecode provided. EOF interactive mode has been removed.");
            println!("Please provide bytecode as a hex string argument.");
        }
        Ok(())
    }
}
