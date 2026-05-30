use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum HotReloadError {
    #[error("Xcode project not found in {0}")]
    ProjectNotFound(String),

    #[error("No iOS scheme found in {0}")]
    NoSchemeFound(String),

    #[error("Failed to parse Xcode build setting: {0}")]
    BuildSettingParse(String),

    #[error("Configuration file not found at {0}")]
    ConfigNotFound(String),

    #[error("Swift compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Injection failed: {0}")]
    InjectionFailed(String),

    #[error("Connection refused by app at {0}:{1}")]
    ConnectionRefused(String, u16),

    #[error("TCP server error: {0}")]
    TcpError(String),

    #[error("Failed to read directory: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Generic(String),
}

