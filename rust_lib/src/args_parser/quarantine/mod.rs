use std::{env::home_dir, fs::{self, File}, io::ErrorKind, os::unix::fs::PermissionsExt, path::{Path, PathBuf}};
use chrono::{DateTime, Local};
use rusqlite::Connection;

#[derive(Clone)]
pub struct QuarantinedFile {
    pub original_path: String,
    pub quarantine_path: Option<String>,
    pub reason: String,
    pub quarantined_date: Option<DateTime<Local>>,
}

pub struct Quarantinizer {
    pub quarantine_dir: PathBuf,
    pub quarantined_files: Vec<QuarantinedFile>,

    db: Option<Connection>,
}

#[allow(clippy::new_without_default)]
impl Quarantinizer {
    pub fn new() -> Self {
        let quarantine_dir = home_dir()
            .map(|h| h.join(".sentinel_quarantine"))
            .ok_or("Couldn't determine home directory")
            .unwrap();
        fs::create_dir_all(&quarantine_dir)
            .unwrap_or_else(|e| println!("Couldn't create {}\nError {e}", quarantine_dir.to_string_lossy()));
        Self {
            quarantine_dir,
            quarantined_files: vec![],
            db: None,
        }
    }

    pub fn from_db(conn: Connection) -> rusqlite::Result<Self, rusqlite::Error> {
        let quarantine_dir = home_dir()
            .map(|h| h.join(".sentinel_quarantine"))
            .ok_or("Couldn't determine home directory")
            .unwrap();
        fs::create_dir_all(&quarantine_dir)
            .unwrap_or_else(|e| println!("Couldn't create {}\nError {e}", quarantine_dir.to_string_lossy()));

        let quarantined_files = {
            let mut stmt = conn.prepare("SELECT id, original_path, quarantine_path, reason, quarantined_date FROM quarantined_files")?;
            stmt.query_map([], |row| {
                Ok(QuarantinedFile {
                    original_path: row.get(1)?,
                    quarantine_path: row.get(2).ok(),
                    reason: row.get(3)?,
                    quarantined_date: None,
                })
            })?
            .filter_map(|result| result.ok())
            .collect::<Vec<QuarantinedFile>>()
        };

        Ok(Self {
            quarantine_dir,
            quarantined_files,
            db: Some(conn),
        })
    }

    pub fn quarantine(&mut self) -> Result<(), String> {
        for quarantined_file in &mut self.quarantined_files {
            let original_file_name = Path::new(&quarantined_file.original_path)
                .file_name()
                .expect("Invalid file path")
                .to_string_lossy();
            let quarantined_file_name = match quarantined_file.quarantined_date {
                Some(date) => &format!("{}_{:?}", original_file_name, date),
                None => &quarantined_file.original_path.to_string(),
            };

            let is_quarantined = quarantined_file.quarantine_path.is_some();
            if !is_quarantined {
                // put file in /home/user/.sentinel_quarantine/ for quarantine
                let full_quarantine_file_path = &self.quarantine_dir.join(quarantined_file_name);

                let file = match File::create_new(full_quarantine_file_path) {
                    Ok(captured_file) => captured_file,
                    Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
                    Err(e) => return Err(format!("Couldn't make file `{}`\nError: {e}", quarantined_file_name))
                };

                quarantined_file.quarantine_path = Some(full_quarantine_file_path.to_string_lossy().to_string());
                let mut perm = file.metadata()
                    .map_err(|e| format!("Couldn't get metadata of {:?}\nError: {e}", file))?
                    .permissions();

                println!("Locking {:?}", full_quarantine_file_path);
                perm.set_mode(0o000); // lock it. even for the user, except root can change it soo yeah
                fs::set_permissions(full_quarantine_file_path, perm)
                    .map_err(|e| format!("Couldn't set permissions to {:?}\nError {e}", full_quarantine_file_path))?;

                println!("Quarantined {:?}", full_quarantine_file_path);
            } else {
                eprintln!("Already quarantined");
            }
        }

        // after all that quaranting, store it to the db
        for quarantined_file in &self.quarantined_files {
            self.store_quarantined(quarantined_file).unwrap();
        }

        Ok(())
    }

    pub fn push_quarantined(&mut self, quarantined: QuarantinedFile) -> Result<(), String> {
        self.quarantined_files.push(quarantined);

        // quarantine again
        self.quarantine()?;
        Ok(())
    }

    fn store_quarantined(&self, quarantined: &QuarantinedFile) -> rusqlite::Result<()> {
        if let Some(db) = &self.db {
            let quarantined_clone  = quarantined.clone();
            db.execute(
                "INSERT INTO quarantined_files (original_path, quarantine_path, reason, quarantined_date)
                        VALUES ($1, $2, $3, $4)",
                [
                    quarantined_clone.original_path,
                    quarantined_clone.quarantine_path.unwrap(),
                    quarantined_clone.reason,
                    quarantined_clone.quarantined_date.unwrap_or_default().to_string()
                ]
            )?;
        }

        Ok(())
    }
}
