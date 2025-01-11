//! The library root for Fast-FEC Rust.
//!
//! This module re-exports key components, allowing them to be accessed from `main.rs`.

pub mod cli; // Command-line interface logic
pub mod csv_helper;
pub mod encoding; // Encoding-related utilities
pub mod errors; // Custom error types
pub mod fec; // FEC parsing logic
pub mod writer;

// Re-export anything you want to expose at the crate root
// e.g., pub use crate::fec::context::FecAppContext;
