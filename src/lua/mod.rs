pub mod block;
pub mod client;
pub mod container;
pub mod direction;
pub mod events;
pub mod logging;
pub mod matrix;
pub mod nochatreports;
pub mod player;
pub mod system;
pub mod thread;
pub mod vec3;

use crate::{ListenerMap, build_info::built};
use mlua::{Lua, Table};
use std::io;

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

pub fn register_globals(
    lua: &Lua,
    globals: &Table,
    event_listeners: ListenerMap,
) -> mlua::Result<()> {
    globals.set("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"))?;
    globals.set("GIT_COMMIT_HASH", built::GIT_COMMIT_HASH)?;
    globals.set("GIT_COMMIT_HASH_SHORT", built::GIT_COMMIT_HASH_SHORT)?;

    block::register_globals(lua, globals)?;
    events::register_globals(lua, globals, event_listeners)?;
    logging::register_globals(lua, globals)?;
    nochatreports::register_globals(lua, globals)?;
    system::register_globals(lua, globals)?;
    thread::register_globals(lua, globals)
}

pub fn reload(lua: &Lua, sender: Option<String>) -> Result<(), Error> {
    lua.load(
        &std::fs::read_to_string(
            lua.globals()
                .get::<String>("SCRIPT_PATH")
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
