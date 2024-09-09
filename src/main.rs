mod restore;
mod backup;
mod utility;

use crate::backup::configure_cron_scheduled_backup;
use crate::restore::restore_volumes;
use crate::utility::configs::server_config::ServerConfig;
use std::error::Error;
use std::path::Path;
use std::{env, fs};
use crate::utility::configs::retention_config::RetentionConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let server_config = ServerConfig::new_from_env(".ssh/id_rsa".to_string())?;
    let action = env::var("ACTION")?;

    const BACKUP_TEMP_PATH: &str = "backup-temp";

    // Create the temp directory if it doesn't exist
    if !Path::new(BACKUP_TEMP_PATH).exists() { fs::create_dir_all(BACKUP_TEMP_PATH)?; }

    match action.as_str() {
        "backup" => {
            let backup_cron = env::var("BACKUP_CRON")?;
            let retention_config = RetentionConfig::new_from_env()?;

            configure_cron_scheduled_backup(&server_config,
                                            &retention_config,
                                            &backup_cron,
                                            BACKUP_TEMP_PATH).await?;
        }
        "restore" => {
            let backup_to_be_restored = env::var("BACKUP_TO_BE_RESTORED")?;
            let volume_to_be_restored = env::var("VOLUME_TO_BE_RESTORED")?;
            restore_volumes(&server_config,
                            &backup_to_be_restored,
                            &volume_to_be_restored,
                            BACKUP_TEMP_PATH)?;
        }
        _ => {
            return Err("Invalid ACTION specified. Use 'backup' or 'restore'.".into());
        }
    }

    Ok(())
}
