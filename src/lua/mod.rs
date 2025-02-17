pub mod block;
pub mod client;
pub mod container;
pub mod direction;
pub mod entity;
pub mod fluid_state;
pub mod hunger;
pub mod logging;
pub mod vec3;

use mlua::{Lua, Table};

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    EvalChunk(mlua::Error),
    ExecChunk(mlua::Error),
    LoadChunk(mlua::Error),
    MissingPath(mlua::Error),
    ReadFile(std::io::Error),
}

pub fn register_functions(lua: &Lua, globals: &Table) -> mlua::Result<()> {
    globals.set(
        "sleep",
        lua.create_async_function(async |_, duration: u64| {
            tokio::time::sleep(std::time::Duration::from_millis(duration)).await;
            Ok(())
        })?,
    )?;

    logging::register_functions(lua, globals)?;
    block::register_functions(lua, globals)
}

pub fn reload(lua: &Lua) -> Result<(), Error> {
    lua.load(
        &std::fs::read_to_string(
            lua.globals()
                .get::<String>("script_path")
                .map_err(Error::MissingPath)?,
        )
        .map_err(Error::ReadFile)?,
    )
    .exec()
    .map_err(Error::LoadChunk)
}

pub async fn eval(lua: &Lua, code: &str) -> Result<String, Error> {
    lua.load(code)
        .eval_async::<String>()
        .await
        .map_err(Error::EvalChunk)
}

pub async fn exec(lua: &Lua, code: &str) -> Result<(), Error> {
    lua.load(code).exec_async().await.map_err(Error::ExecChunk)
}
