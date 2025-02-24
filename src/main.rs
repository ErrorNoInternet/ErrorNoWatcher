#![feature(let_chains)]

mod arguments;
mod commands;
mod events;
mod http;
mod lua;

use azalea::{
    DefaultBotPlugins, DefaultPlugins, brigadier::prelude::CommandDispatcher, prelude::*,
};
use bevy_app::PluginGroup;
use bevy_log::{
    LogPlugin,
    tracing_subscriber::{Layer, fmt::layer},
};
use clap::Parser;
use commands::{CommandSource, register};
use futures::lock::Mutex;
use futures_locks::RwLock;
use mlua::{Function, Lua};
use std::{
    collections::HashMap,
    env,
    fs::{OpenOptions, read_to_string},
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
};

const DEFAULT_SCRIPT_PATH: &str = "errornowatcher.lua";

type ListenerMap = Arc<RwLock<HashMap<String, Vec<(String, Function)>>>>;

#[derive(Default, Clone, Component)]
pub struct State {
    lua: Lua,
    event_listeners: ListenerMap,
    commands: Arc<CommandDispatcher<Mutex<CommandSource>>>,
    http_address: Option<SocketAddr>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = arguments::Arguments::parse();

    let script_path = args.script.unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH));
    let event_listeners = Arc::new(RwLock::new(HashMap::new()));

    let lua = Lua::new();
    let globals = lua.globals();
    globals.set("script_path", &*script_path)?;
    lua::register_functions(&lua, &globals, event_listeners.clone())?;
    lua.load(
        read_to_string(script_path)
            .expect(&(DEFAULT_SCRIPT_PATH.to_owned() + " should be in current directory")),
    )
    .exec()?;
    let server = globals
        .get::<String>("Server")
        .expect("Server should be in lua globals");
    let username = globals
        .get::<String>("Username")
        .expect("Username should be in lua globals");

    let mut commands = CommandDispatcher::new();
    register(&mut commands);

    let log_plugin = LogPlugin {
        custom_layer: |_| {
            env::var("LOG_FILE").ok().map(|log_file| {
                layer()
                    .with_writer(
                        OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(log_file)
                            .expect("should have been able to open log file"),
                    )
                    .boxed()
            })
        },
        ..Default::default()
    };
    let Err(error) = ClientBuilder::new_without_plugins()
        .add_plugins(DefaultPlugins.set(log_plugin))
        .add_plugins(DefaultBotPlugins)
        .set_handler(events::handle_event)
        .set_state(State {
            lua,
            event_listeners,
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
