use crate::utility::compression::{compress_files_to_tar, compress_folder_to_tar};
use crate::utility::docker::{start_containers, stop_containers};
use crate::utility::server::upload_to_server;
use chrono::Local;
use cron::Schedule;
use std::error::Error;
use std::fs;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

/// Configures and manages a scheduled backup process based on a cron expression.
///
/// This function continuously checks for the next scheduled backup time as defined
/// by the `backup_cron` expression. When the next scheduled time arrives, it performs
/// the backup process by calling `run_backup`. It then logs the time of the next backup
/// and waits until that time is reached.
///
/// # Arguments
///
/// * `server_ip` - The IP address of the server to which the backup will be uploaded.
/// * `server_port` - The port on which the server is listening.
/// * `server_user` - The username for authenticating to the server.
/// * `server_directory` - The directory on the server where backups will be stored.
/// * `backup_cron` - A cron expression defining the backup schedule.
/// * `ssh_key_path` - The path to the SSH private key used for authenticating to the server.
/// * `temp_path` - The local path where temporary backup files will be stored.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An empty result if successful, or an error if something goes wrong.
pub async fn configure_cron_scheduled_backup(server_ip: &str,
                                             server_port: &str,
                                             server_user: &str,
                                             server_directory: &str,
                                             backup_cron: &str,
                                             ssh_key_path: &str,
                                             temp_path: &str) -> Result<(), Box<dyn Error>> {
    let schedule = Schedule::from_str(backup_cron)?;

    let mut last = None;
    loop {
        let now = Local::now();
        let upcoming = schedule.upcoming(Local).next();

        if let Some(next_time) = upcoming {
            if last.is_some() && last.unwrap() >= next_time { continue; }
            last = Some(next_time);

            println!("Next backup will be performed at: {}", next_time);

            let duration = next_time - now;
            sleep(Duration::from_secs(duration.num_seconds() as u64)).await;

            run_backup(&server_ip, &server_port, &server_user, &server_directory, &ssh_key_path, temp_path)?;
        }
    }
}

/// Performs a backup operation by compressing Docker volumes (folders in "/backup" folder)
/// and uploading them to a remote server.
///
/// This function stops the containers associated with each volume, compresses the volume's
/// data into a tar.gz archive, and then restarts the containers. It then combines all
/// individual volume backups into a single archive, which is uploaded to the specified
/// server.
///
/// # Arguments
///
/// * `server_ip` - The IP address of the server to which the backup will be uploaded.
/// * `server_port` - The port on which the server is listening.
/// * `server_user` - The username for authenticating to the server.
/// * `server_directory` - The directory on the server where the backup will be stored.
/// * `ssh_key_path` - The path to the SSH private key used for authenticating to the server.
/// * `temp_path` - The local path where temporary backup files will be stored.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An empty result if successful, or an error if something goes wrong.
pub fn run_backup(server_ip: &str,
                  server_port: &str,
                  server_user: &str,
                  server_directory: &str,
                  ssh_key_path: &str,
                  temp_path: &str) -> Result<(), Box<dyn Error>> {
    const BACKUP_PATH: &str = "/backup";

    let mut archives_paths: Vec<String> = Vec::new();

    // Compress each volume directory into a tar.gz archive
    for volume in &get_volume_dirs(BACKUP_PATH)? {
        let backup_archive_path = format!("{}/{}.tar.gz", temp_path, volume);
        let volume_path = format!("{}/{}", BACKUP_PATH, volume);
        archives_paths.push(backup_archive_path.clone());

        let container_ids = stop_containers(volume)?;
        compress_folder_to_tar(&volume_path, &backup_archive_path)?;
        start_containers(container_ids)?;
    }

    // Combine all volume archives into a single backup file with a timestamp
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%dT%H-%M-%S").to_string();
    let combined_backup_name = format!("backup-{}.tar.gz", timestamp);
    let combined_backup_archive_path = format!("{}/{}", temp_path, combined_backup_name);
    let server_combined_backup_path = format!("{}/{}", server_directory, combined_backup_name);
    compress_files_to_tar(&archives_paths, &combined_backup_archive_path)?;

    // Upload backup to server and delete temporary files
    upload_to_server(server_ip, server_port, server_user, &server_combined_backup_path, &combined_backup_archive_path, ssh_key_path)?;
    fs::remove_dir_all(temp_path)?;

    println!("Backup completed successfully.");
    Ok(())
}

/// Retrieves the names of all volumes (directories) located in the specified backup folder.
///
/// This function reads the contents of the backup folder and returns a vector containing
/// the names of all directories (i.e., volume names) found there.
///
/// # Arguments
///
/// * `backup_folder_path` - The path to the folder with volumes.
///
/// # Returns
///
/// * `Result<Vec<String>, Box<dyn Error>>` - A vector of volume names, or an error if something goes wrong.
fn get_volume_dirs(backup_folder_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(fs::read_dir(backup_folder_path)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect())
}
