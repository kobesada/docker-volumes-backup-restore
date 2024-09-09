use ssh2::Session;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

pub fn upload_to_server(server_ip: &str,
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

    if !sess.authenticated() { return Err("Authentication failed.".into()); }

    let local_file_metadata = fs::metadata(local_file)?;
    let file_size = local_file_metadata.len();

    let mut remote_file = sess.scp_send(Path::new(&remote_path), 0o644, file_size, None)?;
    let mut local_file = File::open(local_file)?;
    let mut buffer = Vec::new();
    local_file.read_to_end(&mut buffer)?;
    remote_file.write_all(&buffer)?;

    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    Ok(())
}

pub fn download_from_server(server_ip: &str,
                            server_port: &str,
                            server_user: &str,
                            remote_path: &str,
                            local_file: &str,
                            ssh_key_path: &str) -> Result<(), Box<dyn Error>> {
    let tcp = TcpStream::connect(format!("{}:{}", server_ip, server_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    let mut private_key = Vec::new();
    File::open(ssh_key_path)?.read_to_end(&mut private_key)?;

    sess.userauth_pubkey_memory(server_user, None, &String::from_utf8(private_key)?, None)?;

    if !sess.authenticated() { return Err("Authentication failed.".into()); }

    let (mut remote_file, _) = sess.scp_recv(Path::new(&remote_path))?;
    let mut local_file = File::create(local_file)?;
    let mut buffer = Vec::new();
    remote_file.read_to_end(&mut buffer)?;
    local_file.write_all(&buffer)?;

    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    Ok(())
}

pub fn get_latest_backup_file_name_from_server(server_ip: &str,
                                               server_port: &str,
                                               server_user: &str,
                                               server_directory: &str,
                                               ssh_key_path: &str) -> Result<String, Box<dyn Error>>
{
    let tcp = TcpStream::connect(format!("{}:{}", server_ip, server_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    let mut private_key = Vec::new();
    File::open(ssh_key_path)?.read_to_end(&mut private_key)?;
    sess.userauth_pubkey_memory(server_user, None, &String::from_utf8(private_key)?, None)?;

    if !sess.authenticated() { return Err("Authentication failed.".into()); }

    let mut channel = sess.channel_session()?;
    let command = format!("ls -t {}/backup-*.tar.gz", server_directory);
    channel.exec(&command)?;

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
