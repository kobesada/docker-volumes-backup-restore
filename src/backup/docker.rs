use std::error::Error;
use std::process::Command;

pub fn start_containers(container_ids: Vec<String>) -> Result<(), Box<dyn Error>> {
    for container_id in container_ids {
        Command::new("docker")
            .arg("start")
            .arg(container_id)
            .output()?;
    }
    Ok(())
}

pub fn stop_containers(volume: &str) -> Result<Vec<String>, Box<dyn Error>> {
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

fn get_my_container_id() -> Result<String, Box<dyn Error>> {
    let output = Command::new("hostname").output()?;
    let container_id = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(container_id)
}