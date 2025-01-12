//! The main parsing logic, integrating `encoding` functionality (UTF-8 detection,
//! ISO-8859-1 conversion, ASCII28 detection, etc.).
//!
//! We read raw bytes from a `BufRead`, use `decode_line` to ensure they're valid UTF-8
//! (or convert from ISO-8859-1), then process them for version info, form types, etc.

use anyhow::{anyhow, Context, Result};
use csv::ReaderBuilder;
use std::io::BufRead;

// Bring in our FecContext for parse state
use crate::{encoding::decode_line, writer::WriterContext};

use super::context::FecContext;

/// Primary function to parse the FEC data stream.
///
/// - `ctx`: Tracks state (version, form type, etc.).
/// - `reader`: A buffered reader over the input data (file or STDIN).
/// - `writer`: Manages output operations.
///
/// Returns `Ok(())` on success or an error for unrecoverable issues.
pub fn parse_fec<R: BufRead>(
    ctx: &mut FecContext,
    reader: &mut R,
    writer: &mut WriterContext,
) -> Result<()> {
    let mut buffer = Vec::new();

    // ------------------------------------------------------------------
    // Step 1: Read and decode the "header" line
    // ------------------------------------------------------------------
    buffer.clear();
    let bytes_read = reader
        .read_until(b'\n', &mut buffer)
        .context("Failed to read the header line")?;
    if bytes_read == 0 {
        return Err(anyhow!("No data to parse."));
    }

    let (decoded_header, info_header) = decode_line(&buffer);
    ctx.use_ascii28 = info_header;
    parse_header(ctx, &decoded_header)?;

    // ------------------------------------------------------------------
    // Step 2: Main parse loop for all subsequent lines
    // ------------------------------------------------------------------
    loop {
        buffer.clear();
        let bytes_read = reader
            .read_until(b'\n', &mut buffer)
            .context("Failed to read a line from the input")?;
        if bytes_read == 0 {
            break; // EOF
        }

        let (decoded_line, info_line) = decode_line(&buffer);
        ctx.use_ascii28 = info_line;
        parse_line(ctx, &decoded_line, writer)?;
    }

    Ok(())
}

/// Parse a single non-header line.
///
/// - Handles F99 text blocks.
/// - Updates `ctx` based on parsed data.
/// - Writes output via `writer`.
pub fn parse_line(ctx: &mut FecContext, line: &str, writer: &mut WriterContext) -> Result<()> {
    let trimmed_line = line.trim();

    // Handle F99 text blocks
    if ctx.f99_text_start.is_match(trimmed_line) {
        if ctx.warn && !ctx.silent {
            eprintln!("(Warn) F99 text start encountered.");
        }

        // Continue parsing F99 text block until the end marker is found
        while !ctx.f99_text_end.is_match(trimmed_line) {
            // For now, this is a placeholder for handling multi-line F99 text.
            // You can integrate additional logic here.
        }
        return Ok(());
    }

    // Skip empty lines
    if trimmed_line.is_empty() {
        return Ok(());
    }

    // Parse fields based on the delimiter
    let fields = if ctx.use_ascii28 {
        parse_with_delimiter(trimmed_line, '\x1C')?
    } else {
        parse_csv_line(trimmed_line)?
    };

    // Process fields for specific information
    if fields.len() >= 2 && fields[1].to_lowercase().contains("version") {
        ctx.version = Some(fields[1].clone());
        ctx.version_length = fields[1].len();
        if !ctx.silent {
            eprintln!("Discovered version: {}", fields[1]);
        }
    }

    // Write fields to the output writer context
    writer
        .write_csv_record("output", &fields)
        .context("Failed to write fields to output")?;

    // Log warnings if enabled
    if ctx.warn && !ctx.silent {
        eprintln!("(Warn) parse_line => Found {} fields.", fields.len());
    }

    Ok(())
}

/// Parse a line using a custom delimiter (e.g., ASCII28).
///
/// - Splits the line into fields based on the delimiter.
pub fn parse_with_delimiter(line: &str, delimiter: char) -> Result<Vec<String>> {
    Ok(line.split(delimiter).map(|s| s.to_string()).collect())
}

/// Parse a CSV line using the `csv` crate.
///
/// - Uses the `csv` crate for robust handling of quoted fields, commas, etc.
fn parse_csv_line(line: &str) -> Result<Vec<String>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(line.as_bytes());

    let mut records = rdr.records();
    if let Some(record) = records.next() {
        Ok(record?.iter().map(|s| s.to_string()).collect())
    } else {
        Ok(vec![]) // No records in the line
    }
}

/// Parse the header line.
///
/// - Detects legacy headers or FEC-specific references.
/// - Updates `ctx` with relevant information.
fn parse_header(ctx: &mut FecContext, line: &str) -> Result<()> {
    let trimmed = line.trim();

    if trimmed.starts_with("/*") {
        if !ctx.silent {
            eprintln!("Detected a legacy header: {}", trimmed);
        }
        // Optionally parse additional lines for multi-line legacy headers
        return Ok(());
    }

    if trimmed.contains("FEC") && !ctx.silent {
        eprintln!("Detected a modern header referencing FEC: {}", trimmed);
    }

    Ok(())
}
