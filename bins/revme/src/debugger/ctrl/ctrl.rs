use std::path::PathBuf;

use bytes::Bytes;

use primitive_types::H160;
use revm::{Database, EVMData, Gas, Inspector, Return, OPCODE_JUMPMAP};

use termwiz::lineedit::*;

use super::cli::CtrlCli;

#[derive(Debug)]
pub enum Ctrl {
    Exit,
    None,
    Step,
    StepIn,
    StepOut,
    Continue,
    Restart,
    Help,
    Breakpoint(H160, usize),
    AccountPrint(H160),
    AccountPrintOriginal(H160),
    Print(CtrlPrint)
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

#[derive(Debug)]
pub enum CtrlPrint {
    All,
    Stack,
    Opcode,
}

impl Ctrl {
    fn next(state: StateMachine, history_path: &Option<PathBuf>) -> Self {
        match state {
            StateMachine::TriggerBreakpoint | StateMachine::TriggerStep => {
                Self::parse_cli(history_path)
            }
            StateMachine::StepOut => {
                //if it is step_out we skip any command and want to return
                Ctrl::None
            }
            _ => Ctrl::None,
        }
    }

    pub fn parse_cli(history_path: &Option<PathBuf>) -> Ctrl {
        let mut terminal = line_editor_terminal().unwrap();
        let mut editor = LineEditor::new(&mut terminal);

        let mut host = CtrlCli::new(history_path.clone());
        let out = loop {
            if let Some(line) = editor.read_line(&mut host).unwrap() {
                if line == "exit" {
                    break Ctrl::Exit;
                }

                host.history().add(&line);
                if let Some(ctrl) = host.ctrl(&line) {
                    return ctrl;
                }
            }
        };
        out
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateMachine {
    Exit,
    TriggerStep,
    TriggerBreakpoint,
    StepOut,
}

pub struct Controller {
    state_machine: StateMachine,
    history_path: Option<PathBuf>,
    //call_stack: Vec<>,
}

impl Controller {
    pub fn new(history_path: Option<PathBuf>) -> Self {
        Self {
            state_machine: StateMachine::TriggerStep,
            history_path,
        }
    }
}
///
impl<DB: Database> Inspector<DB> for Controller {
    fn step(
        &mut self,
        machine: &mut revm::Machine,
        data: &mut EVMData<'_, DB>,
        is_static: bool,
    ) -> Return {
        loop {
            match Ctrl::next(self.state_machine, &self.history_path) {
                Ctrl::Help => {
                    
                }
                Ctrl::Exit => {
                    self.state_machine = StateMachine::Exit;
                    break;
                }
                Ctrl::Step => {
                    self.state_machine = StateMachine::TriggerStep;
                    break;
                }
                Ctrl::StepIn => {

                }
                Ctrl::StepOut => {
                    self.state_machine = StateMachine::StepOut;
                }

                Ctrl::Print(print) => {
                    match print {
                        CtrlPrint::All => {
                            println!("PRINT ALL");
                        },
                        CtrlPrint::Opcode => {
                            let opcode = *machine.contract.code.get(machine.program_counter()).unwrap();
                            println!("PC:{} OpCode: {:#x} {:?}",machine.program_counter, opcode,OPCODE_JUMPMAP[opcode as usize])
                        },
                        CtrlPrint::Stack => {
                            println!("PC:{} stack:{:?}",machine.program_counter,machine.stack())

                        },
                    }
                }
                Ctrl::Continue => {
                    self.state_machine = StateMachine::TriggerBreakpoint;
                    break;
                }
                Ctrl::Restart => {
                    //data.subroutine.checkpoint_revert(checkpoint)
                }
                Ctrl::Breakpoint(add, pc) => println!(
                    "Setting breakpoint for contract {} on program counter:{}",
                    add, pc
                ),
                Ctrl::AccountPrint(address) => {
                    println!("print:{:?}", data.subroutine.state().get(&address))
                }
                Ctrl::AccountPrintOriginal(address) => (),
                Ctrl::None => break,
            }
        }
        Return::Continue
    }

    fn step_end(&mut self, _eval: revm::Return, _machine: &mut revm::Machine) -> Return {
        Return::Continue
    }

    fn call(
        &mut self,
        _data: &mut revm::EVMData<'_, DB>,
        _call: primitive_types::H160,
        _context: &revm::CallContext,
        _transfer: &revm::Transfer,
        _input: &bytes::Bytes,
        _gas_limit: u64,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn call_end(&mut self) {
        match self.state_machine {
            StateMachine::StepOut => self.state_machine = StateMachine::TriggerStep,
            _ => (),
        }
    }

    fn create(
        &mut self,
        _data: &mut revm::EVMData<'_, DB>,
        _caller: primitive_types::H160,
        _scheme: &revm::CreateScheme,
        _value: primitive_types::U256,
        _init_code: &bytes::Bytes,
        _gas: u64,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        (Return::Continue, None, Gas::new(0), Bytes::new())
    }

    fn create_end(&mut self) {
        match self.state_machine {
            StateMachine::StepOut => self.state_machine = StateMachine::TriggerStep,
            _ => (),
        }
    }

    fn selfdestruct(&mut self) {}
}
