use std::{env::home_dir, fs::{self, File}, io::{self, ErrorKind}, path::{Path, PathBuf}};
use chrono::{DateTime, Local};

pub struct QuarantinedFile {
    pub original_path: String,
    pub quarantine_path: Option<String>,
    pub reason: String,
    pub quarantined_date: Option<DateTime<Local>>,
}

pub struct Quarantinizer {
    pub quarantine_dir: PathBuf,
    pub quarantined_files: Vec<QuarantinedFile>
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
        }
    }

    pub fn quarantine(&mut self) -> Result<(), String> {
        for file in &mut self.quarantined_files {
            let original_file_name = Path::new(&file.original_path)
                .file_name()
                .expect("Invalid file path")
                .to_string_lossy();
            let quarantined_file_name = match file.quarantined_date {
                Some(date) => &format!("{}_{:?}", original_file_name, date),
                None => &file.original_path.to_string(),
            };

            let quarantine_path = match &file.quarantine_path {
                Some(path) => path,
                None => quarantined_file_name,
            };

            if *quarantine_path != self.quarantine_dir {
                // put file in /home/user/.sentinel_quarantine/ for quarantine
                let full_quarantine_file_path = &self.quarantine_dir.join(quarantined_file_name);

                let file = match File::create_new(full_quarantine_file_path) {
                    Ok(captured_file) => {
                        file.quarantine_path = Some(full_quarantine_file_path.to_string_lossy().to_string());
                        captured_file
                    }
                    Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
                    Err(e) => {
                        return Err(format!("Couldn't make file `{}`\nError: {e}", quarantined_file_name));
                    }
                };
            }
        }

        Ok(())
    }

    pub fn push_quarantined(&mut self, quarantined: QuarantinedFile) -> Result<(), String> {
        self.quarantined_files.push(quarantined);
        Ok(())
    }
}
