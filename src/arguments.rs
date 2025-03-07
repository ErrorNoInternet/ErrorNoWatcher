use crate::build_info;
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};

/// A Minecraft utility bot
#[derive(Parser)]
#[command(version = build_info::version_formatted())]
pub struct Arguments {
    /// Path to main Lua file
    #[arg(short, long)]
    pub script: Option<PathBuf>,

    /// Socket address to bind HTTP server to
    #[arg(short = 'a', long)]
    pub http_address: Option<SocketAddr>,
}
