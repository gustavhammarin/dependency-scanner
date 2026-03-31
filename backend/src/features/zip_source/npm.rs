use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::process::Command;

use crate::features::zip_source::{ZipSourceError, file_extractors::extract_tar_gz};

pub struct NpmSource {
    client: reqwest::Client,
}

impl NpmSource {
    pub fn new() -> Result<Self, ZipSourceError> {
        let client = reqwest::Client::builder()
            .user_agent("3pp-analyzer/1.0")
            .build()?;
        Ok(Self { client })
    }

    pub async fn download_and_extract(
        &self,
        package_id: &str,
        version: &str,
    ) -> Result<(TempDir, PathBuf), ZipSourceError> {
        let (id, ver, bytes) = self.fetch(package_id, version).await?;
        let (temp, extract_dir) = Self::write_and_extract(&id, &ver, &bytes).await?;
        let source_dir = extract_dir.join("package");
        run_cdxgen(&source_dir, &source_dir.join("cdx.json")).await?;
        Ok((temp, source_dir))
    }

    pub async fn fetch(
        &self,
        package_id: &str,
        version: &str,
    ) -> Result<(String, String, bytes::Bytes), ZipSourceError> {
        let id = package_id.trim().to_lowercase();
        let ver = version.trim().to_lowercase();
        let url = format!("https://registry.npmjs.org/{id}/-/{id}-{ver}.tgz");

        tracing::info!("[npm-source] Downloading {id} {ver}");

        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(ZipSourceError::PackageNotFound(format!(
                "NPM returned HTTP {}: package '{package_id}' version '{version}' not found",
                resp.status()
            )));
        }

        let bytes = resp.bytes().await?;
        Ok((id, ver, bytes))
    }

    pub async fn write_and_extract(
        id: &str,
        ver: &str,
        bytes: &bytes::Bytes,
    ) -> Result<(TempDir, PathBuf), ZipSourceError> {
        let temp = tempfile::tempdir()?;
        let tgz_path = temp.path().join(format!("{id}.{ver}.tgz"));
        tokio::fs::write(&tgz_path, bytes).await?;

        let extract_dir = temp.path().join("extracted");
        tokio::fs::create_dir_all(&extract_dir).await?;

        let tp = tgz_path.clone();
        let ed = extract_dir.clone();

        tokio::task::spawn_blocking(move || Self::extract_file(&tp, &ed))
            .await
            .map_err(|e| ZipSourceError::TokioTaskError(e.to_string()))??;

        Ok((temp, extract_dir))
    }

    pub fn extract_file(tgz_path: &Path, output_path: &Path) -> Result<(), ZipSourceError> {
        extract_tar_gz(tgz_path, output_path)?;
        Ok(())
    }
}

async fn run_cdxgen(dir: &Path, output: &Path) -> Result<(), ZipSourceError> {
    tracing::info!("[npm-source] Running cdxgen on {}", dir.display());

    let out = Command::new("cdxgen")
        .args([
            "--output",
            output.to_str().unwrap(),
            "--spec-version",
            "1.4",
            dir.to_str().unwrap(),
        ])
        .output()
        .await?;

    let status_code = out.status.code().unwrap_or(-1);
    tracing::info!("[npm-source] cdxgen exited with status {status_code}");

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(ZipSourceError::CommandFailed(format!(
            "cdxgen failed (exit {status_code}): {stderr}"
        )));
    }

    if !output.exists() {
        return Err(ZipSourceError::CommandFailed(
            "cdxgen did not produce an output file".to_string(),
        ));
    }

    Ok(())
}
