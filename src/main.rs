#![feature(let_chains)]

mod arguments;
mod build_info;
mod commands;
mod events;
mod http;
mod lua;
mod particle;
mod replay;

use arguments::Arguments;
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
use mlua::{Function, Lua, Table};
use replay::{Recorder, plugin::RecordPlugin};
use std::{
    collections::HashMap,
    env,
    fs::{OpenOptions, read_to_string},
    path::PathBuf,
    sync::Arc,
};

const DEFAULT_SCRIPT_PATH: &str = "errornowatcher.lua";

type ListenerMap = Arc<RwLock<HashMap<String, Vec<(String, Function)>>>>;

#[derive(Default, Clone, Component)]
pub struct State {
    lua: Arc<Lua>,
    event_listeners: ListenerMap,
    commands: Arc<CommandDispatcher<Mutex<CommandSource>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "console-subscriber")]
    console_subscriber::init();

    let args = Arguments::parse();
    let script_path = args.script.unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH));
    let event_listeners = Arc::new(RwLock::new(HashMap::new()));
    let lua = unsafe { Lua::unsafe_new() };
    let globals = lua.globals();

    globals.set("script_path", &*script_path)?;
    lua::register_globals(&lua, &globals, event_listeners.clone())?;
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

    let default_plugins = if cfg!(feature = "console-subscriber") {
        DefaultPlugins.build().disable::<LogPlugin>()
    } else {
        DefaultPlugins.set(LogPlugin {
            custom_layer: |_| {
                env::var("LOG_FILE").ok().map(|path| {
                    layer()
                        .with_writer(
                            OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(&path)
                                .expect(&(path + " should be accessible")),
                        )
                        .boxed()
                })
            },
            ..Default::default()
        })
    };
    let record_plugin = RecordPlugin {
        recorder: Arc::new(parking_lot::Mutex::new(
            if let Ok(options) = globals.get::<Table>("ReplayRecordingOptions")
                && let Ok(path) = options.get::<String>("path")
            {
                Some(Recorder::new(
                    path,
                    server.clone(),
                    options
                        .get::<bool>("ignore_compression")
                        .unwrap_or_default(),
                )?)
            } else {
                None
            },
        )),
    };
    let Err(error) = ClientBuilder::new_without_plugins()
        .add_plugins(default_plugins)
        .add_plugins(record_plugin)
        .add_plugins(DefaultBotPlugins)
        .set_handler(events::handle_event)
        .set_state(State {
            lua: Arc::new(lua),
            event_listeners,
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
    eprintln!("{error}");

    Ok(())
}
