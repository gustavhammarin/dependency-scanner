use std::{path::{Path, PathBuf}, time::SystemTime};

use tokio::{
    fs,
    time::{Duration, interval},
};

use crate::features::disk_cache::error::CacheError;

#[derive(Debug)]
pub struct DiskCache {
    root: PathBuf,
    cleanup_handle: Option<tokio::task::JoinHandle<()>>
}


impl DiskCache {
    pub async fn new(app_name: &str, max_age: Duration, cleanup_interval: Duration) -> Result<Self, CacheError> {
        let base = dirs::cache_dir().ok_or(CacheError::NoCacheDirFound)?;

        let root = base.join(&app_name);

        fs::create_dir_all(&root).await?;

        let cleanup_path = root.clone();
        let cleanup_handle = tokio::spawn(async move {
            cleanup_loop(cleanup_path, max_age, cleanup_interval).await;
        });

        Ok(Self { root, cleanup_handle : Some(cleanup_handle) })
    }

    pub async fn insert(&self, key: &str, source: &Path) -> Result<(), CacheError> {
        let dir = entry_dir(&self.root, key)?;

        if dir.exists() {
            fs::remove_dir_all(&dir).await?;
        }

        fs::create_dir_all(&dir).await?;

        if source.is_dir() {
            copy_recursive(source, &dir).await?;
        } else {
            let dest = dir
                .join(source.file_name().unwrap_or_default());
            fs::create_dir_all(dest.parent().unwrap()).await?;
            fs::copy(source, &dest).await?;
        };

        Ok(())
    }
    pub async fn get_path(&self, key: &str) -> Result<PathBuf, CacheError>{
        Ok(self.root.join(&key))
    }

    pub async fn delete_dir(&self, dir: &Path) -> Result<(), CacheError>{
        fs::remove_dir_all(dir).await?;
        Ok(())
    }
}

fn entry_dir(root: &Path, key: &str) -> Result<PathBuf, CacheError> {
    Ok(root.join(&key))
}

async fn copy_recursive(source: &Path, dest: &Path) -> Result<(), CacheError> {
     fs::create_dir_all(dest).await?;

    let mut entries = fs::read_dir(source).await?;

    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            Box::pin(copy_recursive(&src_path, &dst_path)).await?;
        } else {
            fs::copy(&src_path, &dst_path).await?;
        }
    }

    Ok(())
}

async fn cleanup_loop(cache_dir: PathBuf, max_age: Duration, cleanup_duration: Duration){
    let mut interval = interval(cleanup_duration);

    loop {
        interval.tick().await;

        if let Err(e) = cleanup_old_entries(&cache_dir, max_age).await {
            eprintln!("Cleanup error: {}", e)
        }
    }
}

async fn cleanup_old_entries(cache_dir: &Path, max_age: Duration) -> Result<(), CacheError>{
    let now = SystemTime::now();
    let mut entries = fs::read_dir(cache_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if let Ok(metadata) = fs::metadata(&path).await {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > max_age {
                        tracing::debug!("Removing old cache entry: {:?} (age: {:?})", path, age);

                        if metadata.is_dir(){
                            let _ = fs::remove_dir_all(&path).await;

                        }else {
                            let _ = fs::remove_file(&path).await;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

impl Drop for DiskCache{
    fn drop(&mut self) {
        if let Some(handle) = self.cleanup_handle.take(){
            handle.abort();
        }
    }
}
