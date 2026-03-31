pub mod connection;

use adb_client::server::ADBServer;
use adb_client::ADBDeviceExt;
use anyhow::{Context, Result};

/// Get a connected ADB server instance (connects to local adb server on default port).
pub fn server() -> Result<ADBServer> {
    let server = ADBServer::default();
    Ok(server)
}

/// Execute a shell command on the first connected device and return stdout.
pub fn shell(command: &str) -> Result<Vec<u8>> {
    let mut server = server()?;
    let mut device = server
        .get_device()
        .context("No device connected. Is a device/emulator attached via ADB?")?;

    let mut output = Vec::new();
    device
        .shell_command(&command, Some(&mut output), None)
        .context("Failed to execute shell command on device")?;

    Ok(output)
}

/// Execute a shell command and return output as a String.
pub fn shell_str(command: &str) -> Result<String> {
    let output = shell(command)?;
    Ok(String::from_utf8_lossy(&output).to_string())
}

/// Pull a file from the device into memory.
pub fn pull(remote_path: &str) -> Result<Vec<u8>> {
    let mut server = server()?;
    let mut device = server
        .get_device()
        .context("No device connected")?;

    let mut buf = Vec::new();
    device
        .pull(&remote_path, &mut buf)
        .with_context(|| format!("Failed to pull {remote_path}"))?;

    Ok(buf)
}
