use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::features::zip_source::{
    ZipSourceError,
    file_extractors::{extract_tar_gz, extract_zip},
};

pub enum FileType {
    Wheel,
    TarGz,
}

impl FileType {
    pub fn file_type(file_path: &Path) -> FileType {
        if file_path.extension().is_some_and(|e| e == "whl") {
            FileType::Wheel
        } else {
            FileType::TarGz
        }
    }

    pub fn file_extension(&self) -> String {
        match self {
            FileType::Wheel => "whl".to_string(),
            FileType::TarGz => "tar.gz".to_string(),
        }
    }
}

pub struct PyPISource {
    pub client: reqwest::Client,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PyPiMetadata {
    pub urls: Vec<PyPiUrl>,
}

impl PyPiMetadata {
    pub fn best_source(&self) -> Option<(&str, FileType)> {
        if let Some(url) = self.wheel_url() {
            return Some((url, FileType::Wheel));
        }
        if let Some(url) = self.sdist_url() {
            return Some((url, FileType::TarGz));
        }

        None
    }

    pub fn sdist_url(&self) -> Option<&str> {
        self.urls
            .iter()
            .find(|u| u.packagetype == "sdist")
            .map(|u| u.url.as_str())
    }

    pub fn wheel_url(&self) -> Option<&str> {
        self.urls
            .iter()
            .find(|u| u.packagetype == "bdist_wheel" && u.filename.contains("py3-none-any"))
            .or_else(|| {
                // Annars ta första bdist_wheel
                self.urls.iter().find(|u| u.packagetype == "bdist_wheel")
            })
            .map(|u| u.url.as_str())
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PyPiUrl {
    pub packagetype: String,
    pub url: String,
    pub filename: String,
}

impl PyPISource {
    pub fn new() -> Result<Self, super::ZipSourceError>
    where
        Self: Sized,
    {
        let client = reqwest::Client::builder()
            .user_agent("3pp_analyzer/1.0")
            .build()?;

        Ok(Self { client })
    }

    pub async fn download_and_extract(
        &self,
        package_id: &str,
        version: &str,
    ) -> Result<(tempfile::TempDir, std::path::PathBuf), super::ZipSourceError> {
        let (id, ver, file_extension, bytes) = self.fetch(package_id, version).await?;
        let (temp, extract_dir) =
            Self::write_and_extract(&id, &ver, &file_extension, &bytes).await?;
        let source_dir = extract_dir.join(format!("{id}-{ver}"));
        Ok((temp, source_dir))
    }

    pub(crate) async fn fetch(
        &self,
        package_id: &str,
        version: &str,
    ) -> Result<(String, String, String, bytes::Bytes), super::ZipSourceError> {
        tracing::info!("fetching");
        let id = package_id.trim().to_lowercase();
        let ver = version.trim().to_lowercase();
        let metadata = self
            .client
            .get(format!("https://pypi.org/pypi/{id}/{ver}/json"))
            .send()
            .await?
            .json::<PyPiMetadata>()
            .await?;

        let (url, file_type) = match metadata.best_source() {
            Some(u) => u,
            None => return Err(ZipSourceError::SourceUrlNotFound),
        };

        let bytes = self.client.get(url).send().await?.bytes().await?;

        tracing::info!("fetched");

        Ok((id, ver, file_type.file_extension(), bytes))
    }

    async fn write_and_extract(
        id: &str,
        ver: &str,
        file_extension: &str,
        bytes: &bytes::Bytes,
    ) -> Result<(tempfile::TempDir, std::path::PathBuf), super::ZipSourceError> {
        let temp = tempfile::tempdir()?;
        let file_path = temp.path().join(format!("{id}.{ver}.{file_extension}"));
        tokio::fs::write(&file_path, bytes).await?;

        let extract_dir = temp.path().join("extracted");
        tokio::fs::create_dir_all(&extract_dir).await?;

        let tp = file_path.clone();
        let ed = extract_dir.clone();

        tokio::task::spawn_blocking(move || Self::extract_file(&tp, &ed))
            .await
            .map_err(|e| ZipSourceError::TokioTaskError(e.to_string()))??;

        Ok((temp, extract_dir))
    }

    fn extract_file(
        file_path: &std::path::Path,
        output_path: &std::path::Path,
    ) -> Result<(), super::ZipSourceError> {
        match FileType::file_type(file_path) {
            FileType::Wheel => extract_zip(file_path, output_path),
            FileType::TarGz => extract_tar_gz(file_path, output_path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Path::ends_with kollar path-komponenter, inte stringsuffix – använd extension() istället.
    #[test]
    fn file_type_detects_whl_extension() {
        assert!(matches!(
            FileType::file_type(std::path::Path::new("torch.2.2.0.whl")),
            FileType::Wheel
        ));
        assert!(matches!(
            FileType::file_type(std::path::Path::new("torch.2.2.0.tar.gz")),
            FileType::TarGz
        ));
    }

    /// Wheel väljs före sdist – torch sdist är ~800MB och gör att fetch hänger.
    #[test]
    fn best_source_prefers_wheel_over_sdist() {
        let metadata = PyPiMetadata {
            urls: vec![
                PyPiUrl {
                    packagetype: "bdist_wheel".to_string(),
                    url: "https://example.com/torch-2.2.0-cp311-cp311-linux_x86_64.whl".to_string(),
                    filename: "torch-2.2.0-cp311-cp311-linux_x86_64.whl".to_string(),
                },
                PyPiUrl {
                    packagetype: "sdist".to_string(),
                    url: "https://example.com/torch-2.2.0.tar.gz".to_string(),
                    filename: "torch-2.2.0.tar.gz".to_string(),
                },
            ],
        };

        let (url, file_type) = metadata.best_source().expect("should find a source");
        assert_eq!(url, "https://example.com/torch-2.2.0-cp311-cp311-linux_x86_64.whl");
        assert!(matches!(file_type, FileType::Wheel));
    }

    /// py3-none-any väljs framför plattformsspecifik wheel när båda finns.
    #[test]
    fn best_source_prefers_pure_python_wheel() {
        let metadata = PyPiMetadata {
            urls: vec![
                PyPiUrl {
                    packagetype: "bdist_wheel".to_string(),
                    url: "https://example.com/requests-2.31.0-cp311-cp311-linux_x86_64.whl".to_string(),
                    filename: "requests-2.31.0-cp311-cp311-linux_x86_64.whl".to_string(),
                },
                PyPiUrl {
                    packagetype: "bdist_wheel".to_string(),
                    url: "https://example.com/requests-2.31.0-py3-none-any.whl".to_string(),
                    filename: "requests-2.31.0-py3-none-any.whl".to_string(),
                },
            ],
        };

        let (url, _) = metadata.best_source().expect("should find a source");
        assert_eq!(url, "https://example.com/requests-2.31.0-py3-none-any.whl");
    }

    /// Faller tillbaka på sdist om inga wheels finns.
    #[test]
    fn best_source_falls_back_to_sdist() {
        let metadata = PyPiMetadata {
            urls: vec![PyPiUrl {
                packagetype: "sdist".to_string(),
                url: "https://example.com/somelib-1.0.tar.gz".to_string(),
                filename: "somelib-1.0.tar.gz".to_string(),
            }],
        };

        let (url, file_type) = metadata.best_source().expect("should find a source");
        assert_eq!(url, "https://example.com/somelib-1.0.tar.gz");
        assert!(matches!(file_type, FileType::TarGz));
    }

    /// Hämtar metadata från PyPI för torch 2.2.0 och loggar vilken URL som väljs,
    /// utan att ladda ner själva paketet. Bekräftar att en source-URL hittas.
    #[tokio::test]
    #[ignore = "integration: kräver nätverksåtkomst"]
    async fn torch_metadata_fetch_picks_sdist() {
        let source = PyPISource::new().unwrap();
        let metadata = source
            .client
            .get("https://pypi.org/pypi/torch/2.2.0/json")
            .send()
            .await
            .unwrap()
            .json::<PyPiMetadata>()
            .await
            .unwrap();

        let (url, file_type) = metadata.best_source().expect("borde hitta en source");
        println!("Vald URL: {url}");
        println!("Filtyp: {}", file_type.file_extension());

        // Dokumenterar att sdist väljs och förklarar varför fetch hänger
        if metadata.sdist_url().is_some() {
            println!(
                "VARNING: sdist finns – torch sdist är enormt och orsakar timeout vid nedladdning"
            );
        }
    }

    /// Reproduktionstest: kör hela fetch-pipelinen för torch 2.2.0 med timeout.
    /// Förväntas timeouta (=hänger) vilket bekräftar bugg-beteendet.
    #[tokio::test]
    #[ignore = "integration: laddar ner från internet, förväntas timeouta för torch"]
    async fn torch_fetch_times_out() {
        let source = PyPISource::new().unwrap();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            source.fetch("torch", "2.2.0"),
        )
        .await;

        match result {
            Err(_elapsed) => {
                println!("Timeout efter 30s – bekräftar att fetch hänger för torch 2.2.0");
                // Detta är det förväntade bugbeteendet
            }
            Ok(Ok((id, ver, ext, bytes))) => {
                println!("Fetch lyckades: {id} {ver} .{ext} ({} bytes)", bytes.len());
                // Om detta inträffar fungerar pipelinen – kanske med en snabb mirror
            }
            Ok(Err(e)) => panic!("Fetch misslyckades med fel: {e}"),
        }
    }
}
