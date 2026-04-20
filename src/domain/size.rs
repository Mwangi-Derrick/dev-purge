use crate::types::Finding;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn estimate_sizes(candidates: &[PathBuf]) -> Vec<Finding> {
    candidates
        .par_iter()
        .map(|path| Finding {
            path: path.clone(),
            bytes: estimate_dir_size_bytes(path),
        })
        .filter(|f| f.bytes > 0)
        .collect()
}

fn estimate_dir_size_bytes(dir: &Path) -> u64 {
    let mut total: u64 = 0;

    let it = WalkDir::new(dir).follow_links(false).into_iter();
    for entry in it {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.file_type().is_file() {
            if let Ok(md) = entry.metadata() {
                total = total.saturating_add(md.len());
            }
        }
    }

    total
}
