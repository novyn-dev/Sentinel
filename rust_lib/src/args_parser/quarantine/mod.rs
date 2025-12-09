use std::{env::home_dir, fs::{self, File, OpenOptions}, io::ErrorKind, os::unix::fs::PermissionsExt, path::{Path, PathBuf}};
use chrono::{DateTime, Local};
use rusqlite::{Connection, ffi::Error};

#[derive(Clone)]
pub struct QuarantinedFile {
    pub original_path: String,
    pub quarantine_path: String,
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

    pub fn from_db(conn: Connection) -> rusqlite::Result<Self> {
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
                    quarantine_path: row.get(2)?,
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
        let db_quarantined_files = self.get_quarantined().unwrap();

        for qf in self.quarantined_files.iter_mut() {
            let original_file_name = Path::new(&qf.original_path)
                .file_name()
                .expect("Invalid file path")
                .to_string_lossy();
            let quarantine_file_path = self.quarantine_dir.join(&*original_file_name);

            let quarantined_file_name = format!("{}_{}", original_file_name, Local::now().format("%Y%m%d%H%M%S"));
            // println!("{quarantined_file_name}");
            let full_quarantine_file_path = self.quarantine_dir.join(&quarantined_file_name);
            qf.quarantine_path = quarantine_file_path.to_string_lossy().to_string();

            let is_quarantined = db_quarantined_files
                .iter()
                .any(|db_qf| db_qf.quarantine_path.starts_with(&*quarantine_file_path.to_string_lossy()));
            // println!("is_quarantined: {is_quarantined}");
            if !is_quarantined {
                println!("Is not quarantined. Quarantine in process");
                // put file in /home/user/.sentinel_quarantine/ for quarantine
                // println!("{full_quarantine_file_path:?}");
                //
                // println!("About to create: {}", full_quarantine_file_path.display());
                // println!("Parent exists: {}", full_quarantine_file_path.parent().unwrap().exists());
                // println!("Dir perms: {:?}", fs::metadata(full_quarantine_file_path.parent().unwrap()));

                let file = match File::create(&full_quarantine_file_path) {
                    Ok(captured_file) => captured_file,
                    Err(e) => {
                        println!("CREATE FAILED: {e}");
                        return Err(format!("Creation failed for {:?}: {e}", full_quarantine_file_path));
                    }
                };

                let mut perm = file.metadata()
                    .unwrap_or_else(|e| panic!("Couldn't get metadata of {:?}\nError: {e}", file))
                    .permissions();

                println!("Locking {:?}", original_file_name);
                perm.set_mode(0o000); // lock it. even for the user, except root can change it soo yeah
                fs::set_permissions(&full_quarantine_file_path, perm)
                    .unwrap_or_else(|e| panic!("Couldn't set permissions to {:?}\nError {e}", &full_quarantine_file_path));

                println!("Quarantined {:?}", full_quarantine_file_path);
            } else {
                eprintln!("Already quarantined");
                continue;
            }
        }

        // after all that quaranting, store it to the db
        // for quarantined_file in &self.quarantined_files {
        //     if !is_quarantined {
        //         self.store_quarantined(quarantined_file).unwrap();
        //     } else {
        //         println!("Stored into quarantine db");
        //     }
        // }

        Ok(())
    }

    /// Pushing a quarantined file will immediately trigger the `quarantine()` function again
    pub fn push_quarantined(&mut self, quarantined: QuarantinedFile) -> Result<(), String> {
        // old files, just in case everything fails
        let old_files = self.quarantined_files.clone();
        self.quarantined_files.push(quarantined.clone());

        // quarantine again
        if let Err(e) = self.quarantine() {
            self.quarantined_files = old_files;
            return Err(format!("Couldn't quarantine the files\nError: {e}"));
        }

        self.store_quarantined(&quarantined.clone()).unwrap();
        Ok(())
    }

    fn store_quarantined(&self, quarantined: &QuarantinedFile) -> rusqlite::Result<()> {
        for qf in &self.quarantined_files {
            let original_file_name = Path::new(&qf.original_path)
                .file_name()
                .expect("Invalid file path")
                .to_string_lossy();

            let quarantined_file_name = format!("{}_{}", original_file_name, Local::now().format("%Y%m%d%H%M%S"));
            let full_quarantine_file_path = self.quarantine_dir.join(&quarantined_file_name);

            // (i, self.quarantine_dir.join(&quarantined_file_name))
            let quarantined_clone  = quarantined.clone();
            match &self.db {
                Some(db) => {
                    db.execute(
                    "INSERT INTO quarantined_files (original_path, quarantine_path, reason, quarantined_date)
                            VALUES ($1, $2, $3, $4)",
                    [
                        quarantined_clone.original_path,
                        full_quarantine_file_path.to_string_lossy().to_string(),
                        quarantined_clone.reason,
                        quarantined_clone.quarantined_date.unwrap_or_default().to_string()
                    ])?;
                }
                None => panic!("Couldn't load database! Is this database created or initialized?")
            }
        }
        Ok(())
    }

    fn get_quarantined(&self) -> rusqlite::Result<Vec<QuarantinedFile>> {
        let quarantined_files = if let Some(db) = &self.db {
            let mut stmt = db.prepare("SELECT id, original_path, quarantine_path, reason, quarantined_date FROM quarantined_files")?;
            stmt.query_map([], |row| {
                Ok(QuarantinedFile {
                    original_path: row.get(1)?,
                    quarantine_path: row.get(2)?,
                    reason: row.get(3)?,
                    quarantined_date: None,
                })
            })?
            .filter_map(|result| result.ok())
            .collect::<Vec<QuarantinedFile>>()
        } else {
            vec![]
        };

        Ok(quarantined_files)
    }
}
