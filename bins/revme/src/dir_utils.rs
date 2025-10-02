use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        // Log and skip entries that failed to be read instead of silently dropping errors
        .filter_map(|res| match res {
            Ok(e) => Some(e),
            Err(err) => {
                eprintln!("walkdir error at {}: {}", path.display(), err);
                None
            }
        })
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}
