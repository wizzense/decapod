use std::path::Path;
use std::process::{Command, Stdio};

use crate::core::error;

pub fn container_runtime_available() -> bool {
    find_container_runtime().is_ok()
}

pub fn find_container_runtime() -> Result<String, error::DecapodError> {
    if command_present("podman") {
        return Ok("podman".to_string());
    }
    if command_present("docker") {
        return Ok("docker".to_string());
    }
    Err(error::DecapodError::NotFound(
        "No container runtime found (docker/podman)".to_string(),
    ))
}

fn command_present(cmd: &str) -> bool {
    if command_succeeds(cmd, "--help") || command_succeeds(cmd, "--version") {
        return true;
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| executable_exists(&dir.join(cmd)))
}

fn executable_exists(path: &Path) -> bool {
    path.is_file()
}

fn command_succeeds(cmd: &str, arg: &str) -> bool {
    Command::new(cmd)
        .arg(arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_runtime_error_names_both_supported_runtimes() {
        let err = error::DecapodError::NotFound(
            "No container runtime found (docker/podman)".to_string(),
        );
        assert!(err.to_string().contains("docker/podman"));
    }
}
