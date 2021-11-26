use std::io::stdin;

use bytes::Bytes;
use hashbrown::HashMap;
use primitive_types::H160;
use revm::{Gas, Inspector, Return};

use termwiz::cell::AttributeChange;
use termwiz::color::{AnsiColor, ColorAttribute, RgbColor};
use termwiz::lineedit::*;
use termwiz::Result;

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
    fn next(state: StateMachine) -> Self {
        match state {
            StateMachine::TriggerBreakpoint | StateMachine::TriggerStep => {
                Self::parse_inside_trigger()
            }
        }
    }

    pub fn parse_inside_trigger() -> Ctrl {
        let mut terminal = line_editor_terminal().unwrap();
        let mut editor = LineEditor::new(&mut terminal);

        let mut host = CtrlCli::new();
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

struct CtrlCli {
    history: BasicHistory,
    ctrl_tree: HashMap<usize, Vec<(usize, &'static str)>>,
}

impl CtrlCli {
    pub fn new() -> Self {
        let mut ctree = HashMap::new();

        ctree.insert(
            0,
            vec![
                (1, "exit"),
                (2, "step"),
                (3, "continue"),
                (4, "breakpoint"),
                (5, "account"),
            ],
        );
        //account subcommands
        ctree.insert(
            5,
            vec![
                (50, "load"),
                (51, "read"),
                (52, "balance"),
                (53, "0x00000000000000000000"),
            ],
        );
        ctree.insert(50, vec![(501, "0x00000000000000000000")]);

        Self {
            history: BasicHistory::default(),
            ctrl_tree: ctree,
        }
    }
}

impl LineEditorHost for CtrlCli {
    // Render the prompt with a darkslateblue background color if
    // the terminal supports true color, otherwise render it with
    // a navy blue ansi color.
    fn render_prompt(&self, prompt: &str) -> Vec<OutputElement> {
        vec![
            OutputElement::Attribute(AttributeChange::Background(
                ColorAttribute::TrueColorWithPaletteFallback(
                    RgbColor::from_named("darkslateblue").unwrap(),
                    AnsiColor::Navy.into(),
                ),
            )),
            OutputElement::Text(prompt.to_owned()),
        ]
    }

    fn history(&mut self) -> &mut dyn History {
        &mut self.history
    }

    /// Demo of the completion API for words starting with "h" or "he"
    fn complete(&self, line: &str, cursor_position: usize) -> Vec<CompletionCandidate> {
        let mut candidates = vec![];
        if let Some((range, word)) = word_at_cursor(line, cursor_position) {
            //let words = &["hello", "help", "he-man"];
            let mut id = 0;
            let mut has_word = false;
            for word in line[..range.start].split_ascii_whitespace() {
                if let Some(ctree) = self.ctrl_tree.get(&id) {
                    for (ctree_id, ctree_word) in ctree.iter() {
                        if word == *ctree_word {
                            has_word = true;
                            id = *ctree_id;
                            break;
                        }
                    }

                    if !has_word {
                        break;
                    }
                } else {
                    break;
                }
            }
            if let Some(words) = self.ctrl_tree.get(&id) {
                for (_, w) in words {
                    if w.starts_with(word) {
                        candidates.push(CompletionCandidate {
                            range: range.clone(),
                            text: w.to_string(),
                        });
                    }
                }
            }
        }
        candidates
    }
}

/// This is a conceptually simple function that computes the bounds
/// of the whitespace delimited word at the specified cursor position
/// in the supplied line string.
/// It returns the range and the corresponding slice out of the line.
/// This function is sufficient for example purposes; in a real application
/// the equivalent function would need to be aware of quoting and other
/// application specific context.
fn word_at_cursor<'a>(
    line: &'a str,
    cursor_position: usize,
) -> Option<(std::ops::Range<usize>, &'a str)> {
    let char_indices: Vec<(usize, char)> = line.char_indices().collect();
    if char_indices.is_empty() {
        return None;
    }
    let char_position = char_indices
        .iter()
        .position(|(idx, _)| *idx == cursor_position)
        .unwrap_or(char_indices.len());

    // Look back until we find whitespace
    let mut start_position = char_position;
    while start_position > 0
        && start_position <= char_indices.len()
        && !char_indices[start_position - 1].1.is_whitespace()
    {
        start_position -= 1;
    }

    // Look forwards until we find whitespace
    let mut end_position = char_position;
    while end_position < char_indices.len() && !char_indices[end_position].1.is_whitespace() {
        end_position += 1;
    }

    if end_position > start_position {
        let range = char_indices[start_position].0
            ..char_indices
                .get(end_position)
                .map(|c| c.0 + 1)
                .unwrap_or(line.len());
        Some((range.clone(), &line[range]))
    } else {
        None
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
        match Ctrl::next(self.machine) {
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
