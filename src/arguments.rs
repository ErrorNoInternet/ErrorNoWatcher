use clap::Parser;
use std::path::PathBuf;

/// A Minecraft utility bot
#[derive(Parser)]
pub struct Arguments {
    /// Path to main Lua file
    #[arg(short, long)]
    pub script: Option<PathBuf>,
}
