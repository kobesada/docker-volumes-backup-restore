use crate::backup::compression::{compress_files_to_tar, compress_folder_to_tar};
use crate::backup::docker::{start_containers, stop_containers};
use crate::backup::scp::upload_via_scp;
use chrono::Local;
use cron::Schedule;
use std::error::Error;
use std::fs;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

fn get_volume_dirs(backup_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(fs::read_dir(backup_path)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect())
}

fn run_backup(
    server_ip: &str,
    server_port: &str,
    server_user: &str,
    server_directory: &str,
    ssh_key_path: &str,
) -> Result<(), Box<dyn Error>> {
    const BACKUP_PATH: &str = "/backup";

    let mut archives_paths: Vec<String> = Vec::new();

    for volume in &get_volume_dirs(BACKUP_PATH)? {
        let backup_archive_path = format!("/tmp/{}_backup.tar.gz", volume);
        let volume_path = format!("{}/{}", BACKUP_PATH, volume);
        archives_paths.push(backup_archive_path.clone());

        let container_ids = stop_containers(volume)?;
        compress_folder_to_tar(&volume_path, &backup_archive_path)?;
        start_containers(container_ids)?;
    }

    let now = Local::now();
    let timestamp = now.format("%Y-%m-%dT%H-%M-%S").to_string();
    let combined_backup_name = format!("backup-{}.tar.gz", timestamp);
    let combined_backup_archive_path = format!("/tmp/{}", combined_backup_name);
    let server_combined_backup_path = format!("{}/{}", server_directory, combined_backup_name);

    compress_files_to_tar(&archives_paths, &combined_backup_archive_path)?;
    upload_via_scp(server_ip, server_port, server_user, &server_combined_backup_path, &combined_backup_archive_path, ssh_key_path)?;
    fs::remove_file(&combined_backup_archive_path)?;

    println!("Backup completed successfully.");
    Ok(())
}

pub async fn configure_backup(
    server_ip: &str,
    server_port: &str,
    server_user: &str,
    server_directory: &str,
    backup_cron: &str,
    ssh_key_path: &str,
) -> Result<(), Box<dyn Error>> {
    let schedule = Schedule::from_str(backup_cron)?;

    let mut last = None;
    loop {
        let now = Local::now();
        let upcoming = schedule.upcoming(Local).next();

        if let Some(next_time) = upcoming {
            if last.is_some() && last.unwrap() >= next_time { continue; }
            last = Some(next_time);

            let duration = next_time - now;
            sleep(Duration::from_secs(duration.num_seconds() as u64)).await;

            run_backup(&server_ip, &server_port, &server_user, &server_directory, &ssh_key_path)?;
        }
    }
}
