//! Error types for the BrainBit SDK wrapper.

/// All errors that can occur within the BrainBit API.
#[derive(Debug, thiserror::Error)]
pub enum BrainBitError {
    /// The neurosdk2 shared library could not be loaded.
    #[error("NeuroSDK2 library not available: {reason}")]
    LibraryNotAvailable { reason: String },

    /// An SDK operation returned an error via `OpStatus`.
    #[error("SDK error (code {code}): {message}")]
    SdkError { code: u32, message: String },

    /// No devices were found during scanning.
    #[error("No BrainBit device found")]
    NoDeviceFound,

    /// The device is not connected.
    #[error("Device not connected")]
    NotConnected,

    /// Timed out waiting for an operation.
    #[error("Operation timed out")]
    Timeout,

    /// A feature or command is not supported by this device.
    #[error("Not supported: {0}")]
    NotSupported(String),

    /// A null pointer was returned where a valid pointer was expected.
    #[error("Null pointer returned from SDK")]
    NullPointer,
}
