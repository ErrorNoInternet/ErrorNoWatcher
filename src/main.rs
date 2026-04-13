#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::significant_drop_tightening)]

mod arguments;
mod build_info;
mod commands;
mod events;
mod http;
mod lua;
mod particle;

#[cfg(feature = "matrix")]
mod matrix;

#[cfg(feature = "replay")]
mod replay;

use std::{
    collections::HashMap,
    env,
    fs::{OpenOptions, read_to_string},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use arguments::Arguments;
use azalea::{
    DefaultPlugins, bot::DefaultBotPlugins, brigadier::prelude::CommandDispatcher, prelude::*,
};
use azalea_hax::HaxPlugin;
use bevy_app::PluginGroup;
use bevy_log::{
    LogPlugin,
    tracing_subscriber::{Layer, fmt::layer},
};
use clap::Parser;
use commands::{CommandSource, register};
use futures::lock::Mutex;
use futures_locks::RwLock;
use log::debug;
use mlua::{Function, Lua};
#[cfg(feature = "replay")]
use {
    mlua::Table,
    replay::{plugin::RecordPlugin, recorder::Recorder},
};

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
async fn main() -> Result<()> {
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
                    let file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&path)
                        .expect(&(path + " should be accessible"));
                    layer().with_writer(file).boxed()
                })
            },
            ..Default::default()
        })
    };

    let builder = ClientBuilder::new_without_plugins()
        .add_plugins(default_plugins)
        .add_plugins(DefaultBotPlugins)
        .add_plugins(HaxPlugin);

    #[cfg(feature = "replay")]
    let builder = builder.add_plugins(RecordPlugin {
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
    });

    let account = if username.contains('@') {
        Account::microsoft(&username).await?
    } else {
        Account::offline(&username)
    };

    if let AppExit::Error(code) = builder
        .set_handler(events::handle_event)
        .set_state(State {
            lua: Arc::new(lua),
            event_listeners,
            commands: Arc::new(commands),
        })
        .start(account, server)
        .await
    {
        bail!("azalea exited with code {code}")
    }

    Ok(())
}
