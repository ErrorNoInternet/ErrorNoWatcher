use log::{debug, error, info, trace, warn};
use mlua::{Lua, Result, Table};

pub fn register_globals(lua: &Lua, globals: &Table) -> Result<()> {
    globals.set(
        "error",
        lua.create_function(|_, message: String| {
            error!("{message}");
            Ok(())
        })?,
    )?;
    globals.set(
        "warn",
        lua.create_function(|_, message: String| {
            warn!("{message}");
            Ok(())
        })?,
    )?;
    globals.set(
        "info",
        lua.create_function(|_, message: String| {
            info!("{message}");
            Ok(())
        })?,
    )?;
    globals.set(
        "debug",
        lua.create_function(|_, message: String| {
            debug!("{message}");
            Ok(())
        })?,
    )?;
    globals.set(
        "trace",
        lua.create_function(|_, message: String| {
            trace!("{message}");
            Ok(())
        })?,
    )?;

    Ok(())
}
