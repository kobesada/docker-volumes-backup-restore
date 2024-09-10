use crate::utility::compression::{compress_files_to_tar, compress_folder_to_tar};
use crate::utility::configs::retention_policy::RetentionPolicy;
use crate::utility::configs::server_config::ServerConfig;
use crate::utility::docker::{start_containers, stop_containers};
use crate::utility::server::Server;
use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone, Utc};
use cron::Schedule;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tokio::time::sleep;

/// Configures and manages a scheduled backup process based on a cron expression.
///
/// This function continuously checks for the next scheduled backup time as defined
/// by the `backup_cron` expression. When the next scheduled time arrives, it performs
/// the backup process by calling `run_backup`. After completing the backup, it removes
/// old backups based on the retention policy provided in `retention_config`. It then logs
/// the time of the next backup and waits until that time is reached.
///
/// # Arguments
///
/// * `server_config` - A reference to a `ServerConfig` containing connection information for the server.
/// * `retention_config` - A reference to a `RetentionConfig` that defines the retention policy for old backups.
/// * `backup_cron` - A cron expression that defines the schedule for the backups.
/// * `temp_path` - The local path where temporary backup files will be stored.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - Returns an empty result if the operation is successful.
///   Otherwise, it returns an error wrapped in a `Box<dyn Error>`.
pub async fn configure_cron_scheduled_backup(server_config: &ServerConfig,
                                             retention_config: &RetentionPolicy,
                                             backup_cron: &str,
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
            sleep(std::time::Duration::from_secs(duration.num_seconds() as u64)).await;

            run_backup(server_config, retention_config, temp_path)?;
        }
    }
}

