use crate::putio_client::{File, PutIoClient};
use std::collections::HashSet;
use std::error::Error;
use std::path::{Path, PathBuf};
use tokio::fs;
use trauma::{download::Download, downloader::DownloaderBuilder};

pub struct DownloadItem {
    pub file: File,
    pub local_path: PathBuf,
    pub depth: usize,
}

pub struct SyncEngine {
    pub client: PutIoClient,
    base_local_path: PathBuf,
    visited_folders: HashSet<i64>,
}

impl SyncEngine {
    pub fn new(client: PutIoClient, base_local_path: PathBuf) -> Self {
        Self {
            client,
            base_local_path,
            visited_folders: HashSet::new(),
        }
    }

    pub async fn sync_folder(
        &mut self,
        folder_id: i64,
        folder_name: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        let mut download_queue = Vec::new();

        self.dfs_traverse(
            folder_id,
            &self.base_local_path.clone(),
            0,
            &mut download_queue,
            folder_name,
        )
        .await?;

        download_queue.sort_by(|a, b| {
            b.depth
                .cmp(&a.depth)
                .then_with(|| a.file.id.cmp(&b.file.id))
        });

        if download_queue.is_empty() {
            println!("No files to download");
            return Ok(());
        }

        println!("Found {} files to download", download_queue.len());
        println!("Starting downloads (3 concurrent)...");

        self.download_files(download_queue).await?;

        Ok(())
    }

    async fn dfs_traverse(
        &mut self,
        root_folder_id: i64,
        root_local_path: &Path,
        root_depth: usize,
        download_queue: &mut Vec<DownloadItem>,
        root_folder_name: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        #[derive(Clone)]
        struct FolderTask {
            folder_id: i64,
            local_path: PathBuf,
            depth: usize,
            folder_name: Option<String>,
        }

        let mut folder_stack = vec![FolderTask {
            folder_id: root_folder_id,
            local_path: root_local_path.to_path_buf(),
            depth: root_depth,
            folder_name: root_folder_name,
        }];

        while let Some(task) = folder_stack.pop() {
            if self.visited_folders.contains(&task.folder_id) {
                continue;
            }
            self.visited_folders.insert(task.folder_id);

            let current_local_path = if let Some(name) = &task.folder_name {
                task.local_path.join(name)
            } else {
                task.local_path.clone()
            };

            fs::create_dir_all(&current_local_path).await?;

            println!(
                "Scanning folder: {} (depth: {})",
                current_local_path.display(),
                task.depth
            );

            let files_response = self.client.list_files(task.folder_id).await?;

            let mut folders = Vec::new();
            let mut files = Vec::new();

            for file in files_response.files {
                if file.is_folder() {
                    folders.push(file);
                } else {
                    files.push(file);
                }
            }

            for file in files {
                let file_local_path = current_local_path.join(&file.name);
                download_queue.push(DownloadItem {
                    file,
                    local_path: file_local_path,
                    depth: task.depth,
                });
            }

            for folder in folders {
                folder_stack.push(FolderTask {
                    folder_id: folder.id,
                    local_path: current_local_path.clone(),
                    depth: task.depth + 1,
                    folder_name: Some(folder.name),
                });
            }
        }

        Ok(())
    }

    async fn download_files(&self, mut items: Vec<DownloadItem>) -> Result<(), Box<dyn Error>> {
        items.retain(|item| {
            if item.local_path.exists() {
                if let Ok(metadata) = std::fs::metadata(&item.local_path) {
                    if let Some(remote_size) = item.file.size {
                        if metadata.len() == remote_size as u64 {
                            println!("Skipping existing file: {}", item.local_path.display());
                            return false;
                        }
                    }
                }
            }
            true
        });

        if items.is_empty() {
            println!("All files are already up to date");
            return Ok(());
        }

        println!("Downloading {} files (3 concurrent)...", items.len());

        let mut downloads = Vec::new();
        for item in &items {
            let url = self.client.get_download_url(item.file.id);
            let mut download = Download::try_from(url.as_str())?;

            if let Some(filename) = item.local_path.file_name().and_then(|n| n.to_str()) {
                download.filename = filename.to_string();
            }

            let parent_dir = item.local_path.parent().unwrap_or(&self.base_local_path);
            std::fs::create_dir_all(parent_dir)?;

            downloads.push(download);
        }

        let temp_dir = std::env::temp_dir().join("putio-downloads");
        std::fs::create_dir_all(&temp_dir)?;

        let downloader = DownloaderBuilder::new()
            .concurrent_downloads(3)
            .directory(temp_dir.clone())
            .build();

        downloader.download(&downloads).await;

        for (download, item) in downloads.iter().zip(items.iter()) {
            let filename = &download.filename;
            let temp_file = temp_dir.join(filename);
            if temp_file.exists() {
                println!(
                    "Moving {} to {}",
                    temp_file.display(),
                    item.local_path.display()
                );
                std::fs::rename(&temp_file, &item.local_path)?;
            }
        }

        println!("Download complete!");
        Ok(())
    }
}
