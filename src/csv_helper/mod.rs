//! A Rust module replacing the original `csv.c/h` logic using the `csv` crate (and optionally ASCII28).
//!
//! We show how to read CSV records via `csv::Reader` and how to write CSV records via `csv::Writer`.
//! If ASCII28 is encountered, we demonstrate an alternative approach, as the standard CSV crate
//! doesn't handle ASCII28 out of the box.

use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use std::io::{self, BufRead};

/// If your original code had a `FIELD_INFO` for tracking number of quotes, commas, etc.,
/// you might no longer need it in full. However, here's a sample struct if you still want
/// to track those stats for each field.
#[derive(Debug, Default)]
pub struct FieldInfo {
    pub num_commas: usize,
    pub num_quotes: usize,
}

/// If you want a context for parsing CSV lines that might contain ASCII28, define something like:
#[derive(Debug)]
pub struct CsvParseContext {
    /// Whether we detected ASCII28 in the line
    pub ascii28_present: bool,
    /// If needed, store stats about the fields (quotes, commas)
    pub fields_info: Vec<FieldInfo>,
}

impl CsvParseContext {
    pub fn new() -> Self {
        Self {
            ascii28_present: false,
            fields_info: Vec::new(),
        }
    }
}

/// Example: a function to parse a single line that may or may not have ASCII28.
/// If ASCII28 is present, we do a custom split. If not, we parse with the CSV crate.
pub fn parse_line(line: &str) -> Result<(Vec<String>, CsvParseContext)> {
    let mut ctx = CsvParseContext::new();

    // Check if ASCII28 is present
    if line.contains('\u{001C}') {
        // We have ASCII28. Let's do a manual split:
        ctx.ascii28_present = true;
        let mut fields = Vec::new();

        // Split on ASCII28
        for raw_field in line.split('\u{001C}') {
            // We'll do a minimal "unquote" approach or just store as-is
            // If you want advanced quotes logic with ASCII28, you'd replicate readAscii28Field
            let field_info = FieldInfo {
                num_quotes: raw_field.matches('"').count(),
                num_commas: raw_field.matches(',').count(),
            };
            ctx.fields_info.push(field_info);
            fields.push(raw_field.trim().to_string());
        }
        Ok((fields, ctx))
    } else {
        // No ASCII28. Let's parse as CSV using the csv crate
        // We'll build a small in-memory reader for just one line
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(line.as_bytes());

        // read one record
        let mut records_iter = rdr.records();
        if let Some(result) = records_iter.next() {
            let record = result.map_err(|e| anyhow!("CSV parse error: {}", e))?;
            let mut fields = Vec::with_capacity(record.len());
            for field in record.iter() {
                // Track basic comma/quote info if you want
                let field_info = FieldInfo {
                    num_quotes: field.matches('"').count(),
                    num_commas: field.matches(',').count(),
                };
                ctx.fields_info.push(field_info);
                fields.push(field.to_string());
            }
            Ok((fields, ctx))
        } else {
            // No records in line
            Ok((vec![], ctx))
        }
    }
}

/// If you want to parse a stream of lines (like the original parse logic), you can do:
pub fn parse_stream<R: BufRead>(reader: R) -> Result<Vec<Vec<String>>> {
    let mut lines = Vec::new();
    for line_res in reader.lines() {
        let line = line_res?;
        let (fields, _ctx) = parse_line(&line)?;
        // store fields, or do something with them
        lines.push(fields);
    }
    Ok(lines)
}

/// For writing CSV, we can rely on the `csv` crate's `Writer`.
///
/// This example just writes an array of strings as one CSV record.
pub fn write_csv_record<W: io::Write>(
    writer: &mut csv::Writer<W>,
    fields: &[String],
) -> Result<()> {
    writer.write_record(fields).map_err(|e| anyhow!(e))?;
    Ok(())
}

/// If you want to replicate the advanced quoting logic (like `writeField` in C),
/// the `csv` crate does this automatically. But if you want manual control, here's
/// an example that forcibly quotes fields if they contain quotes/commas:
pub fn write_escaped_field<W: io::Write>(writer: &mut csv::Writer<W>, field: &str) -> Result<()> {
    // The CSV crate automatically escapes quotes and commas if `quoting(true)` is enabled.
    // If you want to handle it manually, you'd do something like:
    let escaped = if field.contains('"') || field.contains(',') {
        // Insert quotes, double-quote internal quotes
        let mut buf = String::new();
        buf.push('"');
        for c in field.chars() {
            if c == '"' {
                buf.push_str("\"\"");
            } else {
                buf.push(c);
            }
        }
        buf.push('"');
        buf
    } else {
        field.to_string()
    };

    // Then write as a single record with one field, or accumulate yourself
    writer.write_record(&[escaped])?;
    Ok(())
}
