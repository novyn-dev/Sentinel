use std::env::home_dir;
use std::path::PathBuf;
use std::time::Duration;
use std::{io, panic, process};
use chrono::Local;
use clap::Parser;
use colored::Colorize;
use rust_lib::args_parser::process_behaviors_analyzer::ProcessBehaviorsAnalyzer;
use rust_lib::args_parser::quarantine::{QuarantinedFile, Quarantinizer, ViewMode};
use rust_lib::args_parser::unauthorized_changes_scanner::UnauthorizedChangesScanner;
use rust_lib::args_parser::{file_scanner::FileScanner, Args};
use rust_lib::args_parser::Commands::{ScanDir, CheckUnauthorizedChanges, AnalyzeProcessBehaviors, Quarantine};
use rusqlite::{Connection, Result};

fn init_db_passwd(conn: &Connection) -> Result<()> {
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

fn init_db_quarantine(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS quarantined_files (
                id INTEGER PRIMARY KEY,
                original_path TEXT NOT NULL,
                quarantine_path TEXT NOT NULL,
                reason TEXT NOT NULL,
                quarantined_date TEXT NOT NULL
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

    let home_dir = home_dir().unwrap_or(PathBuf::from("/tmp"));

    let args = Args::parse();

    let conn_passwd = Connection::open("/usr/local/share/sentinel/passwd.db").unwrap();
    let conn_quarantine = Connection::open("/usr/local/share/sentinel/quarantined_files.db").unwrap();
    init_db_passwd(&conn_passwd).expect("Couldn't initialize database for passwd");
    init_db_quarantine(&conn_quarantine).expect("Couldn't initialize database for quarantine");

    match args.clone().command {
        Some(ScanDir { .. }) => {
            let file_scanner = FileScanner::new(args.clone());
            file_scanner.scan_files().unwrap();
        }
        Some(CheckUnauthorizedChanges { .. }) => {
            let mut unauthorized_changes_scanner = UnauthorizedChangesScanner::from_db(conn_passwd);

            loop {
                unauthorized_changes_scanner.scan_unauthorized_checks().unwrap();
                std::thread::sleep(Duration::from_secs(10));
            }
        }
        Some(AnalyzeProcessBehaviors) => {
            let mut process_behaviors_analyzer = ProcessBehaviorsAnalyzer::new();

            loop {
                std::thread::sleep(Duration::from_secs(1));
                process_behaviors_analyzer.analyze();
            }
        }
        Some(Quarantine { file, view, view_mode } ) => {
            let mut quarantinizer = Quarantinizer::from_db(conn_quarantine).unwrap();
            let quarantine_path = home_dir.join(".sentinel_quarantine");
            if view {
                let maybe_files = match view_mode {
                    ViewMode::Database => quarantinizer.get_quarantined(),
                    _ => todo!()
                };
                if let Ok(files) = maybe_files {
                    // specifically, quarantined file paths
                    let paths = files.iter().map(|f| f.quarantine_path.clone()).collect::<Vec<String>>();
                    println!("{:?}", paths);
                }
            } else {
                quarantinizer.push_quarantined(
                    QuarantinedFile {
                        original_path: file.unwrap().to_str().unwrap().to_string(),
                        quarantine_path: quarantine_path.to_string_lossy().to_string(),
                        reason: "No reason".to_string(),
                        quarantined_date: Some(Local::now()),
                    }
                ).unwrap();
            }
        }
        None => {
            panic!("Please enter a command")
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
