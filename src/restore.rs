use crate::utility::compression::decompress_file_from_tar;
use crate::utility::docker::{start_containers, stop_containers};
use crate::utility::server::{download_from_server, get_latest_backup_file_name_from_server};
use fs_extra::dir::CopyOptions;
use fs_extra::{move_items, remove_items};
use std::error::Error;
use std::fs::{self};
use std::path::Path;

pub fn restore_volumes(server_ip: &str,
                       server_port: &str,
                       server_user: &str,
                       server_directory: &str,
                       backup_to_be_restored: &str,
                       volumes_to_be_restored: &str,
                       ssh_key_path: &str,
                       temp_path: &str) -> Result<(), Box<dyn Error>>
{
    let backup_file_name = if backup_to_be_restored == "latest" {
        get_latest_backup_file_name_from_server(server_ip, server_port, server_user, server_directory, ssh_key_path)?
    } else { backup_to_be_restored.to_string() };

    let local_backup_path = format!("{}/{}", temp_path, backup_file_name);
    let remote_backup_path = format!("{}/{}", server_directory, backup_file_name);

    download_from_server(server_ip, server_port, server_user, &remote_backup_path, &local_backup_path, ssh_key_path)?;

    let volumes_temp_path = format!("{}/volumes", temp_path);

    let volume_names = extract_volumes_from_backup(&local_backup_path, volumes_to_be_restored, &volumes_temp_path)?;

    for volume in &volume_names {
        let volume_backup_path = format!("{}/{}.tar.gz", volumes_temp_path, volume);
        let volume_extract_path = format!("{}/{}", volumes_temp_path, volume);
        decompress_file_from_tar(&volume_backup_path, &volume_extract_path)?;
        replace_volume_data_with_dir(&volume_extract_path, volume)?;
    }

    fs::remove_dir_all(&volumes_temp_path)?;
    fs::remove_file(&local_backup_path)?;

    println!("Restoration completed successfully. The {:?} volumes were restored from {}", volume_names, backup_file_name);
    Ok(())
}

fn extract_volumes_from_backup(local_backup_path: &str, volumes_to_be_restored: &str, temp_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    // Decompress the entire tar.gz archive to the temporary directory
    decompress_file_from_tar(local_backup_path, temp_path)?;

    if volumes_to_be_restored == "all" {
        get_names_of_all_volumes(temp_path)
    } else {
        Ok(volumes_to_be_restored.split(',').map(|s| s.trim().to_string()).collect())
    }
}

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

pub fn replace_volume_data_with_dir(dir_path: &str, volume_name: &str) -> Result<(), Box<dyn Error>> {
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

    start_containers(container_ids)?;
    Ok(())
}

fn collect_paths(dir: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(fs::read_dir(dir)?
        .map(|entry| entry.unwrap().path().to_str().unwrap().to_string())
        .collect())
}