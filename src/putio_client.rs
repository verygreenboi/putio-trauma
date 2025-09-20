use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct FileListResponse {
    pub files: Vec<File>,
    pub parent: Option<File>,
    pub total: i64,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct File {
    pub id: i64,
    pub name: String,
    pub file_type: String,
    pub parent_id: i64,
    pub size: Option<i64>,
    pub content_type: Option<String>,
    pub created_at: Option<String>,
    pub is_shared: Option<bool>,
    pub screenshot: Option<String>,
}

impl File {
    pub fn is_folder(&self) -> bool {
        self.file_type == "FOLDER"
    }

    #[allow(dead_code)]
    pub fn is_file(&self) -> bool {
        !self.is_folder()
    }
}

pub struct PutIoClient {
    client: reqwest::Client,
    token: String,
    base_url: String,
}

impl PutIoClient {
    pub fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
            base_url: "https://api.put.io/v2".to_string(),
        }
    }

    pub async fn list_files(&self, parent_id: i64) -> Result<FileListResponse, Box<dyn Error>> {
        let url = format!("{}/files/list", self.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("oauth_token", &self.token),
                ("parent_id", &parent_id.to_string()),
                ("per_page", &"1000".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("API request failed: {}", response.status()).into());
        }

        let data: FileListResponse = response.json().await?;
        Ok(data)
    }

    pub async fn get_file_info(&self, file_id: i64) -> Result<File, Box<dyn Error>> {
        let url = format!("{}/files/{}", self.base_url, file_id);

        let response = self
            .client
            .get(&url)
            .query(&[("oauth_token", &self.token)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("API request failed: {}", response.status()).into());
        }

        #[derive(Deserialize)]
        struct FileResponse {
            file: File,
        }

        let data: FileResponse = response.json().await?;
        Ok(data.file)
    }

    pub fn get_download_url(&self, file_id: i64) -> String {
        format!(
            "{}/files/{}/download?oauth_token={}",
            self.base_url, file_id, self.token
        )
    }

    pub async fn find_folder_by_path(&self, path: &str) -> Result<Option<File>, Box<dyn Error>> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            return Ok(Some(File {
                id: 0,
                name: "Root".to_string(),
                file_type: "FOLDER".to_string(),
                parent_id: -1,
                size: None,
                content_type: None,
                created_at: None,
                is_shared: None,
                screenshot: None,
            }));
        }

        let mut current_parent_id = 0i64;
        let mut current_folder = None;

        for part in parts {
            let files = self.list_files(current_parent_id).await?;

            let folder = files.files.iter().find(|f| f.is_folder() && f.name == part);

            match folder {
                Some(f) => {
                    current_parent_id = f.id;
                    current_folder = Some(f.clone());
                }
                None => return Ok(None),
            }
        }

        Ok(current_folder)
    }
}
