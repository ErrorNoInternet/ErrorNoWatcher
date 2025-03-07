use log::error;
use mlua::{Lua, Result, Table};
use std::{
    ffi::OsString,
    process::{Command, Stdio},
    thread,
};

pub fn register_globals(lua: &Lua, globals: &Table) -> Result<()> {
    globals.set(
        "system",
        lua.create_function(|_, (command, args): (String, Option<Vec<OsString>>)| {
            thread::spawn(|| {
                if let Err(error) = Command::new(command)
                    .args(args.unwrap_or_default().iter())
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    error!("failed to run system command: {error:?}");
                }
            });
            Ok(())
        })?,
    )?;

    globals.set(
        "system_print_output",
        lua.create_function(|_, (command, args): (String, Option<Vec<OsString>>)| {
            thread::spawn(|| {
                if let Err(error) = Command::new(command)
                    .args(args.unwrap_or_default().iter())
                    .spawn()
                {
                    error!("failed to run system command: {error:?}");
                }
            });
            Ok(())
        })?,
    )?;

    globals.set(
        "system_with_output",
        lua.create_function(|lua, (command, args): (String, Option<Vec<OsString>>)| {
            Ok(
                match Command::new(command)
                    .args(args.unwrap_or_default().iter())
                    .output()
                {
                    Ok(output) => {
                        let table = lua.create_table()?;
                        table.set("status", output.status.code())?;
                        table.set("stdout", lua.create_string(output.stdout)?)?;
                        table.set("stderr", lua.create_string(output.stderr)?)?;
                        Some(table)
                    }
                    Err(error) => {
                        error!("failed to run system command: {error:?}");
                        None
                    }
                },
            )
        })?,
    )?;

    Ok(())
}
