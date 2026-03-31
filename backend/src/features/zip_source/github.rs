use std::{
    fs,
    path::{Path, PathBuf},
};

use tempfile::TempDir;
use tokio::process::Command;

use crate::features::zip_source::file_extractors::extract_zip;

use super::error::ZipSourceError;

pub struct GitHubSource {
    client: reqwest::Client,
}

impl GitHubSource {
    pub fn new() -> Result<Self, ZipSourceError> {
        let client = reqwest::Client::builder()
            .user_agent("3pp_analyzer")
            .build()?;
        Ok(Self { client })
    }

    pub async fn download_and_extract(
        &self,
        package_id: &str,
        version: &str,
    ) -> Result<(TempDir, PathBuf), ZipSourceError> {
        let (_, bytes) = self.fetch(package_id, version).await?;
        let (temp, extract_dir) = Self::write_and_extract(&package_id, version, &bytes).await?;
        let source_dir = resolve_repo_path(&extract_dir)?;
        generate_cdx(&source_dir, version).await?;
        Ok((temp, source_dir))
    }

    pub async fn fetch(
        &self,
        package_id: &str,
        version: &str,
    ) -> Result<(String, bytes::Bytes), ZipSourceError> {
        let version = version.trim();
        let mut resp = self.fetch_zip(package_id, version).await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND && !version.is_empty() {
            let alt = if version.starts_with('v') {
                version.trim_start_matches('v').to_string()
            } else {
                format!("v{version}")
            };
            tracing::debug!("[github-source] 404 — retrying with tag {alt:?}");
            resp = self.fetch_zip(package_id, &alt).await?;
        }

        let bytes = resp.error_for_status()?.bytes().await?;
        tracing::info!("[github-source] Download complete: {} bytes", bytes.len());
        Ok((package_id.to_string(), bytes))
    }

    pub async fn write_and_extract(
        package_id: &str,
        version: &str,
        bytes: &bytes::Bytes,
    ) -> Result<(TempDir, PathBuf), ZipSourceError> {
        let temp = tempfile::tempdir()?;
        let safe_name = package_id.replace('/', "_");
        let zip_path = temp.path().join(format!("{safe_name}-{version}.zip"));
        tokio::fs::write(&zip_path, bytes).await?;

        let extract_dir = temp.path().join("extracted");
        tokio::fs::create_dir_all(&extract_dir).await?;

        let zp = zip_path.clone();
        let ed = extract_dir.clone();
        tokio::task::spawn_blocking(move || Self::extract_file(&zp, &ed))
            .await
            .map_err(|e| ZipSourceError::TokioTaskError(e.to_string()))??;

        Ok((temp, extract_dir))
    }

    pub fn extract_file(zip_path: &Path, output_dir: &Path) -> Result<(), ZipSourceError> {
        extract_zip(zip_path, output_dir)?;
        Ok(())
    }

    async fn fetch_zip(
        &self,
        package_id: &str,
        version: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = if version.is_empty() {
            format!("https://api.github.com/repos/{package_id}/zipball")
        } else {
            format!("https://api.github.com/repos/{package_id}/zipball/{version}")
        };
        tracing::info!("[github-source] Downloading {package_id} @ {version:?}");
        self.client.get(url).send().await
    }
}

fn resolve_repo_path(extract_dir: &Path) -> Result<PathBuf, ZipSourceError> {
    fs::read_dir(extract_dir)?
        .filter_map(Result::ok)
        .find(|e| e.path().is_dir())
        .map(|e| e.path())
        .ok_or(ZipSourceError::RepoRootNotFound)
}

/// Generates `cdx.json` in `source_dir`. Uses the Python pipeline (venv + pip + cdxgen)
/// if Python markers are found, otherwise falls back to generic cdxgen.
async fn generate_cdx(source_dir: &Path, version: &str) -> Result<(), ZipSourceError> {
    let cdx_path = source_dir.join("cdx.json");

    let is_python = ["setup.py", "pyproject.toml"]
        .iter()
        .any(|f| source_dir.join(f).exists());

    if is_python {
        generate_python_cdx(source_dir, &cdx_path, version).await
    } else {
        run_cdxgen(source_dir, &cdx_path).await
    }
}

async fn generate_python_cdx(
    source_dir: &Path,
    output: &Path,
    version: &str,
) -> Result<(), ZipSourceError> {
    tracing::info!("[github-source] Python package detected, generating cdx.json via venv");

    let temp = tempfile::tempdir()?;
    let venv_dir = temp.path().join("py-env");

    let out = Command::new("python3")
        .args(["-m", "venv", venv_dir.to_str().unwrap()])
        .output()
        .await?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(ZipSourceError::CommandFailed(format!(
            "python3 -m venv failed: {stderr}"
        )));
    }

    let pip = venv_dir.join("bin/pip");
    let out = Command::new(&pip)
        .args(["install", source_dir.to_str().unwrap()])
        // GitHub zips lack .git so setuptools-scm can't detect the version.
        // Providing the real version here prevents the build from failing.
        .env("SETUPTOOLS_SCM_PRETEND_VERSION", version)
        .output()
        .await?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(ZipSourceError::CommandFailed(format!(
            "pip install failed: {stderr}"
        )));
    }

    let out = Command::new("cdxgen")
        .args([
            "-t",
            "python",
            venv_dir.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .output()
        .await?;

    let status_code = out.status.code().unwrap_or(-1);
    tracing::info!("[github-source] cdxgen exited with status {status_code}");

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(ZipSourceError::CommandFailed(format!(
            "cdxgen failed (exit {status_code}): {stderr}"
        )));
    }

    if !output.exists() {
        return Err(ZipSourceError::CommandFailed(
            "cdxgen did not produce cdx.json".to_string(),
        ));
    }

    tracing::info!("[github-source] cdx.json written to {}", output.display());
    Ok(())
}

async fn run_cdxgen(dir: &Path, output: &Path) -> Result<(), ZipSourceError> {
    tracing::info!("[github-source] Running cdxgen on {}", dir.display());

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
    tracing::info!("[github-source] cdxgen exited with status {status_code}");

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
