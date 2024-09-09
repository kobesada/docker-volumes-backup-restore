use std::env;
use std::error::Error;

/// A struct to hold server configuration parameters.
///
/// The `ServerConfig` struct contains the following fields:
///
/// - `server_ip`: The IP address of the server.
/// - `server_port`: The port on which the server is running.
/// - `server_user`: The username for accessing the server.
/// - `server_directory`: The directory on the server where backups are stored.
/// - `ssh_key_path`: The path to the SSH private key used for authenticating to the server.
#[derive(Clone)]
pub struct ServerConfig {
    pub server_ip: String,
    pub server_port: String,
    pub server_user: String,
    pub server_directory: String,
    pub ssh_key_path: String,
}

impl ServerConfig {
    /// Creates a new `ServerConfig` instance by loading values from environment variables.
    ///
    /// This method reads the following environment variables:
    ///
    /// - `SERVER_IP`: The IP address of the server.
    /// - `SERVER_PORT`: The port on which the server is running.
    /// - `SERVER_USER`: The username for accessing the server.
    /// - `SERVER_DIRECTORY`: The directory on the server where backups are stored.
    ///
    /// The `ssh_key_path` must be provided as a parameter.
    ///
    /// # Arguments
    ///
    /// * `ssh_key_path` - The path to the SSH private key used for authenticating to the server.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if any of the environment variables are not set or cannot be read.
    ///
    pub fn new_from_env(ssh_key_path: String) -> Result<Self, Box<dyn Error>> {
        let server_ip = env::var("SERVER_IP")?;
        let server_port = env::var("SERVER_PORT")?;
        let server_user = env::var("SERVER_USER")?;
        let server_directory = env::var("SERVER_DIRECTORY")?;

        Ok(Self { server_ip, server_port, server_user, server_directory, ssh_key_path })
    }
}
