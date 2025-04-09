use std::fmt;
use std::io;

/// Custom error types for the blockchain wallet CLI.
/// 
/// Provides detailed, context-rich error types for different categories
/// of errors that can occur in the application.
#[derive(Debug)]
pub enum WalletError {
    /// Wallet name already exists.
    WalletExists(String),

    /// Wallet with this name or address was not found.
    WalletNotFound(String),

    /// Invalid blockchain address format.
    AddressInvalid(String),
    
    /// Failed to read from wallet storage file.
    StorageRead { path: String, error: io::Error },

    /// Failed to write to wallet storage file.
    StorageWrite { path: String, error: io::Error },

    /// Failed to create storage directory or file.
    StorageCreate { path: String, error: io::Error },

    /// Failed to parse JSON from storage.
    JsonParse { error: serde_json::Error },

    /// Failed to serialize data to JSON.
    JsonSerialize { error: serde_json::Error },
    
    /// Failed to connect to blockchain service.
    ConnectionFailed { error: tonic::transport::Error },

    /// Error response from blockchain gRPC service.
    RpcError { status: Status },

    /// Transaction was rejected by the blockchain.
    TransactionFailed { message: String },

    /// Faucet request was rejected.
    FaucetFailed { message: String },
    
    /// Failed to decode hex-encoded key.
    KeyDecodingFailed { error: hex::FromHexError },

    /// Invalid private key format or content.
    InvalidPrivateKey { message: String },

    /// Failed to sign transaction with private key.
    SigningFailed { message: String },
    
    /// Error with system time operations.
    SystemTimeError { message: String },
}

/// Formats the error for display.
/// 
/// Provides a human-readable error message with context information
/// appropriate for the specific error type.
impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::WalletExists(name) => 
                write!(f, "Wallet '{}' already exists", name),
            WalletError::WalletNotFound(name) => 
                write!(f, "Wallet '{}' not found", name),
            WalletError::AddressInvalid(address) => 
                write!(f, "Invalid address: {}", address),
                
            WalletError::StorageRead { path, error } => 
                write!(f, "Failed to read from {}: {}", path, error),
            WalletError::StorageWrite { path, error } => 
                write!(f, "Failed to write to {}: {}", path, error),
            WalletError::StorageCreate { path, error } => 
                write!(f, "Failed to create {}: {}", path, error),
            WalletError::JsonParse { error } => 
                write!(f, "Failed to parse JSON: {}", error),
            WalletError::JsonSerialize { error } => 
                write!(f, "Failed to serialize to JSON: {}", error),
                
            WalletError::ConnectionFailed { error } => 
                write!(f, "Failed to connect to blockchain service: {}", error),
            WalletError::RpcError { status } => 
                write!(f, "RPC error: {}", status),
            WalletError::TransactionFailed { message } => 
                write!(f, "Transaction failed: {}", message),
            WalletError::FaucetFailed { message } => 
                write!(f, "Faucet request failed: {}", message),
                
            WalletError::KeyDecodingFailed { error } => 
                write!(f, "Failed to decode key: {}", error),
            WalletError::InvalidPrivateKey { message } => 
                write!(f, "Invalid private key: {}", message),
            WalletError::SigningFailed { message } => 
                write!(f, "Failed to sign transaction: {}", message),
                
            WalletError::SystemTimeError { message } => 
                write!(f, "System time error: {}", message),
        }
    }
}

impl std::error::Error for WalletError {}

/// Conversions from other error types to WalletError
impl From<io::Error> for WalletError {
    fn from(error: io::Error) -> Self {
        WalletError::StorageRead { 
            path: "unknown".to_string(), 
            error 
        }
    }
}

impl From<serde_json::Error> for WalletError {
    fn from(error: serde_json::Error) -> Self {
        WalletError::JsonParse { error }
    }
}

impl From<tonic::transport::Error> for WalletError {
    fn from(error: tonic::transport::Error) -> Self {
        WalletError::ConnectionFailed { error }
    }
}

impl From<Status> for WalletError {
    fn from(status: Status) -> Self {
        WalletError::RpcError { status }
    }
}

impl From<hex::FromHexError> for WalletError {
    fn from(error: hex::FromHexError) -> Self {
        WalletError::KeyDecodingFailed { error }
    }
}

/// Result type alias for the wallet application.
/// 
/// Simplifies error handling by using the custom WalletError type.
pub type Result<T> = std::result::Result<T, WalletError>;