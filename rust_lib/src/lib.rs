pub mod args_parser;

#[cfg(test)]
mod tests {
    use std::{fs, io::ErrorKind, path::PathBuf};
    use chrono::{DateTime, Local};

    use crate::args_parser::quarantine::{QuarantinedFile, Quarantinizer};

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
                quarantine_path: None,
                reason: "No reason".to_string(),
                quarantined_date: Some(Local::now()),
            }
        );
        assert!(result.is_ok(), "Couldn't push a file for quarantine");

        let file = quarantinizer.quarantined_files.first()
            .expect("No quarantined files found");
        let path = file.quarantine_path.as_ref()
            .expect("File has no quarantine path");
        assert!(path.starts_with("/home/zai/.sentinel_quarantine/"));

        let result = quarantinizer.quarantine();
        assert!(result.is_ok(), "Couldn't quarantine");
    }
}
