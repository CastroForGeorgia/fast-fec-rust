//! Handles usage/help printing for Fast-FEC Rust.

/// Print usage information and exit the program with a status code of 1.
pub fn print_usage_and_exit() -> ! {
    eprintln!(
r#"Usage:
  fast-fec-rust [FLAGS] <FILING_ID_OR_FILE>

Flags:
  -f, --include-filing-id  Include a filing_id column in the output CSV
  -s, --silent             Suppress output messages
  -w, --warn               Show warning messages
      --disable-stdin      Disable piped STDIN usage
      --usage              Show usage information

Examples:
  fast-fec-rust 12345
  fast-fec-rust --include-filing-id 12345
  cat somefile.fec | fast-fec-rust --warn
"#
    );
    std::process::exit(1);
}
