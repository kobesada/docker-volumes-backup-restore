use ssh2::Session;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

/// Uploads a local file to a remote server using SCP (Secure Copy Protocol).
///
/// This function establishes an SSH connection to the specified server, authenticates
/// using a private key, and uploads a local file to the specified remote path.
///
/// # Arguments
///
/// * `server_ip` - A string slice representing the IP address of the remote server.
/// * `server_port` - A string slice representing the SSH port on the remote server.
/// * `server_user` - A string slice representing the username for SSH authentication.
/// * `remote_path` - A string slice representing the full path on the remote server where the file will be uploaded.
/// * `local_file` - A string slice representing the path of the local file to be uploaded.
/// * `ssh_key_path` - A string slice representing the path to the SSH private key used for authentication.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An empty result if the upload is successful, or an error if something goes wrong.
pub fn upload_to_server(server_ip: &str,
                        server_port: &str,
                        server_user: &str,
                        remote_path: &str,
                        local_file: &str,
                        ssh_key_path: &str) -> Result<(), Box<dyn Error>> {
    // Establish an SSH connection to the server
    let tcp = TcpStream::connect(format!("{}:{}", server_ip, server_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Load and use the private key for authentication
    let mut private_key = Vec::new();
    File::open(ssh_key_path)?.read_to_end(&mut private_key)?;
    sess.userauth_pubkey_memory(server_user, None, &String::from_utf8(private_key)?, None)?;

    // Ensure authentication is successful
    if !sess.authenticated() { return Err("Authentication failed.".into()); }

    // Get the local file's metadata and size
    let local_file_metadata = fs::metadata(local_file)?;
    let file_size = local_file_metadata.len();

    // Open the remote file for writing
    let mut remote_file = sess.scp_send(Path::new(&remote_path), 0o644, file_size, None)?;

    // Read the local file's content and write it to the remote file
    let mut local_file = File::open(local_file)?;
    let mut buffer = Vec::new();
    local_file.read_to_end(&mut buffer)?;
    remote_file.write_all(&buffer)?;

    // Complete the SCP transfer
    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    Ok(())
}

/// Downloads a file from a remote server using SCP (Secure Copy Protocol).
///
/// This function establishes an SSH connection to the specified server, authenticates
/// using a private key, and downloads a file from the specified remote path to a local file.
///
/// # Arguments
///
/// * `server_ip` - A string slice representing the IP address of the remote server.
/// * `server_port` - A string slice representing the SSH port on the remote server.
/// * `server_user` - A string slice representing the username for SSH authentication.
/// * `remote_path` - A string slice representing the full path on the remote server where the file is located.
/// * `local_file` - A string slice representing the path where the downloaded file will be saved locally.
/// * `ssh_key_path` - A string slice representing the path to the SSH private key used for authentication.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An empty result if the download is successful, or an error if something goes wrong.
pub fn download_from_server(server_ip: &str,
                            server_port: &str,
                            server_user: &str,
                            remote_path: &str,
                            local_file: &str,
                            ssh_key_path: &str) -> Result<(), Box<dyn Error>> {
    // Establish an SSH connection to the server
    let tcp = TcpStream::connect(format!("{}:{}", server_ip, server_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Load and use the private key for authentication
    let mut private_key = Vec::new();
    File::open(ssh_key_path)?.read_to_end(&mut private_key)?;
    sess.userauth_pubkey_memory(server_user, None, &String::from_utf8(private_key)?, None)?;

    // Ensure authentication is successful
    if !sess.authenticated() { return Err("Authentication failed.".into()); }

    // Open the remote file for reading and create the local file
    let (mut remote_file, _) = sess.scp_recv(Path::new(&remote_path))?;
    let mut local_file = File::create(local_file)?;

    // Read the remote file's content and write it to the local file
    let mut buffer = Vec::new();
    remote_file.read_to_end(&mut buffer)?;
    local_file.write_all(&buffer)?;

    // Complete the SCP transfer
    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    Ok(())
}

/// Retrieves the name of the latest backup file from the remote server.
///
/// This function establishes an SSH connection to the specified server, authenticates
/// using a private key, and retrieves the name of the latest backup file in the specified
/// directory by executing a shell command on the remote server.
///
/// # Arguments
///
/// * `server_ip` - A string slice representing the IP address of the remote server.
/// * `server_port` - A string slice representing the SSH port on the remote server.
/// * `server_user` - A string slice representing the username for SSH authentication.
/// * `server_directory` - A string slice representing the directory on the server where backup files are stored.
/// * `ssh_key_path` - A string slice representing the path to the SSH private key used for authentication.
///
/// # Returns
///
/// * `Result<String, Box<dyn Error>>` - The name of the latest backup file as a string, or an error if no backups are found or something goes wrong.
pub fn get_latest_backup_file_name_from_server(server_ip: &str,
                                               server_port: &str,
                                               server_user: &str,
                                               server_directory: &str,
                                               ssh_key_path: &str) -> Result<String, Box<dyn Error>> {
    // Establish an SSH connection to the server
    let tcp = TcpStream::connect(format!("{}:{}", server_ip, server_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Load and use the private key for authentication
    let mut private_key = Vec::new();
    File::open(ssh_key_path)?.read_to_end(&mut private_key)?;
    sess.userauth_pubkey_memory(server_user, None, &String::from_utf8(private_key)?, None)?;

    // Ensure authentication is successful
    if !sess.authenticated() { return Err("Authentication failed.".into()); }

    // Execute a command on the server to list the backup files, sorted by modification time
    let mut channel = sess.channel_session()?;
    let command = format!("ls -t {}/backup-*.tar.gz", server_directory);
    channel.exec(&command)?;

    // Capture the output and extract the latest backup file name
    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    channel.wait_close()?;

    let backup_files: Vec<&str> = output.lines().collect();

    if let Some(latest_backup) = backup_files.first() {
        let filename = latest_backup.trim_start_matches(&format!("{}/", server_directory));
        Ok(filename.to_string())
    } else {
        Err("No backup files found on the server.".into())
    }
}
