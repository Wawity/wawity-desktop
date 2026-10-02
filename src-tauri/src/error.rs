use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[allow(dead_code)]
pub enum VpnError {
    #[error("Failed to parse subscription: {0}")]
    ParseError(String),
    
    #[error("Invalid subscription format: {0}")]
    InvalidFormat(String),
    
    #[error("Failed to generate configuration: {0}")]
    ConfigError(String),
    
    #[error("Process management error: {0}")]
    ProcessError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("JSON serialization error: {0}")]
    JsonError(String),
    
    #[error("VPN is already running")]
    AlreadyRunning,
    
    #[error("VPN is not running")]
    NotRunning,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Wintun driver error: {0}")]
    WintunError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for VpnError {
    fn from(err: std::io::Error) -> Self {
        VpnError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for VpnError {
    fn from(err: serde_json::Error) -> Self {
        VpnError::JsonError(err.to_string())
    }
}

impl From<url::ParseError> for VpnError {
    fn from(err: url::ParseError) -> Self {
        VpnError::ParseError(err.to_string())
    }
}

impl From<base64::DecodeError> for VpnError {
    fn from(err: base64::DecodeError) -> Self {
        VpnError::ParseError(err.to_string())
    }
}

impl From<reqwest::Error> for VpnError {
    fn from(err: reqwest::Error) -> Self {
        VpnError::NetworkError(err.to_string())
    }
}