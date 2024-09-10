use crate::utility::configs::server_config::ServerConfig;
use ssh2::Session;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

/// A struct for interacting with the server.
pub struct Server {
    config: ServerConfig,
}

impl Server {
    /// Creates a new `Server` instance with the provided `ServerConfig`.
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Establishes an SSH connection to the server.
    ///
    /// # Returns
    ///
    /// * `Result<Session, Box<dyn Error>>` - A `Session` instance if successful, or an error if something goes wrong.
    fn connect(&self) -> Result<Session, Box<dyn Error>> {
        let tcp = TcpStream::connect(format!("{}:{}", self.config.server_ip, self.config.server_port))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        let mut private_key = Vec::new();
        File::open(&self.config.ssh_key_path)?.read_to_end(&mut private_key)?;
        sess.userauth_pubkey_memory(&self.config.server_user, None, &String::from_utf8(private_key)?, None)?;

        if !sess.authenticated() { return Err("Authentication failed.".into()); }

        Ok(sess)
    }

    /// Uploads a local file to the remote server using SCP (Secure Copy Protocol).
    ///
    /// # Arguments
    ///
    /// * `remote_file_path` - The full path on the remote server where the file will be uploaded.
    /// * `local_file_path` - The path of the local file to be uploaded.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - An empty result if the upload is successful, or an error if something goes wrong.
    pub fn upload_file(&self, remote_file_path: &str, local_file_path: &str) -> Result<(), Box<dyn Error>> {
        let sess = self.connect()?;

        let local_file_metadata = fs::metadata(local_file_path)?;
        let file_size = local_file_metadata.len();

        let mut remote_file = sess.scp_send(Path::new(remote_file_path), 0o644, file_size, None)?;

        let mut local_file = File::open(local_file_path)?;
        let mut buffer = Vec::new();
        local_file.read_to_end(&mut buffer)?;
        remote_file.write_all(&buffer)?;

        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        Ok(())
    }

    /// Downloads a file from the remote server using SCP (Secure Copy Protocol).
    ///
    /// # Arguments
    ///
    /// * `remote_file_path` - The full path on the remote server where the file is located.
    /// * `local_file_path` - The path where the downloaded file will be saved locally.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - An empty result if the download is successful, or an error if something goes wrong.
    pub fn download_file(&self, remote_file_path: &str, local_file_path: &str) -> Result<(), Box<dyn Error>> {
        let sess = self.connect()?;

        let (mut remote_file, _) = sess.scp_recv(Path::new(remote_file_path))?;
        let mut local_file = File::create(local_file_path)?;

        let mut buffer = Vec::new();
        remote_file.read_to_end(&mut buffer)?;
        local_file.write_all(&buffer)?;

        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        Ok(())
    }

    /// Retrieves the name of the latest backup file from the remote server.
    ///
    /// # Arguments
    ///
    /// * `server_directory` - The directory on the server where backup files are stored.
    ///
    /// # Returns
    ///
    /// * `Result<String, Box<dyn Error>>` - The name of the latest backup file as a string, or an error if no backups are found or something goes wrong.
    pub fn get_latest_backup_file_name(&self) -> Result<String, Box<dyn Error>> {
        let sess = self.connect()?;

        let mut channel = sess.channel_session()?;
        let command = format!("ls -t {}/backup-*.tar.gz", self.config.server_directory);
        channel.exec(&command)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;

        let backup_files: Vec<&str> = output.lines().collect();

        if let Some(latest_backup) = backup_files.first() {
            let filename = latest_backup.trim_start_matches(&format!("{}/", self.config.server_directory));
            Ok(filename.to_string())
        } else {
            Err("No backup files found on the server.".into())
        }
    }

    /// Deletes a file from the remote server.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The name of the file to be deleted on the remote server.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - An empty result if the file deletion is successful, or an error if something goes wrong.
    pub fn delete_file(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let sess = self.connect()?;

        let mut channel = sess.channel_session()?;
        let command = format!("rm {}", file_name);
        channel.exec(&command)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;

        if channel.exit_status()? == 0 {
            Ok(())
        } else {
            Err(format!("Failed to delete file: {}", output).into())
        }
    }

    /// Lists file names in the specified directory on the remote server.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, Box<dyn Error>>` - A vector of file names if successful, or an error if something goes wrong.
    pub fn list_files(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let sess = self.connect()?;

        let mut channel = sess.channel_session()?;
        let command = format!("ls -1 {}", self.config.server_directory); // List files in the server's backup directory
        channel.exec(&command)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;

        if channel.exit_status()? == 0 {
            let files: Vec<String> = output.lines().map(|line| line.to_string()).collect();
            Ok(files)
        } else {
            Err("Failed to list files.".into())
        }
    }
}
