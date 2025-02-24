pub mod block;
pub mod client;
pub mod container;
pub mod direction;
pub mod events;
pub mod logging;
pub mod player;
pub mod system;
pub mod vec3;

use crate::ListenerMap;
use mlua::{Lua, Table};
use std::{io, time::Duration};

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    CreateEnv(mlua::Error),
    EvalChunk(mlua::Error),
    ExecChunk(mlua::Error),
    LoadChunk(mlua::Error),
    MissingPath(mlua::Error),
    ReadFile(io::Error),
}

pub fn register_functions(
    lua: &Lua,
    globals: &Table,
    event_listeners: ListenerMap,
) -> mlua::Result<()> {
    globals.set(
        "sleep",
        lua.create_async_function(async |_, duration: u64| {
            tokio::time::sleep(Duration::from_millis(duration)).await;
            Ok(())
        })?,
    )?;

    block::register_functions(lua, globals)?;
    events::register_functions(lua, globals, event_listeners)?;
    logging::register_functions(lua, globals)?;
    system::register_functions(lua, globals)
}

pub fn reload(lua: &Lua, sender: Option<String>) -> Result<(), Error> {
    lua.load(
        &std::fs::read_to_string(
            lua.globals()
                .get::<String>("script_path")
                .map_err(Error::MissingPath)?,
        )
        .map_err(Error::ReadFile)?,
    )
    .set_environment(create_env(lua, sender)?)
    .exec()
    .map_err(Error::LoadChunk)
}

pub async fn eval(lua: &Lua, code: &str, sender: Option<String>) -> Result<String, Error> {
    lua.load(code)
        .set_environment(create_env(lua, sender)?)
        .eval_async::<String>()
        .await
        .map_err(Error::EvalChunk)
}

pub async fn exec(lua: &Lua, code: &str, sender: Option<String>) -> Result<(), Error> {
    lua.load(code)
        .set_environment(create_env(lua, sender)?)
        .exec_async()
        .await
        .map_err(Error::ExecChunk)
}

fn create_env(lua: &Lua, sender: Option<String>) -> Result<Table, Error> {
    let globals = lua.globals();
    globals.set("sender", sender).map_err(Error::CreateEnv)?;
    Ok(globals)
}
