use crate::build_info;
use clap::Parser;
use std::path::PathBuf;

/// A Minecraft bot with Lua scripting support
#[derive(Parser)]
#[command(version = build_info::version_formatted())]
pub struct Arguments {
    /// Path to Lua entry point
    #[arg(short, long)]
    pub script: Option<PathBuf>,

    /// Code to execute after loading script
    #[arg(short, long)]
    pub exec: Option<String>,
}
