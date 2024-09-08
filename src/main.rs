use chrono::Local;
use flate2::write::GzEncoder;
use flate2::Compression;
use ssh2::Session;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;
use tar::Builder;

fn compress_backup_folder(backup_path: &str, output_file: &str) -> std::io::Result<()> {
    let tar_gz = File::create(output_file)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all(".", backup_path)?;

    Ok(())
}

fn upload_via_sftp(server_ip: &str,
                   server_port: &str,
                   server_user: &str,
                   remote_path: &str,
                   local_file: &str,
                   ssh_key_path: &str) -> Result<(), Box<dyn Error>>
{
    let tcp = TcpStream::connect(format!("{}:{}", server_ip, server_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    let mut private_key = Vec::new();
    File::open(ssh_key_path)?.read_to_end(&mut private_key)?;

    sess.userauth_pubkey_memory(server_user, None, &String::from_utf8(private_key)?, None)?;

    if !sess.authenticated() {
        return Err("Authentication failed.".into());
    }

    // Get the file size
    let local_file_metadata = fs::metadata(local_file)?;
    let file_size = local_file_metadata.len();

    // Open the remote file for writing
    let mut remote_file = sess.scp_send(Path::new(&remote_path), 0o644, file_size, None)?;

    // Open the local file and read its content
    let mut local_file = File::open(local_file)?;
    let mut buffer = Vec::new();
    local_file.read_to_end(&mut buffer)?;

    // Write the content to the remote file
    remote_file.write_all(&buffer)?;

    // Close the channel and ensure the whole content is transferred
    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    Ok(())
}

fn get_my_container_id() -> Result<String, Box<dyn Error>> {
    let output = Command::new("hostname").output()?;
    let container_id = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(container_id)
}

fn start_containers(container_ids: Vec<String>) -> Result<(), Box<dyn Error>> {
    for container_id in container_ids {
        Command::new("docker")
            .arg("start")
            .arg(container_id)
            .output()?;
    }
    Ok(())
}

fn stop_containers(volume: &str) -> Result<Vec<String>, Box<dyn Error>> {

    // Get the list of containers using the volume
    let output = Command::new("docker")
        .arg("ps")
        .arg("-q")
        .arg("--filter")
        .arg(format!("volume={}", volume))
        .output()?;

    let containers = String::from_utf8(output.stdout)?;
    let mut container_ids: Vec<String> = Vec::new();

    for container_id in containers.trim().split('\n') {
        if container_id.is_empty() || container_id == get_my_container_id()? { continue; }

        Command::new("docker")
            .arg("stop")
            .arg(container_id)
            .output()?;

        container_ids.push(container_id.to_string());
    }

    Ok(container_ids)
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let server_ip = env::var("SERVER_IP")?;
    let server_port = env::var("SERVER_PORT")?;
    let server_user = env::var("SERVER_USER")?;
    let server_directory = env::var("SERVER_DIRECTORY")?;

    const BACKUP_PATH: &str = "/backup";
    const SSH_KEY_PATH: &str = "/.ssh/id_rsa";

    // Generate timestamp for the backup filename
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%dT%H-%M-%S").to_string();
    let backup_name = format!("backup-{}.tar.gz", timestamp);
    let backup_archive_path = format!("/tmp/{}", backup_name);
    let server_backup_path = format!("{}/{}", server_directory, backup_name);

    let media_container_ids = stop_containers("deels_media")?;
    let db_container_ids = stop_containers("deels_db")?;

    compress_backup_folder(BACKUP_PATH, &backup_archive_path)?;

    start_containers(media_container_ids)?;
    start_containers(db_container_ids)?;

    upload_via_sftp(&server_ip, &server_port, &server_user, &server_backup_path, &backup_archive_path, SSH_KEY_PATH)?;
    fs::remove_file(&backup_archive_path)?;

    println!("Backup completed successfully.");
    Ok(())
}
