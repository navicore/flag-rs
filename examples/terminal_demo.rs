//! Simple demo to manually verify terminal features
//!
//! Run with different terminal widths:
//! ```bash
//! # Default width
//! cargo run --example terminal_demo
//!
//! # Narrow terminal
//! COLUMNS=50 cargo run --example terminal_demo
//!
//! # Wide terminal  
//! COLUMNS=100 cargo run --example terminal_demo
//! ```

use flag_rs::{CommandBuilder, Flag, FlagType, FlagValue};

fn main() {
    let app = CommandBuilder::new("demo")
        .short("Terminal feature demonstration")
        .long("This application demonstrates the terminal width detection and text wrapping features. The help text should automatically adjust to your terminal width, wrapping long lines at word boundaries while maintaining readability. Try running this with different COLUMNS values to see how it adapts.")
        .flag(
            Flag::new("file")
                .short('f')
                .usage("Input file path. This flag expects a path to a file that will be processed. The file must exist and be readable.")
                .value_type(FlagType::String)
        )
        .flag(
            Flag::new("output-format")
                .short('o')
                .usage("Output format for results. Supported formats include: json (machine-readable JSON format), yaml (human-friendly YAML format), table (formatted ASCII table), csv (comma-separated values for spreadsheet import)")
                .value_type(FlagType::String)
                .default(FlagValue::String("table".to_string()))
        )
        .subcommand(
            CommandBuilder::new("process")
                .short("Process data with various transformations and filters that can be applied in sequence")
                .build()
        )
        .build();

    // Always show help for this demo
    app.print_help();
}
