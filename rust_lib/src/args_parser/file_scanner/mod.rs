use crate::args_parser::Commands::ScanDir;
use crate::args_parser::Args;
use clap::Subcommand;
use std::ffi::CString;
use std::fs::File;
use std::os::raw::c_char;
use std::path::Path;
use std::{env::home_dir, io::{self, Read}, path::PathBuf};

#[link(name = "lief_wrapper")]
unsafe extern "C" {
    fn predict_malware_elf(filepath: *const c_char, model_path: *const c_char) -> bool;
    fn predict_malware_pe(filepath: *const c_char, model_path: *const c_char) -> bool;
}

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
            "protanopia" => Ok(Self::Protanopia),
            _ => Err(
                format!("Invalid colorblindness: {s}.
                    Use [Protanopia]"))
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
            ScanDir { dir: file, .. } => file,
            _ => panic!("How did you even get here..? Did you not use scan-dir")
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
        let mut malwares_count = 0;
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

            let c_file_path = CString::new(file_path.to_str().unwrap()).unwrap();
            match check_file_signature(file_path) {
                Some(FileSignature::Exe) => {
                    let c_model_path = CString::new("model/exe/model.ubj").unwrap();
                    let is_malware = unsafe {
                        predict_malware_pe(c_file_path.as_ptr(), c_model_path.as_ptr())
                    };
                    handle_malware(file_path, is_malware, &mut malwares_count);
                }
                Some(FileSignature::Elf) => {
                    let c_model_path = CString::new("model/elf/model.ubj").unwrap();
                    let is_malware = unsafe {
                        predict_malware_elf( c_file_path.as_ptr(), c_model_path.as_ptr())
                    };
                    handle_malware(file_path, is_malware, &mut malwares_count);
                }
                _ => continue
            }
        }

        println!("Scanning ended");
        println!("Found {malwares_count} possible malwares.");

        Ok(())
    }
}

pub enum FileSignature {
    Exe,
    Elf,
}

fn check_file_signature(file_path: &Path) -> Option<FileSignature> {
    let mut f = File::open(file_path).unwrap();
    let mut magic = [0u8; 4];
    let read_amount = f.read(&mut magic).unwrap();

    if read_amount == 0 {
        return None;
    }
    // ELF magic = 0x7F 'E' 'L' 'F'
    if magic == [0x7F, b'E', b'L', b'F'] {
        return Some(FileSignature::Elf);
    }

    // EXE magic = 'M' 'Z'
    if magic[0..2] == [b'M', b'Z'] {
        return Some(FileSignature::Exe);
    }

    None
}

fn handle_malware<T: AsRef<Path>>(file_path: T, is_malware: bool, malwares_count: &mut usize) {
    if is_malware {
        println!("{:?} is a malware", file_path.as_ref());
        *malwares_count += 1;
    }
}
