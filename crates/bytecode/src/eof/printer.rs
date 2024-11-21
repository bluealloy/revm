#![cfg(feature = "std")]

pub fn print(code: &[u8]) {
    use crate::{opcode::*, utils::read_i16};
    use primitives::hex;

    // We can check validity and jump destinations in one pass.
    let mut i = 0;
    while i < code.len() {
        let op = code[i];
        let opcode = &OPCODE_INFO[op as usize];

        let Some(opcode) = opcode else {
            println!("Unknown opcode: 0x{:02X}", op);
            i += 1;
            continue;
        };

        if opcode.immediate_size() != 0 {
            // check if the opcode immediate are within the bounds of the code
            if i + opcode.immediate_size() as usize >= code.len() {
                println!("Malformed code: immediate out of bounds");
                break;
            }
        }

        print!("{}", opcode.name());
        if opcode.immediate_size() != 0 {
            let immediate = &code[i + 1..i + 1 + opcode.immediate_size() as usize];
            print!(" : 0x{:}", hex::encode(immediate));
            if opcode.immediate_size() == 2 {
                print!(" ({})", i16::from_be_bytes(immediate.try_into().unwrap()));
            }
        }
        println!();

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
                println!("RJUMPV[{vtablei}]: 0x{offset:04X} ({offset})");
            }
        }

        i += 1 + opcode.immediate_size() as usize + rjumpv_additional_immediates;
    }
}

#[cfg(test)]
mod test {
    use primitives::hex;

    #[test]
    fn sanity_test() {
        super::print(&hex!("6001e200ffff00"));
    }
}
