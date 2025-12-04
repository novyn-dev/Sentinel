use std::{io, panic, process};
use clap::Parser;
use colored::Colorize;
use rust_lib::args_parser::unauthorized_changes_scanner::UnauthorizedChangesScanner;
use rust_lib::args_parser::{file_scanner::FileScanner, Args};
use rust_lib::args_parser::Commands::{ScanDir, CheckUnauthorizedChanges};
use rusqlite::{Connection, Result};

fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
    "CREATE TABLE IF NOT EXISTS passwd_checks (
            id INTEGER PRIMARY KEY,
            timestamp TEXT NOT NULL,
            prev_hash TEXT NOT NULL,
            changed BOOLEAN NOT NULL
        )",
        []
    )?;
    Ok(())
}

fn main() -> io::Result<()> {
    panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location();
        if let Some(location) = location {
            eprintln!("{} {}\nat file {}\nat line {}", "[ERROR]".red().bold(), panic_info.payload_as_str().unwrap(), location.file(), location.line());
        } else {
            eprintln!("{} {}", "[ERROR]".red().bold(), panic_info.payload_as_str().unwrap());
        }
        process::exit(1);
    }));

    let args = Args::parse();

    let conn = Connection::open("/usr/local/share/sentinel/passwd.db").unwrap();
    init_db(&conn).expect("Couldn't initialize database");

    match args.clone().command {
        Some(ScanDir { .. }) => {
            let file_scanner = FileScanner::new(args.clone());
            file_scanner.scan_files().unwrap();
        }
        Some(CheckUnauthorizedChanges { .. }) => {
            let mut unauthorized_changes_scanner = UnauthorizedChangesScanner::from_db(conn);
            unauthorized_changes_scanner.scan_unauthorized_checks().unwrap();
        }
        None => {
            panic!("Please enter a command")
        }
    }

    Ok(())
}
