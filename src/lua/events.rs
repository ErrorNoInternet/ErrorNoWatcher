use std::time::{SystemTime, UNIX_EPOCH};

use futures::executor::block_on;
use mlua::{Function, Lua, Result, Table};

use crate::ListenerMap;

pub fn register_globals(lua: &Lua, globals: &Table, event_listeners: ListenerMap) -> Result<()> {
    let m = event_listeners.clone();
    globals.set(
        "add_listener",
        lua.create_function(
            move |_, (event_type, callback, optional_id): (String, Function, Option<String>)| {
                let m = m.clone();
                let id = optional_id.unwrap_or_else(|| {
                    callback.info().name.unwrap_or_else(|| {
                        format!(
                            "anonymous @ {}",
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis()
                        )
                    })
                });
                tokio::spawn(async move {
                    m.write()
                        .await
                        .entry(event_type)
                        .or_default()
                        .push((id, callback));
                });
                Ok(())
            },
        )?,
    )?;

    let m = event_listeners.clone();
    globals.set(
        "remove_listeners",
        lua.create_function(move |_, (event_type, target_id): (String, String)| {
            let m = m.clone();
            tokio::spawn(async move {
                let mut m = m.write().await;
                let empty = m.get_mut(&event_type).is_some_and(|listeners| {
                    listeners.retain(|(id, _)| target_id != *id);
                    listeners.is_empty()
                });
                if empty {
                    m.remove(&event_type);
                }
            });
            Ok(())
        })?,
    )?;

    globals.set(
        "get_listeners",
        lua.create_function(move |lua, (): ()| {
            let m = block_on(event_listeners.read());
            let listeners_table = lua.create_table()?;
            for (event_type, callbacks) in m.iter() {
                let type_listeners_table = lua.create_table()?;
                for (id, callback) in callbacks {
                    let info = callback.info();
                    let table = lua.create_table()?;
                    table.set("name", info.name)?;
                    table.set("line_defined", info.line_defined)?;
                    table.set("source", info.source)?;
                    type_listeners_table.set(id.to_owned(), table)?;
                }
                listeners_table.set(event_type.to_owned(), type_listeners_table)?;
            }
            Ok(listeners_table)
        })?,
    )?;

    Ok(())
}
