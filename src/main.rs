#![feature(if_let_guard, let_chains)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::significant_drop_tightening)]

mod arguments;
mod build_info;
mod commands;
mod events;
mod hacks;
mod http;
mod lua;
mod particle;
mod replay;

#[cfg(feature = "matrix")]
mod matrix;

use std::{
    collections::HashMap,
    env,
    fs::{OpenOptions, read_to_string},
    sync::Arc,
};

use anyhow::Context;
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
use hacks::HacksPlugin;
use log::debug;
use mlua::{Function, Lua, Table};
use replay::{plugin::RecordPlugin, recorder::Recorder};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

type ListenerMap = Arc<RwLock<HashMap<String, Vec<(String, Function)>>>>;

#[derive(Default, Clone, Component)]
struct State {
    lua: Arc<Lua>,
    event_listeners: ListenerMap,
    commands: Arc<CommandDispatcher<Mutex<CommandSource>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "console-subscriber")]
    console_subscriber::init();

    let args = Arguments::parse();
    let event_listeners = Arc::new(RwLock::new(HashMap::new()));
    let lua = unsafe { Lua::unsafe_new() };
    let globals = lua.globals();
    lua::register_globals(&lua, &globals, event_listeners.clone())?;

    if let Some(path) = args.script {
        globals.set("SCRIPT_PATH", &*path)?;
        lua.load(read_to_string(path)?).exec()?;
    } else if let Some(code) = ["main.lua", "errornowatcher.lua"].iter().find_map(|path| {
        debug!("trying to load code from {path}");
        globals.set("SCRIPT_PATH", *path).ok()?;
        read_to_string(path).ok()
    }) {
        lua.load(code).exec()?;
    }
    if let Some(code) = args.exec {
        lua.load(code).exec()?;
    }

    let server = globals
        .get::<String>("Server")
        .context("lua globals missing Server variable")?;
    let username = globals
        .get::<String>("Username")
        .context("lua globals missing Username variable")?;

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
    let account = if username.contains('@') {
        Account::microsoft(&username).await?
    } else {
        Account::offline(&username)
    };

    let Err(error) = ClientBuilder::new_without_plugins()
        .add_plugins(DefaultBotPlugins)
        .add_plugins(HacksPlugin)
        .add_plugins(default_plugins)
        .add_plugins(record_plugin)
        .set_handler(events::handle_event)
        .set_state(State {
            lua: Arc::new(lua),
            event_listeners,
            commands: Arc::new(commands),
        })
        .start(account, server)
        .await;
    eprintln!("{error}");

    Ok(())
}
