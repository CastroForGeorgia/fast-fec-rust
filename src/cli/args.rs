//! Argument parsing logic for Fast-FEC Rust.
//!
//! Uses `clap` to parse command-line arguments and return a `CliConfig`.

use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use atty;

/// A struct representing parsed command-line arguments.
#[derive(Debug)]
pub struct CliConfig {
    pub fec_id: String,           // Filing ID or file path
    pub include_filing_id: bool,  // Whether to include a filing_id column
    pub silent: bool,             // Suppress output messages
    pub warn: bool,               // Show warning messages
    pub use_stdin: bool,          // Whether to read from STDIN
    pub show_usage: bool,         // Whether to show usage/help
    pub output_directory: String, // Directory for output files
    pub write_to_disk: bool,      // Whether to write output to disk
    pub buffer_size: usize,       // Buffer size for WriterContext
}

/// Parse command-line arguments and return a `CliConfig`.
pub fn parse_args() -> Result<CliConfig> {
     // Use Clap to define arguments and flags.
     let matches = Command::new("fast-fec-rust")
         .version("0.1.0")
         .about("Rust port of FastFEC with no persistent memory context")
         .arg(Arg::new("filing-id-or-file")
              .help("Filing ID or file path")
              .required(false)
              .index(1))
         .arg(Arg::new("include-filing-id")
              .long("include-filing-id")
              .short('f')
              .help("Include a filing_id column in the output CSV")
              .action(ArgAction::SetTrue))
         .arg(Arg::new("silent")
              .long("silent")
              .short('s')
              .help("Suppress output messages")
              .action(ArgAction::SetTrue))
         .arg(Arg::new("warn")
              .long("warn")
              .short('w')
              .help("Show warning messages")
              .action(ArgAction::SetTrue))
         .arg(Arg::new("disable-stdin")
              .long("disable-stdin")
              .help("Force reading from a file even if STDIN is piped")
              .action(ArgAction::SetTrue))
         .arg(Arg::new("usage")
              .long("usage")
              .help("Show usage information")
              .action(ArgAction::SetTrue))
         .arg(Arg::new("output-directory")
              .long("output-directory")
              .short('o')
              .help("Specify the directory for output files (default: 'output')")
              .default_value("output"))
         .arg(Arg::new("write-to-disk")
              .long("write-to-disk")
              .help("Write output to disk (default: true)")
              .action(ArgAction::SetTrue))
         .arg(Arg::new("buffer-size")
              .long("buffer-size")
              .help("Set the buffer size for WriterContext (default: 4096)")
              .default_value("4096"))
         .get_matches();
 
     // Parse values into a CliConfig struct.
     let fec_id = matches.get_one::<String>("filing-id-or-file")
         .cloned()
         .unwrap_or_else(|| "".to_string());
 
     let include_filing_id = matches.get_flag("include-filing-id");
     let silent = matches.get_flag("silent");
     let warn = matches.get_flag("warn");
     let disable_stdin = matches.get_flag("disable-stdin");
     let show_usage = matches.get_flag("usage");
     let output_directory = matches
         .get_one::<String>("output-directory")
         .cloned()
         .unwrap_or_else(|| "output".to_string());
     let write_to_disk = matches.get_flag("write-to-disk");
     let buffer_size = matches
         .get_one::<String>("buffer-size")
         .unwrap_or(&"4096".to_string())
         .parse::<usize>()
         .unwrap_or(4096);
 
     // Determine if STDIN is piped.
     let stdin_piped = !atty::is(atty::Stream::Stdin);
     let use_stdin = stdin_piped && !disable_stdin && fec_id.is_empty();
 
     // Return the configuration.
     Ok(CliConfig {
         fec_id: if use_stdin && fec_id.is_empty() { "STDIN_DATA".to_string() } else { fec_id },
         include_filing_id,
         silent,
         warn,
         use_stdin,
         show_usage,
         output_directory,
         write_to_disk,
         buffer_size,
     })
 }