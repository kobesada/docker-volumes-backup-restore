use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use ssh2::Session;

pub fn upload_via_scp(server_ip: &str,
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