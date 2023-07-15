use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Emulator for the POC-8 architecture
pub struct Cli {
    /// Specify RAM image to load
    #[arg(short, long, value_name = "FILE")]
    pub image: Option<PathBuf>,

    /// Enable debug mode
    #[arg(short, long)]
    debug: bool
}