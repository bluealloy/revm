#[cfg(feature = "std")]
pub fn print_eof_code(code: &[u8]) {
    use super::*;
    use crate::instructions::utility::read_i16;
    use revm_primitives::hex;

    // We can check validity and jump destinations in one pass.
    let mut i = 0;
    while i < code.len() {
        let op = code[i];
        let opcode = &OPCODE_INFO_JUMPTABLE[op as usize];

        let Some(opcode) = opcode else {
            println!("Unknown opcode: 0x{:02X}", op);
            i += 1;
            continue;
        };

        if opcode.immediate_size != 0 {
            // check if the opcode immediate are within the bounds of the code
            if i + opcode.immediate_size as usize >= code.len() {
                println!("Malformed code: immediate out of bounds");
                break;
            }
        }

        print!("{}", opcode.name);
        if opcode.immediate_size != 0 {
            print!(
                " : 0x{:}",
                hex::encode(&code[i + 1..i + 1 + opcode.immediate_size as usize])
            );
        }

        let mut rjumpv_additional_immediates = 0;
        if op == RJUMPV {
            let max_index = code[i + 1] as usize;
            let len = max_index + 1;
            // and max_index+1 is to get size of vtable as index starts from 0.
            rjumpv_additional_immediates = len * 2;

            // +1 is for max_index byte
            if i + 1 + rjumpv_additional_immediates >= code.len() {
                println!("Malformed code: immediate out of bounds");
                break;
            }

            for vtablei in 0..len {
                let offset = unsafe { read_i16(code.as_ptr().add(i + 2 + 2 * vtablei)) } as isize;
                println!("RJUMPV[{vtablei}]: 0x{offset:04X}({offset})");
            }
        }

        i += 1 + opcode.immediate_size as usize + rjumpv_additional_immediates;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use revm_primitives::hex;

    #[test]
    fn sanity_test() {
        print_eof_code(&hex!("6001e200ffff00"));
    }
}
