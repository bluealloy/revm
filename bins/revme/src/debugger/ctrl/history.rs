use std::{borrow::Cow, collections::VecDeque, path::PathBuf};

use hashbrown::HashMap;
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, BufReader},
};
use termwiz::lineedit::{History, HistoryIndex, SearchDirection, SearchResult, SearchStyle};

pub struct CliHistory {
    entries: VecDeque<String>,
    file_name: Option<PathBuf>,
}

impl CliHistory {
    pub fn new(file_name: Option<PathBuf>) -> Self {
        if let Some(ref file) = file_name {
            let mut lines: HashMap<String, usize> = HashMap::new();

            let file = File::open(file).unwrap();
            let reader = BufReader::new(file);
            let mut num = 0;
            for line in reader.lines().map(|l| l.unwrap()) {
                if lines.try_insert(line, num).is_ok() {
                    num += 1;
                }
            }
            let mut entries = VecDeque::with_capacity(lines.len());
            entries.resize(lines.len(), String::new());
            for (key, val) in lines {
                *entries.get_mut(val).unwrap() = key;
            }
            Self { entries, file_name }
        } else {
            Self {
                entries: VecDeque::new(),
                file_name: None,
            }
        }
    }
}

impl History for CliHistory {
    fn get(&self, idx: HistoryIndex) -> Option<Cow<str>> {
        self.entries.get(idx).map(|s| Cow::Borrowed(s.as_str()))
    }

    fn last(&self) -> Option<HistoryIndex> {
        if self.entries.is_empty() {
            None
        } else {
            Some(self.entries.len() - 1)
        }
    }

    fn add(&mut self, line: &str) {
        if self.entries.back().map(String::as_str) == Some(line) {
            return;
        }
        self.entries.push_back(line.to_owned());
        if let Some(ref file_name) = self.file_name {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(file_name)
                .unwrap();

            if let Err(e) = writeln!(file, "{}", line) {
                eprintln!("Couldn't write to history file: {}", e);
            }
        }
    }

    fn search(
        &self,
        idx: HistoryIndex,
        style: SearchStyle,
        direction: SearchDirection,
        pattern: &str,
    ) -> Option<SearchResult> {
        let mut idx = idx;

        loop {
            let line = match self.entries.get(idx) {
                Some(line) => line,
                None => return None,
            };

            if let Some(cursor) = style.match_against(pattern, line) {
                return Some(SearchResult {
                    line: Cow::Borrowed(line.as_str()),
                    idx,
                    cursor,
                });
            }

            idx = match direction.next(idx) {
                None => return None,
                Some(idx) => idx,
            };
        }
    }
}