/// Performs a backup operation by compressing Docker volumes (folders in the "/backup" directory)
/// and uploading them to a remote server. Afterward, the function removes old backups according
/// to the retention policy provided in `retention_config`.
///
/// This function stops the containers associated with each volume, compresses the volume's
/// data into a tar.gz archive, and then restarts the containers. It then combines all
/// individual volume backups into a single archive, which is uploaded to the specified
/// server.
///
/// After the upload, the function removes temporary backup files and runs the `remove_old_backups`
/// function to ensure old backups are deleted based on the specified retention policy.
///
/// # Arguments
///
/// * `server_config` - A reference to a `ServerConfig` containing connection information for the server.
/// * `retention_config` - A reference to a `RetentionConfig` that defines how many backups to retain.
/// * `temp_path` - The local path where temporary backup files will be stored.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An empty result if successful, or an error if something goes wrong.
pub fn run_backup(server_config: &ServerConfig, retention_config: &RetentionPolicy, temp_path: &str) -> Result<(), Box<dyn Error>> {
    const BACKUP_PATH: &str = "/backup";

    // Create the temp directory if it doesn't exist
    if !Path::new(temp_path).exists() { fs::create_dir_all(temp_path)?; }

    let mut archives_paths: Vec<String> = Vec::new();

    remove_old_backups(server_config, retention_config)?;

    let volume_names = get_volume_dirs(BACKUP_PATH)?;

    // Compress each volume directory into a tar.gz archive
    for volume in &volume_names {
        let backup_archive_path = format!("{}/{}.tar.gz", temp_path, volume);
        let volume_path = format!("{}/{}", BACKUP_PATH, volume);
        archives_paths.push(backup_archive_path.clone());

        let container_ids = stop_containers(volume)?;
        let result = compress_folder_to_tar(&volume_path, &backup_archive_path);
        start_containers(container_ids)?;
        result?
    }

    // Combine all volume archives into a single backup file with a timestamp
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%dT%H-%M-%S").to_string();
    let combined_backup_name = format!("backup-{}.tar.gz", timestamp);
    let combined_backup_archive_path = format!("{}/{}", temp_path, combined_backup_name);
    let server_combined_backup_path = format!("{}/{}", server_config.server_directory, combined_backup_name);
    compress_files_to_tar(&archives_paths, &combined_backup_archive_path)?;

    // Upload backup to server and delete temporary files
    Server::new(server_config.clone()).upload_file(&server_combined_backup_path,
                                                   &combined_backup_archive_path)?;
    fs::remove_dir_all(temp_path)?;

    remove_old_backups(server_config, retention_config)?;

    println!("Backup completed successfully. The {:?} volumes have been backed up to the {}",
             volume_names, server_combined_backup_path);
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

/// Removes old backups from the server based on the retention policy.
///
/// This function connects to the server using the provided configuration,
/// retrieves the list of backup files, and determines which backups to delete
/// according to the retention policy.
///
/// # Arguments
///
/// * `server_config` - A reference to a `ServerConfig` struct containing the server's configuration.
/// * `retention_config` - A reference to a `RetentionPolicy` struct defining the backup retention rules.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - Returns `Ok(())` on success, or an `Error` if something goes wrong.
///
/// # Errors
///
/// This function returns errors that might occur while listing or deleting files from the server.
pub fn remove_old_backups(
    server_config: &ServerConfig,
    retention_config: &RetentionPolicy,
) -> Result<(), Box<dyn Error>> {
    let server = Server::new(server_config.clone());

    // Fetch the list of backup files from the server
    let backup_names = server.list_files()?.into_iter().filter(|file_name|
        file_name.starts_with("backup-") && file_name.ends_with(".tar.gz")).collect();

    // Determine which backups to delete based on the retention policy
    let backups_to_delete = filter_backups_to_delete(backup_names, retention_config);

    // Delete old backups that are not retained
    for file_name in backups_to_delete {
        server.delete_file(&file_name)?;
    }

    Ok(())
}

/// Filters backups to determine which ones should be deleted based on the retention policy.
///
/// This function first filters out backups that are older than the retention period. Then,
/// it determines which backups to keep based on the specified retention count and interval.
/// Backups are evenly distributed through the retention period, ensuring that the latest backup
/// is always retained and the amount of backups in the period does not exceed the retention count.
/// The function returns a vector of backup file names that should be deleted.
///
/// # Arguments
///
/// * `backups` - A vector of backup file names (strings) to be evaluated.
/// * `retention` - A reference to a `RetentionPolicy` struct defining the backup retention rules.
///
/// # Returns
///
/// * `Vec<String>` - A vector of backup file names that should be deleted.
fn filter_backups_to_delete(backups: Vec<String>, retention: &RetentionPolicy) -> Vec<String> {
    let now = Utc::now();
    let retention_period = Duration::days(retention.period as i64);

    // Parse backups and filter based on the retention period
    let mut backups_with_dates: Vec<(String, DateTime<Utc>)> = backups.iter()
        .filter_map(|b| parse_backup_date(b).map(|d| (b.clone(), d))) // Clone the backup string here
        .collect();

    // Filter out backups older than the retention period
    backups_with_dates.retain(|(_, date)| date > &(now - retention_period));

    // Sort backups by date in descending order (newest first)
    backups_with_dates.sort_by(|a, b| b.1.cmp(&a.1));

    // Collect backups to keep, ensuring one per interval
    let mut retained_backups: HashSet<String> = HashSet::new();

    // Filter the backups that they are evenly distributed through the retention period.
    while retained_backups.len() < retention.count {
        let keep_interval = (backups_with_dates.len() as f64 / (retention.count - retained_backups.len()) as f64).ceil() as usize;
        let mut last_keep_index = 0;

        for (i, (backup, _)) in backups_with_dates.iter().enumerate() {
            if i == 0 || (i - last_keep_index) >= keep_interval {
                retained_backups.insert(backup.clone());
                last_keep_index = i;
            }
        }
        backups_with_dates.retain(|(backup_name, _)| !retained_backups.contains(backup_name));

        if backups_with_dates.is_empty() { break; }
    }

    // Filter original backups to determine which should be deleted
    backups.into_iter()
        .filter(|b| !retained_backups.contains(b))
        .collect()
}

/// Parses a backup file name to extract the date and time it was created.
///
/// The file name should start with "backup-" and end with ".tar.gz". The date and time
/// should be in the format "YYYY-MM-DDTHH-MM-SS". If the file name does not conform to
/// this format, `None` is returned.
///
/// # Arguments
///
/// * `backup` - A string slice containing the backup file name.
///
/// # Returns
///
/// * `Option<DateTime<Utc>>` - Returns `Some(DateTime<Utc>)` if parsing is successful,
///   or `None` if the file name does not match the expected format.
fn parse_backup_date(backup: &str) -> Option<DateTime<Utc>> {
    let prefix = "backup-";
    let suffix = ".tar.gz";

    if !backup.starts_with(prefix) || !backup.ends_with(suffix) {
        return None;
    }

    let datetime_str = &backup[prefix.len()..backup.len() - suffix.len()];
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H-%M-%S") {
        return Some(Utc.from_utc_datetime(&naive_dt));
    }

    None
}
