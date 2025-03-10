use crate::build_info;
use clap::Parser;
use std::path::PathBuf;

/// A Minecraft utility bot
#[derive(Parser)]
#[command(version = build_info::version_formatted())]
pub struct Arguments {
    /// Path to Lua entrypoint
    #[arg(short, long)]
    pub script: Option<PathBuf>,

    /// Code to execute after loading script
    #[arg(short, long)]
    pub exec: Option<String>,
}
