pub mod client;
pub mod entity;
pub mod logging;
pub mod position;

use mlua::Lua;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    EvalChunk(mlua::Error),
    ExecChunk(mlua::Error),
    LoadChunk(mlua::Error),
    MissingPath(mlua::Error),
    ReadFile(std::io::Error),
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

pub fn eval(lua: &Lua, code: &str) -> Result<String, Error> {
    lua.load(code).eval::<String>().map_err(Error::EvalChunk)
}

pub fn exec(lua: &Lua, code: &str) -> Result<(), Error> {
    lua.load(code).exec().map_err(Error::ExecChunk)
}
