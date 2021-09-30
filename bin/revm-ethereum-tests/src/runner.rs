#![feature(slice_as_chunks)]

use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize},
        Arc,
    },
};

use indicatif::ProgressBar;
use std::sync::atomic::Ordering;
use tokio::{join, sync::Semaphore};
use walkdir::{DirEntry, WalkDir};

use crate::models::TestSuit;

pub async fn find_all_json_tests(path: PathBuf) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}

pub async fn execute_test_suit(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let json_reader = std::fs::read(&path).unwrap();
    let suit: TestSuit = serde_json::from_reader(&*json_reader)?;
    for (_,unit) in suit.0.into_iter() {
       // unit.

    }
    Ok(())
}

pub async fn run(test_files: Vec<PathBuf>) {
    let semaphore = Arc::new(Semaphore::new(10)); //execute 10 at the time
    let endjob = Arc::new(AtomicBool::new(false));
    let console_bar = Arc::new(ProgressBar::new(test_files.len() as u64));
    let mut joins = Vec::new();
    for chunk in test_files.chunks(20) {
        let chunk = Vec::from(chunk);
        let endjob = endjob.clone();
        let semaphore = semaphore.clone();
        let console_bar = console_bar.clone();
        joins.push(tokio::spawn(async move {
            for test in chunk {
                let _ = semaphore.acquire().await;
                if endjob.load(Ordering::SeqCst) {
                    return;
                }
                if let Err(err) = execute_test_suit(&test).await {
                    endjob.store(true, Ordering::SeqCst);
                    console_bar.finish();
                    println!("{:?} failed: {}", test, err);
                    return;
                }
                console_bar.inc(1);
            }
        }));
    }
    for join in joins {
        let _ = join.await;
    }
    // if not error finish console bar
    if !endjob.load(Ordering::SeqCst) {
        console_bar.finish();
    }
}
