//! Defines the `FecContext` struct, mirroring the original `FEC_CONTEXT` from C.

use regex::Regex;

/// The primary context for managing FEC parsing state.
pub struct FecContext {
    pub f99_text_start: Regex,     // Regex for detecting F99 text start
    pub f99_text_end: Regex,       // Regex for detecting F99 text end
    pub version: Option<String>,   // Parsed version (if any)
    pub version_length: usize,     // Length of the version string
    pub silent: bool,              // Suppress output messages
    pub warn: bool,                // Show warning messages
    pub use_ascii28: bool,         // Whether to use ASCII28 delimiters
    pub summary: bool,             // Whether this is a summary parse
    pub form_type: Option<String>, // Current form type
    pub num_fields: usize,         // Number of fields in the form
    pub include_filing_id: bool,   // Include filing ID in CSV output
    pub fec_id: String,            // Filing ID or file name
}

impl FecContext {
    /// Create a new FecContext with the given configuration.
    pub fn new(fec_id: String, include_filing_id: bool, silent: bool, warn: bool) -> Self {
        Self {
            f99_text_start: Regex::new(r"(?i)^\s*\[BEGIN ?TEXT\]\s*$")
                .expect("Failed to compile F99 start regex"),
            f99_text_end: Regex::new(r"(?i)^\s*\[END ?TEXT\]\s*$")
                .expect("Failed to compile F99 end regex"),
            version: None,
            version_length: 0,
            silent,
            warn,
            use_ascii28: false,
            summary: false,
            form_type: None,
            num_fields: 0,
            include_filing_id,
            fec_id,
        }
    }
}
