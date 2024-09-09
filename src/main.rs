mod restore;
mod backup;
mod utility;

use crate::backup::configure_cron_scheduled_backup;
use crate::restore::restore_volumes;
use std::error::Error;
use std::path::Path;
use std::{env, fs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let action = env::var("ACTION")?;
    let server_ip = env::var("SERVER_IP")?;
    let server_port = env::var("SERVER_PORT")?;
    let server_user = env::var("SERVER_USER")?;
    let server_directory = env::var("SERVER_DIRECTORY")?;
    const SSH_KEY_PATH: &str = ".ssh/id_rsa";
    const BACKUP_TEMP_PATH: &str = "backup-temp";

    // Create the temp directory if it doesn't exist
    if !Path::new(BACKUP_TEMP_PATH).exists() { fs::create_dir_all(BACKUP_TEMP_PATH)?; }

    match action.as_str() {
        "backup" => {
            let backup_cron = env::var("BACKUP_CRON")?;
            configure_cron_scheduled_backup(
                &server_ip,
                &server_port,
                &server_user,
                &server_directory,
                &backup_cron,
                SSH_KEY_PATH,
                BACKUP_TEMP_PATH,
            ).await?;
        }
        "restore" => {
            let backup_to_be_restored = env::var("BACKUP_TO_BE_RESTORED")?;
            let volume_to_be_restored = env::var("VOLUME_TO_BE_RESTORED")?;
            restore_volumes(
                &server_ip,
                &server_port,
                &server_user,
                &server_directory,
                &backup_to_be_restored,
                &volume_to_be_restored,
                SSH_KEY_PATH,
                BACKUP_TEMP_PATH,
            )?;
        }
        _ => {
            return Err("Invalid ACTION specified. Use 'backup' or 'restore'.".into());
        }
    }

    Ok(())
}
