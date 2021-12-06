use std::path::PathBuf;
use std::str::FromStr;

use primitive_types::{H160, U256};
use termwiz::cell::AttributeChange;
use termwiz::color::{AnsiColor, ColorAttribute, RgbColor};
use termwiz::lineedit::*;

use super::ctrl::{Ctrl, CtrlPrint};
use super::history::CliHistory;

pub struct CtrlCli {
    history: CliHistory,
}

const N: usize = usize::MAX;

const CTREE: &'static [&'static [(usize, &'static str)]] = &[
    &[
        (N, "exit"),       //0
        (N, "step"),       //1
        (N, "continue"),   //2
        (1, "account"),    //3
        (2, "breakpoint"), //4
        (N, "restart"),    //5
        (N, "help"),       //6
        (3, "print"),      //7
        (4, "stack"),      //8
    ],
    /*1*/ &[(N, "original"), (N, "0x00000000000000000000")],
    /*2*/ &[(N, "0x00000000000000000000")],
    /*3*/ &[(N, "all"), (N, "stack"), (N, "opcode"), (N, "memory")],
    /*4*/ &[(N, "pop"), (2, "push")],
];

pub fn parse_u256(str: &str) -> Option<U256> {
    if str.len() > 2 && &str[..2] == "0x" {
        U256::from_str(&str[2..]).ok()
    } else {
        U256::from_dec_str(str).ok()
    }
}

pub fn parse_address(addr: &str, info: &str) -> Option<H160> {
    if addr.len() < 40 {
        println!("Error: :{} not big enough. Expect 20bytes hex ", &info);
        return None;
    }
    let addr = if &addr[0..2] == "0x" {
        &addr[2..]
    } else {
        addr
    };
    let addr = match hex::decode(addr) {
        Ok(hex) => H160::from_slice(&hex),
        Err(_) => {
            println!("Error: {} not in hex format", &info);

            return None;
        }
    };
    Some(addr)
}

impl CtrlCli {
    pub fn ctrl(&self, line: &str) -> Option<Ctrl> {
        let words: Vec<&str> = line.split_whitespace().collect();
        let len = words.len();
        match len {
            0 => None,
            1 => {
                let w1 = words[0];
                let exit = CTREE[0][0].1;
                let step = CTREE[0][1].1;
                let continu = CTREE[0][2].1;
                let restart = CTREE[0][5].1;
                let help = CTREE[0][6].1;
                if w1 == exit {
                    Some(Ctrl::Exit)
                } else if w1 == step {
                    Some(Ctrl::Step)
                } else if w1 == continu {
                    Some(Ctrl::Continue)
                } else if w1 == restart {
                    Some(Ctrl::Restart)
                } else if w1 == help {
                    Some(Ctrl::Help)
                } else {
                    None
                }
            }
            2 => {
                let w1 = words[0];
                let w2 = words[1];
                let (_, account) = CTREE[0][3];
                let (printn, print) = CTREE[0][7];
                let stack = CTREE[0][8];
                if w1 == account {
                    if let Some(address) = parse_address(w2, "Account address not valid") {
                        return Some(Ctrl::AccountPrint(address));
                    }
                    None
                } else if w1 == print {
                    let all = CTREE[printn][0].1;
                    let stack = CTREE[printn][1].1;
                    let opcode = CTREE[printn][2].1;
                    let memory = CTREE[printn][3].1;
                    if w2 == all {
                        return Some(Ctrl::Print(CtrlPrint::All));
                    } else if w2 == stack {
                        return Some(Ctrl::Print(CtrlPrint::Stack));
                    } else if w2 == opcode {
                        return Some(Ctrl::Print(CtrlPrint::Opcode));
                    } else if w2 == memory {
                        return Some(Ctrl::Print(CtrlPrint::Memory));
                    }
                    None
                } else if w1 == stack.1 {
                    let pop = CTREE[stack.0][0].1;
                    if w2 == pop {
                        return Some(Ctrl::StackPop);
                    }
                    None
                } else {
                    None
                }
            }
            3 => {
                let w1 = words[0];
                let w2 = words[1];
                let w3 = words[2];
                //account
                let account = CTREE[0][3].1;
                let stack = CTREE[0][8];

                if w1 == account {
                    let original = CTREE[1][0].1;
                    let address = parse_address(w3, "Account address not valid")?;
                    if w2 == original {
                        return Some(Ctrl::AccountPrintOriginal(address));
                    }
                } else if w1 == stack.1 {
                    let pop = CTREE[stack.0][1].1;
                    if w2 == pop {
                        if let Some(stack_item) = parse_u256(w3) {
                            return Some(Ctrl::StackPush(stack_item));
                        } else {
                            println!("stack values not valid");
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
    pub fn new(history_file: Option<PathBuf>) -> Self {
        Self {
            history: CliHistory::new(history_file),
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
                if let Some(ctree) = CTREE.get(id) {
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
            if let Some(&words) = CTREE.get(id) {
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
