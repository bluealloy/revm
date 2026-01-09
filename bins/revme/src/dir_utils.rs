use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

/// Find all JSON test files in the given path.
/// If path is a file, returns it in a vector.
/// If path is a directory, recursively finds all .json files.
pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension() == Some("json".as_ref()))
            .map(DirEntry::into_path)
            .collect()
    }
}
