use crate::build_info;
use clap::Parser;
use std::path::PathBuf;

/// A Minecraft bot with Lua scripting support
#[derive(Parser)]
#[command(version = build_info::version_formatted())]
pub struct Arguments {
    /// Path to main Lua file
    #[arg(short, long)]
    pub script: Option<PathBuf>,

    /// Code to execute (after script)
    #[arg(short, long)]
    pub exec: Option<String>,
}
