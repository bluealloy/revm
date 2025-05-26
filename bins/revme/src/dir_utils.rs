use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}
