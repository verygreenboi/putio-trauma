pub mod putio_client;
pub mod sync;

pub use putio_client::{File, PutIoClient};
pub use sync::SyncEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_types() {
        let file = File {
            id: 1,
            name: "test".to_string(),
            file_type: "FOLDER".to_string(),
            parent_id: 0,
            size: None,
            content_type: None,
            created_at: None,
            is_shared: None,
            screenshot: None,
        };

        assert!(file.is_folder());
    }

    #[test]
    fn test_client_creation() {
        let client = PutIoClient::new("test_token".to_string());
        assert_eq!(client.get_download_url(123), "https://api.put.io/v2/files/123/download?oauth_token=test_token");
    }
}