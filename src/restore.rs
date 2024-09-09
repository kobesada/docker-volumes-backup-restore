use crate::backup::run_backup;
use crate::utility::compression::decompress_file_from_tar;
use crate::utility::docker::{start_containers, stop_containers};
use crate::utility::server::{download_from_server, get_latest_backup_file_name_from_server};
use fs_extra::dir::CopyOptions;
use fs_extra::{move_items, remove_items};
use std::error::Error;
use std::fs;
use std::path::Path;

/// Restores specified Docker volumes from a backup file on a remote server.
///
/// This function performs the following steps:
/// 1. Determines which backup file to restore, either the latest or a specified one.
/// 2. Downloads the backup file from the remote server.
/// 3. Extracts the specified volumes from the backup file.
/// 4. Performs a backup before the restoration process.
/// 5. Replaces the existing volume data with the extracted data.
/// 6. Cleans up temporary files and directories.
///
/// # Arguments
///
/// * `server_ip` - A string slice representing the IP address of the remote server.
/// * `server_port` - A string slice representing the SSH port on the remote server.
/// * `server_user` - A string slice representing the username for SSH authentication.
/// * `server_directory` - A string slice representing the directory on the server where backup files are stored.
/// * `backup_to_be_restored` - A string slice representing the backup file to restore, or "latest" for the most recent backup.
/// * `volumes_to_be_restored` - A string slice representing the volumes to restore, comma-separated, or "all" to restore all volumes.
/// * `ssh_key_path` - A string slice representing the path to the SSH private key used for authentication.
/// * `temp_path` - A string slice representing the path to a temporary directory for storing the backup during restoration.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An empty result if the restoration is successful, or an error if something goes wrong.
pub fn restore_volumes(server_ip: &str,
                       server_port: &str,
                       server_user: &str,
                       server_directory: &str,
                       backup_to_be_restored: &str,
                       volumes_to_be_restored: &str,
                       ssh_key_path: &str,
                       temp_path: &str) -> Result<(), Box<dyn Error>> {
    // Determine the backup file to restore (either specified or the latest)
    let backup_file_name = if backup_to_be_restored == "latest" {
        get_latest_backup_file_name_from_server(server_ip, server_port, server_user, server_directory, ssh_key_path)?
    } else { backup_to_be_restored.to_string() };

    // Define paths for the local and remote backup files
    let local_backup_path = format!("{}/{}", temp_path, backup_file_name);
    let remote_backup_path = format!("{}/{}", server_directory, backup_file_name);

    // Download the backup file from the remote server
    download_from_server(server_ip, server_port, server_user, &remote_backup_path, &local_backup_path, ssh_key_path)?;

    // Define the temporary path for extracted volumes
    let volumes_temp_path = format!("{}/volumes", temp_path);

    // Extract the specified volumes from the backup file
    let volume_names = extract_volumes_from_backup(&local_backup_path, volumes_to_be_restored, &volumes_temp_path)?;

    // Perform a backup before restoration
    run_backup(server_ip, server_port, server_user, server_directory, ssh_key_path, temp_path)?;

    // Restore each volume by decompressing and replacing existing data
    for volume in &volume_names {
        let volume_backup_path = format!("{}/{}.tar.gz", volumes_temp_path, volume);
        let volume_extract_path = format!("{}/{}", volumes_temp_path, volume);
        decompress_file_from_tar(&volume_backup_path, &volume_extract_path)?;
        replace_volume_data_with_dir(&volume_extract_path, volume)?;
    }

    // Clean up temporary files
    fs::remove_dir_all(temp_path)?;

    println!("Restoration completed successfully. The {:?} volumes were restored from {}", volume_names, backup_file_name);
    Ok(())
}

/// Extracts specific volumes from a backup file.
///
/// This function decompresses a backup file to a temporary directory and returns the names
/// of the volumes to be restored. If "all" is specified, all volumes in the backup file
/// are returned.
///
/// # Arguments
///
/// * `local_backup_path` - A string slice representing the path to the local backup file.
/// * `volumes_to_be_restored` - A string slice representing the volumes to restore, comma-separated, or "all" to restore all volumes.
/// * `temp_path` - A string slice representing the path to a temporary directory for extracting the volumes.
///
/// # Returns
///
/// * `Result<Vec<String>, Box<dyn Error>>` - A vector of volume names to be restored, or an error if something goes wrong.
fn extract_volumes_from_backup(local_backup_path: &str,
                               volumes_to_be_restored: &str,
                               temp_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    // Decompress the entire tar.gz archive to the temporary directory
    decompress_file_from_tar(local_backup_path, temp_path)?;

    // Return the names of all volumes or the specified ones
    if volumes_to_be_restored == "all" {
        get_names_of_all_volumes(temp_path)
    } else {
        Ok(volumes_to_be_restored.split(',').map(|s| s.trim().to_string()).collect())
    }
}

/// Retrieves the names of all volumes from a directory.
///
/// This function scans a directory and returns the names of all files that have a `.tar.gz`
/// extension, representing the volumes.
///
/// # Arguments
///
/// * `dir_path` - A string slice representing the path to the directory containing the volumes.
///
/// # Returns
///
/// * `Result<Vec<String>, Box<dyn Error>>` - A vector of volume names (without the `.tar.gz` extension), or an error if something goes wrong.
fn get_names_of_all_volumes(dir_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let volumes: Vec<String> = fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".tar.gz"))
        .map(|name| name.trim_end_matches(".tar.gz").to_string())
        .collect();

    Ok(volumes)
}

/// Replaces the data in a Docker volume with the contents of a specified directory.
///
/// This function performs the following steps:
/// 1. Stops containers using the specified volume.
/// 2. Removes the existing data in the volume.
/// 3. Moves new data from the extracted directory to the volume's mount point.
/// 4. Restarts the containers.
///
/// # Arguments
///
/// * `dir_path` - A string slice representing the path to the directory containing the new volume data.
/// * `volume_name` - A string slice representing the name of the Docker volume to be replaced.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An empty result if the replacement is successful, or an error if something goes wrong.
pub fn replace_volume_data_with_dir(dir_path: &str, volume_name: &str) -> Result<(), Box<dyn Error>> {
    // Stop containers using the specified volume
    let container_ids = stop_containers(volume_name)?;

    // Define the path where the volume is mounted inside the container
    let container_path = format!("/backup/{}", volume_name);

    // Check if the volume mount point exists
    if !Path::new(&container_path).exists() {
        return Err(format!("Volume {} does not exist.", volume_name).into());
    }

    // Remove existing data in the volume's mount point
    let volume_data = collect_paths(&container_path)?;
    remove_items(&volume_data)?;

    // Move the new data from the extracted directory to the volume's mount point
    let dir_data = collect_paths(&dir_path)?;
    let options = CopyOptions::new();
    move_items(&dir_data, &container_path, &options)?;

    // Restart the containers that were stopped
    start_containers(container_ids)?;
    Ok(())
}

/// Collects the paths of all files and directories within a given directory.
///
/// This function reads the contents of a directory and returns a vector of paths as strings.
///
/// # Arguments
///
/// * `dir` - A string slice representing the directory to scan.
///
/// # Returns
///
/// * `Result<Vec<String>, Box<dyn Error>>` - A vector of paths within the directory, or an error if something goes wrong.
fn collect_paths(dir: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(fs::read_dir(dir)?
        .map(|entry| entry.unwrap().path().to_str().unwrap().to_string())
        .collect())
}
