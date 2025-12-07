use std::{fs::File, io::Read};
use chrono::{DateTime, Local};
use rusqlite::{Connection, OptionalExtension, Result};
use sha2::{Digest, Sha256};

pub struct UnauthorizedChangesScanner {
    hash: Option<String>,
    prev_hash: Option<String>,
    last_checked: Option<DateTime<Local>>,
    changed: bool,

    db: Connection,
}

#[allow(clippy::new_without_default)]
impl UnauthorizedChangesScanner {
    pub fn new() -> Self {
        Self {
            hash: None,
            prev_hash: None,
            last_checked: None,
            changed: false,
            db: Connection::open_in_memory().unwrap(),
        }
    }

    pub fn from_db(conn: Connection) -> Self {
        let prev_hash: Option<String> = {
            let mut stmt = conn.prepare("SELECT prev_hash FROM passwd_checks ORDER BY rowid DESC LIMIT 1").unwrap();
            stmt.query_one([], |row| {
                row.get(0)
            }).optional().unwrap()
        };

        match prev_hash {
            Some(hash) => {
                Self {
                    hash: Some(hash.to_string()),
                    prev_hash: None,
                    last_checked: Some(Local::now()),
                    changed: false,
                    db: conn,
                }
            }
            None => {
                Self {
                    hash: None,
                    prev_hash: None,
                    last_checked: None,
                    changed: false,
                    db: conn,
                }
            }
        }

        // println!("{}", hash);
    }

    pub fn scan_unauthorized_checks(&mut self) -> std::io::Result<()> {
        self.check_for_changes("/etc/passwd")?;

        if self.changed {
            println!("Something changed in /etc/passwd");
        }

        // println!("{} {:?}", self.changed, self.hash.clone());
        self.store_check().unwrap();
        Ok(())
    }

    fn hash_passwd_file(&self, path: &str) -> std::io::Result<String> {
        let mut file = File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let hash = hex::encode(hasher.finalize());

        Ok(hash)
    }

    fn check_for_changes(&mut self, path: &str) -> std::io::Result<()> {
        let current_hash = self.hash_passwd_file(path)?;

        if let Some(prev_hash) = &self.hash {
            self.changed = current_hash != *prev_hash;
        }

        self.prev_hash = self.hash.clone();
        self.hash = Some(current_hash);
        self.last_checked = Some(Local::now());

        Ok(())
    }

    fn store_check(&self) -> Result<()> {
        self.db.execute(
        "INSERT INTO passwd_checks (timestamp, prev_hash, changed)
        VALUES ($1, $2, $3)", 
        [
                chrono::Local::now().to_rfc3339(),
                self.hash.clone().unwrap(),
                self.changed.to_string(),
            ]
        )?;
        Ok(())
    }
}
