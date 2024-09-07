use flate2::write::GzEncoder;
use flate2::Compression;
use ssh2::Session;
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
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
                   local_file: &str, ) -> Result<(), Box<dyn std::error::Error>>
{
    let tcp = TcpStream::connect(format!("{}:{}", server_ip, server_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    let mut private_key = Vec::new();
    File::open("/.ssh/id_rsa")?.read_to_end(&mut private_key)?;

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


fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let server_ip = env::var("SERVER_IP")?;
    let server_port = env::var("SERVER_PORT")?;
    let server_user = env::var("SERVER_USER")?;
    let server_directory = env::var("SERVER_DIRECTORY")?;

    let backup_path = "/backup";
    let output_file = "/tmp/backup.tar.gz";

    compress_backup_folder(backup_path, output_file)?;
    upload_via_sftp(&server_ip, &server_port, &server_user, &server_directory, output_file)?;
    fs::remove_file(output_file)?;

    println!("Backup completed successfully.");
    Ok(())
}
