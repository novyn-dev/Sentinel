use std::io;

use clap::Parser;
use sentinel::args_parser::{file_scanner::FileScanner, Args};

fn main() -> io::Result<()> {
    let args = Args::parse();
    // checks
    if args.command.is_none() {
        eprintln!("Error occured! please enter a command");
    }

    let file_scanner = FileScanner::new(args);
    file_scanner.scan_files().unwrap();

    Ok(())
}
