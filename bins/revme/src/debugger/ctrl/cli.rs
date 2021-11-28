
use std::path::PathBuf;


use hashbrown::HashMap;



use termwiz::cell::AttributeChange;
use termwiz::color::{AnsiColor, ColorAttribute, RgbColor};
use termwiz::lineedit::*;

use super::history::CliHistory;

pub struct CtrlCli {
    history: CliHistory,
    ctrl_tree: HashMap<usize, Vec<(usize, &'static str)>>,
}

impl CtrlCli {
    pub fn new(history_file: Option<PathBuf>) -> Self {
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
            history: CliHistory::new(history_file),
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
