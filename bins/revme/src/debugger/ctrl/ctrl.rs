use std::path::PathBuf;

use bytes::Bytes;

use primitive_types::H160;
use revm::{Database, EVMData, Gas, Inspector, Return};

use termwiz::lineedit::*;

use super::cli::CtrlCli;

#[derive(Debug)]
pub enum Ctrl {
    Exit,
    None,
    Step,
    Continue,
    Breakpoint(H160, usize),
    AccountPrint(H160),
    AccountPrintOriginal(H160),
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

impl Ctrl {
    fn next(state: StateMachine, history_path: &Option<PathBuf>) -> Self {
        println!("NEXT LINE");
        match state {
            StateMachine::TriggerBreakpoint | StateMachine::TriggerStep => {
                Self::parse_cli(history_path)
            }
            _ => Ctrl::None,
        }
    }

    pub fn parse_cli(history_path: &Option<PathBuf>) -> Ctrl {
        let mut terminal = line_editor_terminal().unwrap();
        let mut editor = LineEditor::new(&mut terminal);

        let mut host = CtrlCli::new(history_path.clone());
        println!("PARSE_CLI");
        let out = loop {
            if let Some(line) = editor.read_line(&mut host).unwrap() {
                println!("PARSE_CLI2");
                if line == "exit" {
                    break Ctrl::Exit;
                }

                host.history().add(&line);
                if let Some(ctrl) = host.ctrl(&line) {
                    return ctrl;
                }
            }
        };
        println!("PARSE_CLI00");
        out
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateMachine {
    Exit,
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

impl<DB: Database> Inspector<DB> for Controller {
    fn step(&mut self, _machine: &mut revm::Machine, data: &mut EVMData<'_,DB>, is_static: bool) {
        println!("STEP INSIDE");
        loop {
            match Ctrl::next(self.machine, &self.history_path) {
                Ctrl::Exit => {
                    self.machine = StateMachine::Exit;
                    break;
                }
                Ctrl::Step => {
                    self.machine = StateMachine::TriggerStep;
                    break;
                }
                Ctrl::Continue => {
                    self.machine = StateMachine::TriggerBreakpoint;
                    break;
                }
                Ctrl::Breakpoint(add, pc) => println!(
                    "Setting breakpoint for contract {} on program counter:{}",
                    add, pc
                ),
                Ctrl::AccountPrint(address) => {
                    println!("print:{:?}", data.subroutine.state().get(&address))
                }
                Ctrl::AccountPrintOriginal(address) => (),
                Ctrl::None => (),
            }
        }
    }

    fn step_end(&mut self, _eval: revm::Return, _machine: &mut revm::Machine) {}

    fn call(
        &mut self,
        _data: &mut revm::EVMData<'_,DB>,
        _call: primitive_types::H160,
        _context: &revm::CallContext,
        _transfer: &revm::Transfer,
        _input: &bytes::Bytes,
        _gas_limit: u64,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn create(
        &mut self,
        _data: &mut revm::EVMData<'_,DB>,
        _caller: primitive_types::H160,
        _scheme: &revm::CreateScheme,
        _value: primitive_types::U256,
        _init_code: &bytes::Bytes,
        _gas: u64,
    )-> (Return,Option<H160>, Gas, Bytes) {
        (Return::Continue,None, Gas::new(0), Bytes::new())
    }

    fn selfdestruct(&mut self) {}
}
