pub mod file_scanner;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::args_parser::file_scanner::FileCommands;

// #[derive(Debug, Clone, Copy)]
// pub enum Platform {
//     Win32,
//     Linux,
// }
//
// impl std::str::FromStr for Platform {
//     type Err = String;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "windows" => Ok(Self::Win32),
//             "linux" => Ok(Self::Linux),
//             _ => Err(
//                 format!("Invalid platform: {s}.
//                     Use [Windows or Linux]"))
//         }
//     }
// }

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    // #[arg(short, long)]
    // platform: Platform,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    ScanFile {
        #[arg(short, long)]
        file: Option<PathBuf>,

        #[command(subcommand)]
        scan: Option<FileCommands>,
    },
}
