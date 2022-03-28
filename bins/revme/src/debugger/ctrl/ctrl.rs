use std::path::PathBuf;

use bytes::Bytes;

use primitive_types::{H160, U256};
use revm::{CallInputs, CreateInputs, Database, EVMData, Gas, Inspector, Return, OPCODE_JUMPMAP};

use termwiz::lineedit::*;

use super::cli::CtrlCli;

#[derive(Debug)]
pub enum Ctrl {
    Exit,
    None,
    Step,
    //StepIn,
    //StepOut,
    Continue,
    Restart,
    Help,
    //Breakpoint(H160, usize),
    AccountPrint(H160),
    AccountPrintOriginal(H160),
    Print(CtrlPrint),
    StackPop,
    StackPush(U256),
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
    // StorageSet
}

#[derive(Debug)]
pub enum CtrlPrint {
    All,
    Stack,
    Opcode,
    Memory,
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
    state_interp: StateMachine,
    history_path: Option<PathBuf>,
    //call_stack: Vec<>,
}

impl Controller {
    pub fn new(history_path: Option<PathBuf>) -> Self {
        Self {
            state_interp: StateMachine::TriggerStep,
            history_path,
        }
    }
}

///
impl<DB: Database> Inspector<DB> for Controller {
    fn step(
        &mut self,
        interp: &mut revm::Interpreter,
        data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        loop {
            match Ctrl::next(self.state_interp, &self.history_path) {
                Ctrl::Help => {
                    println!(
                        "available controls: \nstep\nexit\nprint all\nstack pop\nstack push 10\n"
                    )
                }
                Ctrl::Exit => {
                    self.state_interp = StateMachine::Exit;
                    break;
                }
                Ctrl::Step => {
                    self.state_interp = StateMachine::TriggerStep;
                    break;
                }
                //Ctrl::StepIn => {}
                //Ctrl::StepOut => {
                //    self.state_interp = StateMachine::StepOut;
                //}
                Ctrl::Print(print) => match print {
                    CtrlPrint::All => {
                        let opcode = interp
                            .contract
                            .code
                            .get(interp.program_counter())
                            .cloned()
                            .unwrap();
                        let gas_spend = interp.gas().spend();
                        let gas_remaining = interp.gas().remaining();
                        println!(
                            "call_depth:{} PC:{} Opcode: {:#x} {:?} gas(spend,remaining):({},{})\n\
                            Stack:{}",
                            data.subroutine.depth(),
                            interp.program_counter(),
                            opcode,
                            OPCODE_JUMPMAP[opcode as usize].unwrap_or("Invalid"),
                            gas_spend,
                            gas_remaining,
                            interp.stack(),
                        );
                    }
                    CtrlPrint::Opcode => {
                        let opcode = *interp.contract.code.get(interp.program_counter()).unwrap();
                        println!(
                            "PC:{} OpCode: {:#x} {:?}",
                            interp.program_counter(),
                            opcode,
                            OPCODE_JUMPMAP[opcode as usize]
                        )
                    }
                    CtrlPrint::Stack => {
                        println!("PC:{} stack:{}", interp.program_counter(), interp.stack())
                    }
                    CtrlPrint::Memory => {
                        println!("memory:{}", hex::encode(&interp.memory.data()))
                    }
                },
                Ctrl::Continue => {
                    self.state_interp = StateMachine::TriggerBreakpoint;
                    break;
                }
                Ctrl::Restart => {
                    //data.subroutine.checkpoint_revert(checkpoint)
                    println!("t:{:?}", StateMachine::StepOut);
                }
                //Ctrl::Breakpoint(add, pc) => println!(
                //    "Setting breakpoint for contract {} on program counter:{}",
                //    add, pc
                //),
                Ctrl::AccountPrint(address) => {
                    println!("print:{:?}", data.subroutine.state().get(&address))
                }
                Ctrl::AccountPrintOriginal(_address) => (),
                Ctrl::StackPop => {
                    println!("pop:{:?}", interp.stack.pop());
                }
                Ctrl::StackPush(value) => match interp.stack.push(value) {
                    Ok(()) => println!("stack:{}", interp.stack()),
                    Err(e) => println!("push error:{:?}", e),
                },
                Ctrl::None => break,
            }
        }
        Return::Continue
    }

    fn step_end(
        &mut self,
        _interp: &mut revm::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: revm::Return,
    ) -> Return {
        Return::Continue
    }

    fn call(
        &mut self,
        _data: &mut revm::EVMData<'_, DB>,
        _inputs: &mut CallInputs,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CallInputs,
        remaining_gas: Gas,
        ret: Return,
        out: Bytes,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        if let StateMachine::StepOut = self.state_interp {
            self.state_interp = StateMachine::TriggerStep
        }

        (ret, remaining_gas, out)
    }

    fn create(
        &mut self,
        _data: &mut revm::EVMData<'_, DB>,
        _inputs: &mut CreateInputs,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        (Return::Continue, None, Gas::new(0), Bytes::new())
    }

    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CreateInputs,
        ret: Return,
        address: Option<H160>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        if let StateMachine::StepOut = self.state_interp {
            self.state_interp = StateMachine::TriggerStep
        }

        (ret, address, remaining_gas, out)
    }

    fn selfdestruct(&mut self) {}
}
