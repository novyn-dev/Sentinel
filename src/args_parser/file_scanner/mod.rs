use crate::args_parser::Commands::ScanFile;
use crate::args_parser::Args;
use clap::Subcommand;
use std::{env::home_dir, fs, io::{self, Read, Write}, path::PathBuf, process::Command, thread::sleep, time::Duration};
use statrs::statistics::Statistics;

#[derive(Debug, Clone, Copy)]
pub enum Aggressiveness {
    Chill,
    Cautious,
    Normal,
    Aggressive,
    Hardcore,
}

impl std::str::FromStr for Aggressiveness {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "0" | "chill" => Ok(Self::Chill),
            "1" | "cautious" => Ok(Self::Cautious),
            "2" | "normal" => Ok(Self::Normal),
            "3" | "aggressive" => Ok(Self::Aggressive),
            "4" | "hardcore" => Ok(Self::Hardcore),
            _ => Err(
                format!("Invalid aggressiveness: {s}.
                    Use 0-4 or [chill, cautious, normal, aggressive, hardcore]"))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Colorblindness {
    Protanopia
}

impl std::str::FromStr for Colorblindness {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "0" | "protanopia" => Ok(Self::Protanopia),
            _ => Err(
                format!("Invalid colorblindness: {s}.
                    Use 0-0 or [protanopia]"))
        }
    }
}

#[derive(Subcommand, Clone)]
pub enum FileCommands {
    Scan {
        #[arg(short, long)]
        response_aggressiveness: Aggressiveness,

        #[arg(short, long)]
        safety_aggressiveness: Aggressiveness,

        #[arg(short, long)]
        colorblindness: Option<Colorblindness>,
    }
}

pub struct FileScanner {
    args: Args,
    file: PathBuf,
    response_aggressiveness: Aggressiveness,
    safety_aggressiveness: Aggressiveness,
}

impl FileScanner {
    pub fn new(args: Args) -> Self {
        let commands = args.clone().command.unwrap();
        let file = match commands {
            ScanFile { file, .. } => file,
        }.unwrap_or(home_dir().expect("Couldn't get the home directory"));

        Self {
            args,
            file,
            response_aggressiveness: Aggressiveness::Normal,
            safety_aggressiveness: Aggressiveness::Normal,
        }
    }

    pub fn scan_files(&self) -> io::Result<()> {
        println!("Scanning directory: {:?}", &self.file);
        for entry in walkdir::WalkDir::new(&self.file).max_depth(3) {
            let entry = match entry {
                Ok(file) => file,
                Err(ref e) if e.io_error().is_some_and(|err| err.kind() == io::ErrorKind::PermissionDenied) => {
                    eprintln!("Permission denied when accessing {:?}", e.path());
                    continue;
                }
                Err(e) => {
                    eprintln!("An unexpected error occured: {e}");
                    continue;
                }
            };
            let file_path = entry.path();

            if !file_path.is_file() {
                continue;
            }

            if file_path.extension().and_then(|s| s.to_str()) == Some("exe") {
                println!("Found: {:?}", file_path);

                let _output = Command::new("python")
                    .args(["model/predict.py", "--model-path", "model/model.json", "--filepath", file_path.to_str().unwrap()])
                    .status()
                    .expect("Couldn't run the code");
            }
        }

        Ok(())
    }
}
