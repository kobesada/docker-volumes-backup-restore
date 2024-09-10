mod restore;
mod backup;
mod utility;

use crate::backup::{configure_cron_scheduled_backup, run_backup};
use crate::restore::restore_volumes;
use crate::utility::configs::retention_policy::RetentionPolicy;
use crate::utility::configs::server_config::ServerConfig;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let server_config = ServerConfig::new_from_env(".ssh/id_rsa".to_string())?;
    let action = env::var("ACTION")?;

    const BACKUP_TEMP_PATH: &str = "backup-temp";

    match action.as_str() {
        "backup" => {
            let retention_config = RetentionPolicy::new_from_env()?;

            if let Ok(backup_cron) = env::var("BACKUP_CRON") {
                configure_cron_scheduled_backup(&server_config,
                                                &retention_config,
                                                &backup_cron,
                                                BACKUP_TEMP_PATH).await?;
            } else { run_backup(&server_config, &retention_config, BACKUP_TEMP_PATH)?; }
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
