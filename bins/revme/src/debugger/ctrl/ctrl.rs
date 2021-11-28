use std::path::PathBuf;

use bytes::Bytes;

use primitive_types::H160;
use revm::{Gas, Inspector, Return};

use termwiz::lineedit::*;

use super::cli::CtrlCli;

#[derive(Debug)]
pub enum Ctrl {
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

pub fn parse_address(add: &str, info: &str) -> Option<H160> {
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

impl Ctrl {
    fn next(state: StateMachine, history_path: &Option<PathBuf>) -> Self {
        match state {
            StateMachine::TriggerBreakpoint | StateMachine::TriggerStep => {
                Self::parse_inside_trigger(history_path)
            }
        }
    }

    pub fn parse_inside_trigger(history_path: &Option<PathBuf>) -> Ctrl {
        let mut terminal = line_editor_terminal().unwrap();
        let mut editor = LineEditor::new(&mut terminal);

        let mut host = CtrlCli::new(history_path.clone());
        loop {
            if let Some(line) = editor.read_line(&mut host).unwrap() {
                if line == "exit" {
                    break Ctrl::Exit;
                }

                host.history().add(&line);
                let words: Vec<&str> = line.split_whitespace().collect();
                let mut len = words.len();
                check_word!(len, 1, "Command empty. Use help");
                return match words[0] {
                    "exit" => Ctrl::Exit,
                    "step" => Ctrl::Step,
                    "continue" => Ctrl::Continue,
                    "breakpoint" => {
                        check_word!(len,2,"Error: Not enought args for breakpoint. expected breakpoint <address> <program counter>");
                        let add = match parse_address(words[1], "First argument of breakpoint") {
                            Some(add) => add,
                            None => continue,
                        };
                        let pc = match words[2].parse::<usize>() {
                            Ok(t) => t,
                            Err(e) => {
                                println!(
                                "Error: Second argument of breakpoint is not usize. std_errror:{}",
                                e
                            );
                                continue;
                            }
                        };

                        if len != 0 {
                            let ignored = &words[(words.len() - len)..];
                            println!("Ignoring:{:?}", ignored);
                        }

                        Ctrl::Breakpoint(add, pc)
                    }
                    //"insert" => Ctrl::InsertAccount,
                    t => {
                        println!("Command {:?} not found. Use help", t);
                        continue;
                    }
                };
            }
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
    history_path: Option<PathBuf>,
}

impl Controller {
    pub fn new(history_path: Option<PathBuf>) -> Self {
        Self {
            pc: 0,
            machine: StateMachine::TriggerStep,
            history_path,
        }
    }
}

impl<DB> Inspector<DB> for Controller {
    fn step(&mut self, _machine: &mut revm::Machine) {
        match Ctrl::next(self.machine, &self.history_path) {
            Ctrl::Exit => todo!(),
            Ctrl::Step => self.machine = StateMachine::TriggerStep,
            Ctrl::Continue => self.machine = StateMachine::TriggerBreakpoint,
            Ctrl::Breakpoint(add, pc) => println!(
                "Setting breakpoint for contract {} on program counter:{}",
                add, pc
            ),
            Ctrl::InsertAccount => (),
        }
    }

    fn eval(&mut self, _eval: revm::Return, _machine: &mut revm::Machine) {}

    fn load_account(&mut self, _address: &primitive_types::H160) {}

    fn sload(
        &mut self,
        _address: &primitive_types::H160,
        _slot: &primitive_types::U256,
        _value: &primitive_types::U256,
        _is_cold: bool,
    ) {
    }

    fn sstore(
        &mut self,
        _address: primitive_types::H160,
        _slot: primitive_types::U256,
        _new_value: primitive_types::U256,
        _old_value: primitive_types::U256,
        _original_value: primitive_types::U256,
        _is_cold: bool,
    ) {
    }

    fn call(
        &mut self,
        _env: &mut revm::Env,
        _subroutine: &mut revm::SubRoutine,
        _: &mut DB,
        _call: primitive_types::H160,
        _context: &revm::CallContext,
        _transfer: &revm::Transfer,
        _input: &bytes::Bytes,
        _gas_limit: u64,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn call_return(&mut self, _exit: revm::Return) {}

    fn create(
        &mut self,
        _caller: primitive_types::H160,
        _scheme: &revm::CreateScheme,
        _value: primitive_types::U256,
        _init_code: &bytes::Bytes,
        _gas: u64,
    ) {
    }

    fn create_return(&mut self, _address: primitive_types::H256) {}

    fn selfdestruct(&mut self) {}
}
