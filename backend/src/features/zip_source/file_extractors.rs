use std::{fs::File, path::{Component, Path}};

use flate2::read::GzDecoder;
use tar::Archive;

use crate::features::zip_source::ZipSourceError;

const MAX_UNCOMPRESSED_SIZE: u64 = 500 * 1024 * 1024; // 500 MB
const MAX_FILE_COUNT: usize = 50_000;

const SKIP_DIRS: &[&str] = &[".git", ".svn"];

fn should_skip(path: &str) -> bool {
    path.split('/').any(|component| SKIP_DIRS.contains(&component))
}

pub fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<(), ZipSourceError> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut total_size: u64 = 0;
    let mut file_count: usize = 0;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        file_count += 1;
        if file_count > MAX_FILE_COUNT {
            return Err(ZipSourceError::Validation("Too many files".to_string()));
        }

        let entry_name = entry.name().replace('\\', "/");
        let entry_name = entry_name.trim_start_matches('/');

        if entry_name.split('/').any(|c| c == "..") {
            return Err(ZipSourceError::Validation("Path traversal".to_string()));
        }

        if should_skip(entry_name) {
            continue;
        }

        total_size += entry.size();
        if total_size > MAX_UNCOMPRESSED_SIZE {
            return Err(ZipSourceError::Validation("File too big".to_string()));
        }

        let outpath = output_dir.join(entry_name);

        if entry.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(())
}

pub fn extract_tar_gz(tgz_path: &Path, output_path: &Path) -> Result<(), ZipSourceError> {
    let file = File::open(tgz_path)?;

    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    archive.set_preserve_permissions(false);
    archive.set_overwrite(true);

    let mut total_size: u64 = 0;
    let mut file_count: usize = 0;

    for entry in archive.entries()? {
        let mut entry = entry?;

        file_count += 1;
        if file_count > MAX_FILE_COUNT {
            return Err(ZipSourceError::Validation("Too many files".to_string()));
        }

        let entry_path = entry.path()?;
        let mut skip = false;
        for component in entry_path.components() {
            if matches!(component, Component::ParentDir) {
                return Err(ZipSourceError::Validation("Path traversal".to_string()));
            }
            if let Component::Normal(s) = component {
                if SKIP_DIRS.contains(&s.to_string_lossy().as_ref()) {
                    skip = true;
                    break;
                }
            }
        }
        if skip {
            continue;
        }

        total_size += entry.header().size()?;
        if total_size > MAX_UNCOMPRESSED_SIZE {
            return Err(ZipSourceError::Validation("File too big".to_string()));
        }

        entry.unpack_in(output_path)?;
    }

    Ok(())
}
