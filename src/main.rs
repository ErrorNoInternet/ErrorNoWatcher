#![feature(let_chains)]

mod arguments;
mod commands;
mod events;
mod http;
mod scripting;

use azalea::{brigadier::prelude::CommandDispatcher, prelude::*};
use clap::Parser;
use commands::{CommandSource, register};
use events::handle_event;
use mlua::Lua;
use parking_lot::Mutex;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

#[derive(Default, Clone, Component)]
pub struct State {
    commands: Arc<CommandDispatcher<Mutex<CommandSource>>>,
    lua: Arc<Mutex<Lua>>,
    http_address: Option<SocketAddr>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = arguments::Arguments::parse();
    let script_path = args.script.unwrap_or(PathBuf::from("errornowatcher.lua"));

    let lua = Lua::new();
    lua.load(std::fs::read_to_string(&script_path)?).exec()?;

    let globals = lua.globals();
    let server = globals.get::<String>("SERVER")?;
    let username = globals.get::<String>("USERNAME")?;

    globals.set("script_path", script_path)?;
    scripting::logging::register(&lua, &globals)?;

    let mut commands = CommandDispatcher::new();
    register(&mut commands);

    let Err(error) = ClientBuilder::new()
        .set_handler(handle_event)
        .set_state(State {
            commands: Arc::new(commands),
            lua: Arc::new(Mutex::new(lua)),
            http_address: args.http_address,
        })
        .start(
            if username.contains('@') {
                Account::microsoft(&username).await?
            } else {
                Account::offline(&username)
            },
            server.as_ref(),
        )
        .await;
    eprintln!("{error:?}");

    Ok(())
}
