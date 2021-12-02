use std::path::PathBuf;

use hashbrown::HashMap;

use primitive_types::H160;
use termwiz::cell::AttributeChange;
use termwiz::color::{AnsiColor, ColorAttribute, RgbColor};
use termwiz::lineedit::*;

use super::ctrl::Ctrl;
use super::history::CliHistory;

pub struct CtrlCli {
    history: CliHistory,
}

const N: usize = usize::MAX;

const CTREE: &'static [&'static [(usize, &'static str)]] = &[
    &[
        (N, "exit"),
        (N, "step"),
        (N, "continue"),
        (1, "account"),
        (2, "breakpoint"),
    ],
    &[(N, "original"), (N, "0x00000000000000000000")],
    &[(N, "0x00000000000000000000")],
];

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
                if w1 == exit {
                    Some(Ctrl::Exit)
                } else if w1 == step {
                    Some(Ctrl::Step)
                } else if w1 == continu {
                    Some(Ctrl::Continue)
                } else {
                    None
                }
            }
            2 => {
                let w1 = words[0];
                let w2 = words[1];
                let account = CTREE[0][3].1;
                if w1 == account {
                    if let Some(address) = parse_address(w2, "Account address not valid") {
                        return Some(Ctrl::AccountPrint(address));
                    }
                }
                None
            }
            3 => {
                let w1 = words[0];
                let w2 = words[1];
                let w3 = words[3];
                //account
                let account = CTREE[0][3].1;
                //let breakpoint = CTREE[0][4].1;

                if w1 == account {
                    let original = CTREE[1][0].1;
                    let address = parse_address(w3, "Account address not valid")?;
                    if w2 == original {
                        return Some(Ctrl::AccountPrintOriginal(address));
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
