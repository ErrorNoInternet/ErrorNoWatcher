use std::time::Duration;

use mlua::{Error, Function, Lua, Result, Table};
use tokio::time::{sleep, timeout};

pub fn register_globals(lua: &Lua, globals: &Table) -> Result<()> {
    globals.set(
        "sleep",
        lua.create_async_function(async |_, duration: u64| {
            sleep(Duration::from_millis(duration)).await;
            Ok(())
        })?,
    )?;

    globals.set(
        "timeout",
        lua.create_async_function(async |_, (duration, function): (u64, Function)| {
            timeout(
                Duration::from_millis(duration),
                function.call_async::<()>(()),
            )
            .await
            .map_err(Error::external)
        })?,
    )?;

    Ok(())
}
