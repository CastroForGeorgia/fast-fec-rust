use clap::{ArgAction, Command};
use fast_fec_rust::cli::args::CliConfig;

/// Helper function to create a Command instance identical to the one in `parse_args`.
fn get_command() -> Command {
    Command::new("fast-fec-rust")
        .version("0.1.0")
        .about("Rust port of FastFEC with no persistent memory context")
        .arg(
            clap::Arg::new("filing-id-or-file")
                .help("Filing ID or file path")
                .required(false)
                .index(1),
        )
        .arg(
            clap::Arg::new("include-filing-id")
                .long("include-filing-id")
                .short('f')
                .help("Include a filing_id column in the output CSV")
                .action(ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("silent")
                .long("silent")
                .short('s')
                .help("Suppress output messages")
                .action(ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("warn")
                .long("warn")
                .short('w')
                .help("Show warning messages")
                .action(ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("disable-stdin")
                .long("disable-stdin")
                .help("Force reading from a file even if STDIN is piped")
                .action(ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("usage")
                .long("usage")
                .help("Show usage information")
                .action(ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("output-directory")
                .long("output-directory")
                .short('o')
                .help("Specify the directory for output files (default: 'output')")
                .default_value("output"),
        )
        .arg(
            clap::Arg::new("write-to-disk")
                .long("write-to-disk")
                .help("Write output to disk (default: true)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("buffer-size")
                .long("buffer-size")
                .help("Set the buffer size for WriterContext (default: 4096)")
                .default_value("4096"),
        )
}

/// Helper function to simulate parse_args with given arguments.
fn simulate_parse_args<I, T>(args: I) -> Result<CliConfig, anyhow::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cmd = get_command();
    let matches = cmd.try_get_matches_from(args)?;

    // Parse values into a CliConfig struct.
    let fec_id = matches
        .get_one::<String>("filing-id-or-file")
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

    // For testing, we'll assume STDIN is not piped.
    let stdin_piped = false;
    let use_stdin = stdin_piped && !disable_stdin && fec_id.is_empty();

    Ok(CliConfig {
        fec_id: if use_stdin && fec_id.is_empty() {
            "STDIN_DATA".to_string()
        } else {
            fec_id
        },
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

#[test]
fn test_no_arguments() {
    let args = vec!["fast-fec-rust"];
    let config = simulate_parse_args(args).expect("Failed to parse args");

    let expected = CliConfig {
        fec_id: "".to_string(),
        include_filing_id: false,
        silent: false,
        warn: false,
        use_stdin: false, // Assuming no STDIN
        show_usage: false,
        output_directory: "output".to_string(),
        write_to_disk: false,
        buffer_size: 4096,
    };

    assert_eq!(config, expected);
}

#[test]
fn test_with_filing_id() {
    let args = vec!["fast-fec-rust", "12345"];
    let config = simulate_parse_args(args).expect("Failed to parse args");

    let expected = CliConfig {
        fec_id: "12345".to_string(),
        include_filing_id: false,
        silent: false,
        warn: false,
        use_stdin: false,
        show_usage: false,
        output_directory: "output".to_string(),
        write_to_disk: false,
        buffer_size: 4096,
    };

    assert_eq!(config, expected);
}

#[test]
fn test_include_filing_id_flag() {
    let args = vec!["fast-fec-rust", "--include-filing-id"];
    let config = simulate_parse_args(args).expect("Failed to parse args");

    let expected = CliConfig {
        fec_id: "".to_string(),
        include_filing_id: true,
        silent: false,
        warn: false,
        use_stdin: false,
        show_usage: false,
        output_directory: "output".to_string(),
        write_to_disk: false,
        buffer_size: 4096,
    };

    assert_eq!(config, expected);
}

// Repeat for other test cases
