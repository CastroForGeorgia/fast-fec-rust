//! Custom error types for Fast-FEC Rust, implemented using `thiserror`.

use thiserror::Error;
use std::io;

/// A general error type for the FEC parser.
#[derive(Debug, Error)]
pub enum FecError {
    /// For general parsing errors.
    #[error("Parsing error: {0}")]
    ParseError(String),

    /// For I/O errors (e.g., file not found).
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    // Add more error types as needed.
}
