use std::error::Error;
use std::process::Command;

/// Starts a Docker containers by their container IDs.
///
/// This function takes a vector of Docker container IDs and starts each container using
/// the `docker start` command. If the start command fails for any container, an error is returned.
///
/// # Arguments
///
/// * `container_ids` - A vector of strings representing the IDs of the containers to start.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An empty result if successful, or an error if something goes wrong.
pub fn start_containers(container_ids: Vec<String>) -> Result<(), Box<dyn Error>> {
    for container_id in container_ids {
        Command::new("docker")
            .arg("start")
            .arg(container_id)
            .output()?;
    }
    Ok(())
}

/// Stops all Docker containers using a specific volume, excluding the container running this function.
///
/// This function retrieves the list of container IDs that are using a specified Docker volume
/// and stops each of them using the `docker stop` command. It also excludes the container
/// that is executing this function from being stopped.
///
/// # Arguments
///
/// * `volume` - A string slice representing the name of the Docker volume used as a filter to find containers.
///
/// # Returns
///
/// * `Result<Vec<String>, Box<dyn Error>>` - A vector of strings containing the IDs of the stopped containers, or an error if something goes wrong.
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

/// Retrieves the ID of the Docker container running this function.
///
/// This function uses the `hostname` command to get the ID of the Docker container
/// in which the function is being executed. The container ID is returned as a string.
///
/// # Returns
///
/// * `Result<String, Box<dyn Error>>` - The container ID as a string, or an error if something goes wrong.
fn get_my_container_id() -> Result<String, Box<dyn Error>> {
    let output = Command::new("hostname").output()?;
    let container_id = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(container_id)
}
