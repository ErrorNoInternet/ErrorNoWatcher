#![feature(let_chains)]

mod arguments;
mod commands;
mod events;
mod http;
mod lua;

use azalea::{brigadier::prelude::CommandDispatcher, prelude::*};
use clap::Parser;
use commands::{CommandSource, register};
use events::handle_event;
use futures::lock::Mutex;
use mlua::{Function, Lua};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};

const DEFAULT_SCRIPT_PATH: &str = "errornowatcher.lua";

type ListenerMap = HashMap<String, Vec<(String, Function)>>;

#[derive(Default, Clone, Component)]
pub struct State {
    lua: Lua,
    event_listeners: Arc<Mutex<ListenerMap>>,
    commands: Arc<CommandDispatcher<Mutex<CommandSource>>>,
    http_address: Option<SocketAddr>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = arguments::Arguments::parse();
    let script_path = args.script.unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH));

    let lua = Lua::new();
    lua.load(
        std::fs::read_to_string(&script_path)
            .expect(&(DEFAULT_SCRIPT_PATH.to_owned() + " should be in current directory")),
    )
    .exec()?;

    let globals = lua.globals();
    let server = globals
        .get::<String>("SERVER")
        .expect("SERVER should be in lua globals");
    let username = globals
        .get::<String>("USERNAME")
        .expect("USERNAME should be in lua globals");

    globals.set("script_path", script_path)?;
    lua::register_functions(&lua, &globals)?;

    let mut commands = CommandDispatcher::new();
    register(&mut commands);

    let Err(error) = ClientBuilder::new()
        .set_handler(handle_event)
        .set_state(State {
            lua,
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            commands: Arc::new(commands),
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
