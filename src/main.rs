#![feature(let_chains)]

mod arguments;
mod commands;
mod events;
mod scripting;

use azalea::{brigadier::prelude::CommandDispatcher, prelude::*};
use clap::Parser;
use commands::{CommandSource, register};
use events::handle_event;
use mlua::Lua;
use parking_lot::Mutex;
use std::{path::PathBuf, process::ExitCode, sync::Arc};

#[derive(Default, Clone, Component)]
pub struct State {
    lua: Arc<Mutex<Lua>>,
    commands: Arc<CommandDispatcher<Mutex<CommandSource>>>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = arguments::Arguments::parse();
    let lua = Lua::new();

    let config_path = args.script.unwrap_or(PathBuf::from("errornowatcher.lua"));
    if let Err(error) = match &std::fs::read_to_string(&config_path) {
        Ok(string) => lua.load(string).exec(),
        Err(error) => {
            eprintln!("failed to read {config_path:?}: {error:?}");
            return ExitCode::FAILURE;
        }
    } {
        eprintln!("failed to execute configuration as lua code: {error:?}");
        return ExitCode::FAILURE;
    }

    let globals = lua.globals();
    let Ok(server) = globals.get::<String>("SERVER") else {
        eprintln!("no server defined in lua globals!");
        return ExitCode::FAILURE;
    };
    let Ok(username) = globals.get::<String>("USERNAME") else {
        eprintln!("no username defined in lua globals!");
        return ExitCode::FAILURE;
    };

    if let Err(error) = globals.set("config_path", config_path) {
        eprintln!("failed to set config_path in lua globals: {error:?}");
        return ExitCode::FAILURE;
    };

    let Ok(server) = globals.get::<String>("Server") else {
        eprintln!("no server defined in lua globals!");
        return ExitCode::FAILURE;
    };
    let Ok(username) = globals.get::<String>("Username") else {
        eprintln!("no username defined in lua globals!");
        return ExitCode::FAILURE;
    };

    let account = if username.contains('@') {
        match Account::microsoft(&username).await {
            Ok(a) => a,
            Err(error) => {
                eprintln!("failed to login using microsoft account: {error:?}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        Account::offline(&username)
    };

    let mut commands = CommandDispatcher::new();
    register(&mut commands);

    let Err(error) = ClientBuilder::new()
        .set_handler(handle_event)
        .set_state(State {
            lua: Arc::new(Mutex::new(lua)),
            commands: Arc::new(commands),
        })
        .start(account, server.as_ref())
        .await;
    eprintln!("{error:?}");

    ExitCode::SUCCESS
}
