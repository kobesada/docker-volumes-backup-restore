mod backup;

use crate::backup::backup::configure_backup;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let server_ip = env::var("SERVER_IP")?;
    let server_port = env::var("SERVER_PORT")?;
    let server_user = env::var("SERVER_USER")?;
    let server_directory = env::var("SERVER_DIRECTORY")?;
    let backup_cron = env::var("BACKUP_CRON")?;
    const SSH_KEY_PATH: &str = "/.ssh/id_rsa";

    configure_backup(&server_ip, &server_port, &server_user, &server_directory, &backup_cron, SSH_KEY_PATH).await?;

    Ok(())
}

