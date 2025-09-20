use putio_trauma::{PutIoClient, SyncEngine};
use std::env;
use std::path::PathBuf;
use std::process;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!(
            "Usage: {} <remote_folder_path> <local_destination>",
            args[0]
        );
        eprintln!("Example: {} /Movies ./downloads", args[0]);
        eprintln!("Set PUT_IO_TOKEN environment variable with your put.io OAuth token");
        process::exit(1);
    }

    let remote_folder_path = &args[1];
    let local_destination = PathBuf::from(&args[2]);

    let token = match env::var("PUT_IO_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("Error: PUT_IO_TOKEN environment variable not set");
            eprintln!("Get your token from: https://app.put.io/settings/account");
            process::exit(1);
        }
    };

    if token.trim().is_empty() {
        eprintln!("Error: PUT_IO_TOKEN is empty");
        process::exit(1);
    }

    let client = PutIoClient::new(token);
    let mut sync_engine = SyncEngine::new(client, local_destination.clone());

    println!("Put.io Folder Sync");
    println!("Remote path: {remote_folder_path}");
    println!("Local destination: {}", local_destination.display());
    println!();

    let folder_to_sync = if remote_folder_path == "/" || remote_folder_path == "root" {
        (0, None)
    } else if let Ok(folder_id) = remote_folder_path.parse::<i64>() {
        // If it's a numeric ID, use it directly
        match sync_engine.client.get_file_info(folder_id).await {
            Ok(folder) => {
                if folder.is_folder() {
                    (folder.id, Some(folder.name))
                } else {
                    eprintln!("Error: ID {folder_id} is not a folder");
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error getting folder info for ID {folder_id}: {e}");
                process::exit(1);
            }
        }
    } else {
        // Try to find by path
        match sync_engine
            .client
            .find_folder_by_path(remote_folder_path)
            .await
        {
            Ok(Some(folder)) => (folder.id, Some(folder.name)),
            Ok(None) => {
                eprintln!("Error: Folder '{remote_folder_path}' not found in your put.io account");
                process::exit(1);
            }
            Err(e) => {
                eprintln!("Error finding folder: {e}");
                process::exit(1);
            }
        }
    };

    if let Err(e) = sync_engine
        .sync_folder(folder_to_sync.0, folder_to_sync.1)
        .await
    {
        eprintln!("Sync failed: {e}");
        process::exit(1);
    }

    println!("Sync completed successfully!");
}
