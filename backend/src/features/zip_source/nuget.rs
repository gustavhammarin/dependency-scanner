use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tokio::process::Command;

use crate::features::zip_source::file_extractors::extract_zip;

use super::{decompiler, error::ZipSourceError};

const FRAMEWORKS: &[&str] = &[
    "net8.0",
    "net8.0-android",
    "net6.0",
    "netstandard2.1",
    "netstandard2.0",
];

pub struct NuGetSource {
    client: reqwest::Client,
}

impl NuGetSource {
    pub fn new() -> Result<Self, ZipSourceError> {
        let client = reqwest::Client::builder()
            .user_agent("3pp-analyzer/1.0")
            .build()?;
        Ok(Self { client })
    }

    /// Download, extract, decompile DLLs/AARs/JARs, write synthetic `.csproj` files,
    /// run `dotnet restore` + `cdxgen` to produce `cdx.json`, and return the root dir.
    /// The root contains:
    ///   - `{id}.{ver}.nupkg`       — original package file
    ///   - `extracted/`             — raw nupkg contents
    ///   - `decompiled/`            — decompiled source (for crypto analysis)
    ///   - `project_{tfm}/`         — synthetic .csproj per framework
    ///   - `cdx.json`               — CycloneDX SBOM (first successful TFM)
    /// Caller must keep TempDir alive.
    pub async fn download_and_extract(
        &self,
        package_id: &str,
        version: &str,
    ) -> Result<(TempDir, PathBuf), ZipSourceError> {
        let id = package_id.trim().to_lowercase();
        let ver = version.trim().to_lowercase();
        let url = format!("https://api.nuget.org/v3-flatcontainer/{id}/{ver}/{id}.{ver}.nupkg");

        tracing::info!("[nuget-source] Downloading {id} {ver}");

        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(ZipSourceError::PackageNotFound(format!(
                "NuGet returned HTTP {}: package '{package_id}' version '{version}' not found",
                resp.status()
            )));
        }
        let bytes = resp.bytes().await?;

        let temp = tempfile::tempdir()?;
        let nupkg_path = temp.path().join(format!("{id}.{ver}.nupkg"));
        tokio::fs::write(&nupkg_path, &bytes).await?;

        let extract_dir = temp.path().join("extracted");
        tokio::fs::create_dir_all(&extract_dir).await?;

        let np = nupkg_path;
        let ed = extract_dir.clone();
        tokio::task::spawn_blocking(move || extract_zip(&np, &ed))
            .await
            .map_err(|e| ZipSourceError::TokioTaskError(e.to_string()))??;

        let decompiled_dir = temp.path().join("decompiled");
        tokio::fs::create_dir_all(&decompiled_dir).await?;

        let dst = decompiled_dir.clone();
        tokio::task::spawn_blocking(move || {
            decompiler::process_directory(&extract_dir, &dst, "ilspycmd", "jadx")
        })
        .await
        .unwrap()?;

        tracing::info!("[nuget-source] Decompiled {id} {ver}");

        for tfm in FRAMEWORKS {
            let project_dir = temp.path().join(format!("project_{tfm}"));
            tokio::fs::create_dir_all(&project_dir).await?;
            tokio::fs::write(
                project_dir.join("project.csproj"),
                csproj_content(&id, &ver, tfm),
            )
            .await?;
        }

        tracing::info!("[nuget-source] Created csproj stubs for {id} {ver}");

        let root = temp.path().to_path_buf();
        let cdx_path = root.join("cdx.json");

        for tfm in FRAMEWORKS {
            let project_dir = root.join(format!("project_{tfm}"));

            match run_dotnet_restore(&project_dir, &root, tfm).await? {
                RestoreOutcome::Skip => continue,
                RestoreOutcome::Ok => {}
            }

            run_cdxgen(&project_dir, &cdx_path).await?;
            tracing::info!("[nuget-source] cdx.json generated with TFM {tfm}");
            break;
        }

        if !cdx_path.exists() {
            tracing::warn!(
                "[nuget-source] No compatible TFM for {id} {ver} — cdx.json not generated"
            );
        }

        Ok((temp, root))
    }
}

enum RestoreOutcome {
    Ok,
    Skip,
}

async fn run_dotnet_restore(
    project_dir: &Path,
    nupkg_dir: &Path,
    tfm: &str,
) -> Result<RestoreOutcome, ZipSourceError> {
    let out = Command::new("dotnet")
        .args([
            "restore",
            project_dir.to_str().unwrap(),
            "--source",
            nupkg_dir.to_str().unwrap(),
            "--source",
            "https://api.nuget.org/v3/index.json",
        ])
        .output()
        .await?;

    if out.status.success() {
        tracing::info!("[nuget-source] dotnet restore succeeded with TFM {tfm}");
        return Ok(RestoreOutcome::Ok);
    }

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");

    // NETSDK1202 = EOL framework warning promoted to error, but if
    // "Restored" appears the restore actually completed — treat as success.
    if combined.contains("NETSDK1202") && stdout.contains("Restored ") {
        tracing::info!(
            "[nuget-source] dotnet restore completed with TFM {tfm} (NETSDK1202 EOL warning ignored)"
        );
        return Ok(RestoreOutcome::Ok);
    }

    // NETSDK1147 = workload not installed, NU1202/NU1213 = no compatible TFM — try next
    if combined.contains("NETSDK1147")
        || combined.contains("NU1202")
        || combined.contains("NU1213")
        || combined.contains("compatible")
    {
        tracing::info!("[nuget-source] TFM {tfm} not available, trying next");
        return Ok(RestoreOutcome::Skip);
    }

    Err(ZipSourceError::CommandFailed(format!(
        "dotnet restore failed (exit {}):\nstdout: {}\nstderr: {}",
        out.status.code().unwrap_or(-1),
        stdout.trim(),
        stderr.trim(),
    )))
}

async fn run_cdxgen(dir: &Path, output: &Path) -> Result<(), ZipSourceError> {
    tracing::info!("[nuget-source] Running cdxgen on {}", dir.display());

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
    tracing::info!("[nuget-source] cdxgen exited with status {status_code}");

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

fn csproj_content(package_id: &str, version: &str, tfm: &str) -> String {
    format!(
        r#"<Project Sdk="Microsoft.NET.Sdk">
        <PropertyGroup>
            <TargetFramework>{tfm}</TargetFramework>
            <Nullable>enable</Nullable>
        </PropertyGroup>
        <ItemGroup>
            <PackageReference Include="{package_id}" Version="{version}" />
        </ItemGroup>
        </Project>
        "#
    )
}
