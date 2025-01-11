//! Main entry point for the Fast-FEC Rust implementation.
//!
//! This file:
//! - Parses command-line arguments.
//! - Initializes the FecContext and WriterContext.
//! - Decides whether to read from a file or STDIN.
//! - Calls the FEC parser to process the input data.

use anyhow::Result;
use std::fs::File;
use std::io::{self, BufReader};

use fast_fec_rust::cli::args::parse_args;
use fast_fec_rust::cli::usage::print_usage_and_exit;
use fast_fec_rust::fec::context::FecContext;
use fast_fec_rust::fec::parser::parse_fec;
use fast_fec_rust::writer::WriterContext;

fn main() -> Result<()> {
    // Step 1: Parse command-line arguments.
    let cli_config = match parse_args() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error parsing arguments: {e}");
            print_usage_and_exit(); // Print usage if parsing fails
        }
    };

    // Step 2: Handle explicit usage request.
    if cli_config.show_usage {
        print_usage_and_exit();
    }

    // Step 3: Create the FecContext for managing state during parsing.
    let mut ctx = FecContext::new(
        cli_config.fec_id.clone(),
        cli_config.include_filing_id,
        cli_config.silent,
        cli_config.warn,
    );

    // Step 4: Initialize WriterContext for managing output.
    let mut writer_ctx = WriterContext::new(
        cli_config.output_directory.clone(),
        cli_config.fec_id.clone(),
        cli_config.write_to_disk,
        cli_config.buffer_size,
        None, // Optionally, pass a custom write function
        None, // Optionally, pass a custom line function
    );

    // Step 5: Determine input source: file or STDIN.
    let mut reader: Box<dyn io::BufRead> = if cli_config.use_stdin {
        if !cli_config.silent {
            eprintln!("Reading from STDIN for: {}", cli_config.fec_id);
        }
        Box::new(BufReader::new(io::stdin()))
    } else {
        if !cli_config.silent {
            eprintln!("Opening file: {}", cli_config.fec_id);
        }
        let file = File::open(&cli_config.fec_id)?;
        Box::new(BufReader::new(file))
    };

    // Step 6: Parse the FEC data.
    parse_fec(&mut ctx, &mut reader, &mut writer_ctx)?;

    // Step 7: Finalize WriterContext (flush all buffers).
    writer_ctx.flush_all()?;

    // Step 8: If parsing succeeds, print a success message (unless silent).
    if !cli_config.silent {
        println!("Done; parsing successful for: {}", cli_config.fec_id);
    }

    Ok(())
}
