use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    GenerateFahrplan(GenerateFahrplanArgs),
}

#[derive(Args, Debug)]
/// Copy trains and delay them by given time
pub struct GenerateFahrplanArgs {
    /// Path to config file
    #[arg(short, long)]
    pub config: PathBuf,
}