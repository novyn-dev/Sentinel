pub mod file_scanner;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::args_parser::file_scanner::FileCommands;

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
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
