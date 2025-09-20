# Put.io Folder Sync

[![Build](https://github.com/USER/putio-trauma/actions/workflows/build.yml/badge.svg)](https://github.com/USER/putio-trauma/actions/workflows/build.yml)

A Rust application that syncs folders from put.io to your local filesystem using depth-first search (DFS) traversal. Downloads up to 3 files concurrently while preserving the folder structure.

## Features

- **DFS Traversal**: Traverses folders depth-first, prioritizing deepest files
- **Folder Structure Preservation**: Creates local directories matching put.io structure
- **Concurrent Downloads**: Downloads up to 3 files simultaneously using the trauma library
- **Smart Skip**: Skips files that already exist locally with matching file sizes
- **OAuth Authentication**: Uses put.io OAuth token for secure API access

## Setup

1. **Get your put.io OAuth token**:
   - Go to https://app.put.io/settings/account
   - Copy your OAuth token

2. **Set the environment variable**:
   ```bash
   export PUT_IO_TOKEN=your_oauth_token_here
   ```

3. **Build the application**:
   ```bash
   cargo build --release
   ```

## Usage

```bash
# Sync a specific folder by path
./target/release/putio-trauma /Movies ./local/movies

# Sync a folder by ID
./target/release/putio-trauma 1529952071 ./local/folder

# Sync the root folder
./target/release/putio-trauma / ./local/putio

# Sync using "root" alias
./target/release/putio-trauma root ./local/putio
```

### Arguments

- `<remote_folder_path>`: Path to the folder in your put.io account (e.g., `/Movies`, `/`, `root`) or folder ID (e.g., `1529952071`)
- `<local_destination>`: Local directory where files will be downloaded

## How it Works

1. **Authentication**: Connects to put.io API using your OAuth token
2. **Folder Discovery**: Finds the specified remote folder by path
3. **DFS Traversal**: Recursively traverses all subfolders using depth-first search
4. **File Queue**: Builds a download queue prioritizing files in deeper folders
5. **Concurrent Download**: Downloads files 3 at a time using the trauma library
6. **Structure Preservation**: Creates local folder structure matching the remote layout

## Error Handling

- Validates PUT_IO_TOKEN environment variable
- Handles missing remote folders gracefully
- Skips already downloaded files (size comparison)
- Provides clear error messages for API failures

## Dependencies

- `tokio`: Async runtime
- `reqwest`: HTTP client for API calls
- `serde`: JSON serialization/deserialization
- `trauma`: Concurrent file downloader

## CI/CD and Releases

This project uses GitHub Actions for continuous integration and cross-platform builds:

- **Automated Testing**: Runs tests, formatting checks, and linting on every push/PR
- **Cross-Platform Builds**: Builds binaries for Linux x64, macOS x64, and macOS ARM64
- **Release Automation**: Automatically creates release assets when a new version is tagged

### Supported Platforms

- **Linux**: `x86_64-unknown-linux-gnu`
- **macOS Intel**: `x86_64-apple-darwin`
- **macOS Apple Silicon**: `aarch64-apple-darwin`

### Release Downloads

Pre-built binaries are available from the [Releases](https://github.com/USER/putio-trauma/releases) page.

### Workflow Features

The GitHub Actions workflow (`.github/workflows/build.yml`) includes:

- **Code Quality**: `cargo fmt --check` and `cargo clippy`
- **Testing**: Library unit tests with `cargo test --lib`
- **Cross-compilation**: Uses Rust toolchain targets for different platforms
- **Caching**: Speeds up builds by caching Cargo dependencies
- **Artifact Upload**: Uploads build artifacts for each platform
- **Release Automation**: Attaches binaries to GitHub releases automatically

## Example

```bash
export PUT_IO_TOKEN=abc123xyz789
./target/release/putio-trauma /Movies/Comedy ./downloads/comedy

# Output:
# Put.io Folder Sync
# Remote path: /Movies/Comedy
# Local destination: ./downloads/comedy
#
# Scanning folder: ./downloads/comedy (depth: 0)
# Scanning folder: ./downloads/comedy/New Releases (depth: 1)
# Found 15 files to download
# Downloading 15 files (3 concurrent)...
# Downloading: movie1.mp4 -> ./downloads/comedy/movie1.mp4
# Downloading: movie2.mp4 -> ./downloads/comedy/New Releases/movie2.mp4
# Download complete!
# Sync completed successfully!
```