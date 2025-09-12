use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    GenerateFahrplan(GenerateFahrplanArgs),

    #[command(subcommand)]
    Schedule(CliScheduleCommand),
}

#[derive(Subcommand)]
pub enum CliScheduleCommand {
    Apply(ApplyScheduleArgs),
    Generate(GenerateScheduleArgs),
}

/// Copy trains and delay them by given time
#[derive(Args, Debug)]
pub struct GenerateFahrplanArgs {
    /// Path to config file
    #[arg(short, long)]
    pub config: PathBuf,
}

/// Updates times in specified .trn files according to provided schedule file
#[derive(Args, Debug)]
pub struct ApplyScheduleArgs {
    /// Path to schedule file
    #[arg(short, long)]
    pub schedule: PathBuf,

    /// .trn files to modify
    #[arg(short, long, num_args = 1..)]
    pub trn_files: Vec<PathBuf>,
}

/// Generates a schedule file based on the given .trn file
#[derive(Args, Debug)]
pub struct GenerateScheduleArgs {
    /// Path to trn file
    #[arg(short, long)]
    pub trn: String,

    /// Path where to create the schedule file
    #[arg(short, long)]
    pub schedule: String,
}