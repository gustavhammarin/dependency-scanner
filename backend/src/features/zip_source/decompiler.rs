use std::io;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

use super::error::ZipSourceError;

/// Walk `source_dir` and decompile every .dll / .aar / .jar found.
/// Decompiled sources are written under `decompile_dir`.
pub fn process_directory(
    source_dir: &Path,
    decompile_dir: &Path,
    dotnet_decompiler: &str,
    java_decompiler: &str,
) -> Result<(), ZipSourceError> {
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "dll" => {
                tracing::info!("[decompiler] [.NET] {}", path.display());
                if let Err(e) = decompile_dotnet(path, decompile_dir, dotnet_decompiler) {
                    tracing::warn!("[decompiler] {e}");
                }
            }
            "aar" => {
                tracing::info!("[decompiler] [AAR] {}", path.display());
                if let Err(e) = process_aar(path, decompile_dir, java_decompiler) {
                    tracing::warn!("[decompiler] {e}");
                }
            }
            "jar" => {
                tracing::info!("[decompiler] [JAR] {}", path.display());
                let stem = file_stem(path);
                let out = decompile_dir.join(format!("java_{stem}"));
                if let Err(e) = decompile_java(path, &out, java_decompiler) {
                    tracing::warn!("[decompiler] {e}");
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn decompile_dotnet(
    dll_path: &Path,
    decompile_dir: &Path,
    decompiler: &str,
) -> Result<(), ZipSourceError> {
    let stem = file_stem(dll_path);
    let out = decompile_dir.join(format!("dotnet_{stem}"));
    std::fs::create_dir_all(&out)?;

    let status = Command::new("ilspycmd")
        .args(["-p", "-o", out.to_str().unwrap(), dll_path.to_str().unwrap()])
        .status();

    run_result(status, decompiler, &out)
}

fn process_aar(
    aar_path: &Path,
    decompile_dir: &Path,
    java_decompiler: &str,
) -> Result<(), ZipSourceError> {
    let stem = file_stem(aar_path);
    let aar_dir = decompile_dir.join(format!("aar_{stem}"));

    extract_zip(aar_path, &aar_dir)?;
    tracing::info!("[decompiler] Extracted AAR -> {}", aar_dir.display());

    for entry in WalkDir::new(&aar_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "jar" | "dex" => {
                let inner_stem = file_stem(path);
                tracing::info!(
                    "[decompiler]  [{}] {}",
                    ext.to_uppercase(),
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                let out = aar_dir.join(format!("src_{inner_stem}"));
                if let Err(e) = decompile_java(path, &out, java_decompiler) {
                    tracing::warn!("[decompiler]  {e}");
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn decompile_java(
    jar_path: &Path,
    output_dir: &Path,
    decompiler: &str,
) -> Result<(), ZipSourceError> {
    std::fs::create_dir_all(output_dir)?;

    let status = Command::new("jadx")
        .args(["-d", output_dir.to_str().unwrap(), jar_path.to_str().unwrap()])
        .status();

    run_result(status, decompiler, output_dir)
}

fn run_result(
    status: io::Result<std::process::ExitStatus>,
    decompiler: &str,
    out: &Path,
) -> Result<(), ZipSourceError> {
    match status {
        Ok(s) if s.success() => {
            tracing::info!("[decompiler]  -> {}", out.display());
            Ok(())
        }
        Ok(s) => {
            tracing::warn!("[decompiler] {} exited with {}", decompiler, s);
            Ok(()) // non-fatal: log and continue
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(ZipSourceError::DecompilerNotFound(
            format!("'{decompiler}' not found — make sure it is installed"),
        )),
        Err(e) => {
            tracing::warn!("[decompiler] Failed to run '{decompiler}': {e}");
            Ok(()) // non-fatal
        }
    }
}

fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<(), ZipSourceError> {
    std::fs::create_dir_all(output_dir)?;
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().replace('\\', "/");
        let entry_name = entry_name.trim_start_matches('/');
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

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}
