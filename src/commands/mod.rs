pub mod init;
pub mod inject;
pub mod log;
pub mod status;
pub mod watch;

pub fn detect_local_ip() -> Option<String> {
    let output = std::process::Command::new("ipconfig")
        .args(["getifaddr", "en0"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let output = std::process::Command::new("ipconfig")
            .args(["getifaddr", "en1"])
            .output()
            .ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }
}
