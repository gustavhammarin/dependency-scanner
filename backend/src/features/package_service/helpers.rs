use std::path::Path;

use walkdir::WalkDir;

pub async fn calculate_bytes(source_dir: &Path) -> u64 {
    WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().unwrap().len())
        .sum::<u64>()
}
