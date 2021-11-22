use std::io::stdin;

use bytes::Bytes;
use primitive_types::H160;
use revm::{Gas, Inspector, Return};

#[derive(Debug)]
pub enum Control {
    Exit,
    Step,
    Continue,
    Breakpoint(H160, usize),
    InsertAccount,
    // RewindCall,
    // RewindOpcode,
    // Stack,
    // StackSet,
    // Memory,
    // MemorySet,
    // Account,
    // AccountSetBalance,
    // AccountSetNonce,
    // Storage,
    // StorageSet,
}

macro_rules! check_word {
    ($len:ident, $pop:expr,$error:expr) => {
        if $pop > $len {
            println!("{}", $error);
            continue;
        }
        $len -= $pop;
    };
}

pub fn parse_add(add: &str, info: &str) -> Option<H160> {
    if add.len() < 40 {
        println!("Error: :{} not big enough. Expect 20bytes hex ", &info);
        return None;
    }
    let add = if &add[0..2] == "0x" { &add[2..] } else { add };
    let add = match hex::decode(add) {
        Ok(hex) => H160::from_slice(&hex),
        Err(_) => {
            println!("Error: {} not in hex format", &info);

            return None;
        }
    };
    Some(add)
}

impl Control {
    fn next(state: StateMachine) -> Self {
        match state {
            StateMachine::TriggerBreakpoint | StateMachine::TriggerStep => {
                Self::parse_inside_trigger()
            }
        }
    }

    pub fn parse_inside_trigger() -> Control {
        'main: loop {
            let mut buffer = String::new();
            let _ = stdin().read_line(&mut buffer);

            let buffer = buffer.trim_end();
            let words: Vec<&str> = buffer.split_whitespace().collect();
            let mut len = words.len();
            check_word!(len, 1, "Command empty. Use help");
            return match words[0] {
                "exit" => Control::Exit,
                "step" => Control::Step,
                "continue" => Control::Continue,
                "breakpoint" => {
                    check_word!(len,2,"Error: Not enought args for breakpoint. expected breakpoint <address> <program counter>");
                    let add = match parse_add(words[1], "First argument of breakpoint") {
                        Some(add) => add,
                        None => continue 'main,
                    };
                    let pc = match words[2].parse::<usize>() {
                        Ok(t) => t,
                        Err(e) => {
                            println!("Error: Second argument of breakpoint is not usize. std_errror:{}",e);
                            continue 'main;
                        }
                    };

                    Control::Breakpoint(add, pc)
                }
                //"insert" => Control::InsertAccount,
                t => {
                    println!("Command {:?} not found. Use help", t);
                    continue;
                }
            };
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateMachine {
    TriggerStep,
    TriggerBreakpoint,
}

pub struct Controller {
    pc: usize,
    machine: StateMachine,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            pc: 0,
            machine: StateMachine::TriggerStep,
        }
    }
}

impl<DB> Inspector<DB> for Controller {
    fn step(&mut self, machine: &mut revm::Machine) {
        match Control::next(self.machine) {
            Control::Exit => todo!(),
            Control::Step => self.machine = StateMachine::TriggerStep,
            Control::Continue => self.machine = StateMachine::TriggerBreakpoint,
            Control::Breakpoint(add, pc) => println!(
                "Setting breakpoint for contract {} on program counter:{}",
                add, pc
            ),
            Control::InsertAccount => (),
        }
    }

    fn eval(&mut self, eval: revm::Return, machine: &mut revm::Machine) {}

    fn load_account(&mut self, address: &primitive_types::H160) {}

    fn sload(
        &mut self,
        address: &primitive_types::H160,
        slot: &primitive_types::U256,
        value: &primitive_types::U256,
        is_cold: bool,
    ) {
    }

    fn sstore(
        &mut self,
        address: primitive_types::H160,
        slot: primitive_types::U256,
        new_value: primitive_types::U256,
        old_value: primitive_types::U256,
        original_value: primitive_types::U256,
        is_cold: bool,
    ) {
    }

    fn call(
        &mut self,
        env: &mut revm::Env,
        subroutine: &mut revm::SubRoutine,
        _: &mut DB,
        call: primitive_types::H160,
        context: &revm::CallContext,
        transfer: &revm::Transfer,
        input: &bytes::Bytes,
        gas_limit: u64,
        is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn call_return(&mut self, exit: revm::Return) {}

    fn create(
        &mut self,
        caller: primitive_types::H160,
        scheme: &revm::CreateScheme,
        value: primitive_types::U256,
        init_code: &bytes::Bytes,
        gas: u64,
    ) {
    }

    fn create_return(&mut self, address: primitive_types::H256) {}

    fn selfdestruct(&mut self) {}
}
