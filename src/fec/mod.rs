//! The main FEC module, containing context and parser submodules.
//! A module that replicates the core functionality of `encoding.c` in a Rust-idiomatic way.
//!
//! This includes:
//! 1. `LineInfo`: a struct holding ASCII28, ASCII-only, and UTF-8 validity flags.
//! 2. `collect_line_info()`: to detect line characteristics (length, ASCII28, etc.).
//! 3. `decode_line()`: to ensure the returned string is UTF-8, converting from ISO-8859-1 if needed.

pub mod context; // FecContext definition
pub mod parser; // Parsing logic

/// A struct containing metadata about a line, similar to the C `LINE_INFO`.
#[derive(Debug)]
pub struct LineInfo {
    /// Whether the line contains the ASCII 28 character (file separator).
    pub ascii28: bool,
    /// Whether the line is strictly ASCII (all bytes < 128).
    pub ascii_only: bool,
    /// Whether the line is valid UTF-8 in its current form.
    pub valid_utf8: bool,
    /// The number of bytes (characters) in the line (not necessarily Unicode scalar count).
    pub length: usize,
}

impl LineInfo {
    /// Create an empty `LineInfo`. This is usually only used internally.
    pub fn new() -> Self {
        Self {
            ascii28: false,
            ascii_only: true,
            valid_utf8: true,
            length: 0,
        }
    }
}

/// Examine a raw byte slice and return a `LineInfo` containing ASCII28 / ASCII-only / UTF-8 info.
///
/// - `data`: the raw line data (e.g., from a file or stream).
///
/// The check for UTF-8 uses standard Rust string decoding. If `from_utf8` fails,
/// we mark the line as invalid UTF-8.
pub fn collect_line_info(data: &[u8]) -> LineInfo {
    let mut info = LineInfo::new();

    // Check for ASCII28 and ASCII-only by iterating bytes
    for &byte in data {
        info.length += 1;
        if byte == 28 {
            info.ascii28 = true;
        }
        if byte > 127 {
            info.ascii_only = false;
        }
    }

    // Attempt to interpret as UTF-8
    if std::str::from_utf8(data).is_err() {
        // If it fails, mark as invalid
        info.valid_utf8 = false;
    }

    info
}

/// Convert a byte slice from ISO-8859-1 to UTF-8, returning a newly allocated `Vec<u8>`.
///
/// - `data`: the raw (ISO-8859-1) bytes to be converted.
///
/// This is a simplified approach:  
/// - Bytes < 128 remain the same  
/// - Bytes >= 128 become two-byte UTF-8 sequences: either `C2 xx` or `C3 xx`  
fn iso_8859_1_to_utf8(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len() * 2); // worst case: double in size
    for &byte in data {
        if byte < 128 {
            // If it's ASCII range, just push as-is
            output.push(byte);
        } else {
            // According to ISO-8859-1 => two-byte UTF-8:
            // 0xC2 or 0xC3 prefix, depending on whether >= 0xC0
            // (In the C code, it used 0xC2 + (byte > 0xBF).)
            let first = 0xC2 + ((byte > 0xBF) as u8);
            let second = (byte & 0x3F) + 0x80;
            output.push(first);
            output.push(second);
        }
    }
    output
}

/// Decode a line into a guaranteed UTF-8 `String`, returning `(decoded_string, LineInfo)`.
///
/// - If the line is already valid UTF-8, we just return a copy (or the same bytes).
/// - If it is invalid UTF-8, we assume **ISO-8859-1** and convert it to UTF-8.
///
/// # Arguments
/// - `data`: raw bytes of the line, e.g. read from a file or stdin.
///
/// # Returns
/// A tuple `(String, LineInfo)`, where `String` is the UTF-8 version of the line,
/// and `LineInfo` includes details about ASCII28, ASCII-only, validity, and length.
pub fn decode_line(data: &[u8]) -> (String, LineInfo) {
    // Step 1: Collect line info (ASCII28, ASCII-only, etc.)
    let info = collect_line_info(data);

    // Step 2: If valid UTF-8, just convert to String
    if info.valid_utf8 {
        // safe to unwrap because from_utf8 succeeded in collect_line_info check
        let s = String::from_utf8(data.to_vec()).unwrap();
        (s, info)
    } else {
        // Step 3: If invalid, treat as ISO-8859-1 => convert to UTF-8
        let converted = iso_8859_1_to_utf8(data);
        let s = String::from_utf8(converted)
            .unwrap_or_else(|_| "<invalid iso_8859_1 data>".to_string());
        (s, info)
    }
}
