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
use futures::lock::Mutex;
use mlua::Lua;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

#[derive(Default, Clone, Component)]
pub struct State {
    lua: Lua,
    http_address: Option<SocketAddr>,
    commands: Arc<CommandDispatcher<Mutex<CommandSource>>>,
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
    scripting::register_functions(&lua, &globals)?;

    let mut commands = CommandDispatcher::new();
    register(&mut commands);

    let Err(error) = ClientBuilder::new()
        .set_handler(handle_event)
        .set_state(State {
            lua,
            http_address: args.http_address,
            commands: Arc::new(commands),
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
