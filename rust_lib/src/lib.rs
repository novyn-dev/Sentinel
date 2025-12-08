use rusqlite::{Connection, Result};

pub mod args_parser;

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use chrono::Local;
    use rusqlite::Connection;

    use crate::{args_parser::quarantine::{QuarantinedFile, Quarantinizer}, init_db_quarantine};

    #[test]
    fn check_quarantine_attrs() {
        let quarantinizer = Quarantinizer::new();
        assert_eq!(quarantinizer.quarantine_dir, PathBuf::from("/home/zai/.sentinel_quarantine/"));
    }

    #[test]
    fn quarantine_test() {
        let mut quarantinizer = Quarantinizer::new();
        let result = quarantinizer.quarantine();
        assert!(result.is_ok(), "An error happened");
    }

    #[test]
    fn push_quarantined_file_test() {
        let mut quarantinizer = Quarantinizer::new();
        let result = quarantinizer.push_quarantined(
            QuarantinedFile {
                original_path: "/home/zai/sigma.txt".to_string(),
                quarantine_path: "/home/zai/.sentinel_quarantine/".to_string(),
                reason: "No reason".to_string(),
                quarantined_date: Some(Local::now()),
            }
        );
        assert!(result.is_ok(), "Couldn't push a file for quarantine");

        let file = quarantinizer.quarantined_files.first()
            .expect("No quarantined files found");
        let path = file.quarantine_path.clone();
        assert!(path.starts_with("/home/zai/.sentinel_quarantine/"));

        let result = quarantinizer.quarantine();
        assert!(result.is_ok(), "Couldn't quarantine");
    }

    #[test]
    fn quarantine_store_db_test() {
        let conn = Connection::open("/usr/local/share/sentinel/dummy/quarantined_files.db").unwrap();
        init_db_quarantine(&conn).unwrap();

        let mut quarantinier = Quarantinizer::from_db(conn).unwrap();
        quarantinier.push_quarantined(
            QuarantinedFile {
                original_path: "/home/zai/shit.txt".to_string(),
                quarantine_path: "/home/zai/.sentinel_quarantine/".to_string(),
                reason: "No reason".to_string(),
                quarantined_date: Some(Local::now()),
            }
        ).unwrap();
    }

}
