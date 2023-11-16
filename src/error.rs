//! Error of JsyMk194 struct
use std::fmt;

/// Uart type of error
#[derive(Debug, Clone, PartialEq)]
pub enum UartErrorKind {
    Read,
    ReadInsuffisantBytes,
    BadCrc,
    Write,
    WriteInsuffisantBytes,
    Other,
}

/// Uart error with type and message
#[derive(Debug, Clone)]
pub struct UartError {
    pub message: String,
    pub kind: UartErrorKind
}

impl fmt::Display for UartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "Error when use UART interface. Reason: {}", self.message)
    }
}

impl UartError {
    pub fn new(kind: UartErrorKind, message: String) -> Self {
        Self {
            message,
            kind
        }
    }

    pub fn from(kind: UartErrorKind) -> Self {
        Self {
            message: String::new(),
            kind
        }
    }

    pub fn other(message: String) -> Self {
        Self {
            message,
            kind: UartErrorKind::Other
        }
    }
}

/// Error return by change bitrate
#[derive(Debug, Clone)]
pub struct ChangeBitrateError {
    pub parent: UartError
}

impl fmt::Display for ChangeBitrateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot change bitrate. Parent error: {}", self.parent)
    }
}

impl ChangeBitrateError {
    pub fn new(parent: UartError) -> Self {
        Self {
            parent
        }
    }
}
