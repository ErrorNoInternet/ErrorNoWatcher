use crate::State;
use futures::executor::block_on;
use mlua::{Function, Lua, Result, Table};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn register_functions(lua: &Lua, globals: &Table, state: State) -> Result<()> {
    let l = state.event_listeners.clone();
    globals.set(
        "add_listener",
        lua.create_function(
            move |_, (event_type, callback, id): (String, Function, Option<String>)| {
                let l = l.clone();
                tokio::spawn(async move {
                    l.lock().await.entry(event_type).or_default().push((
                        id.unwrap_or(callback.info().name.unwrap_or(format!(
                                "anonymous @ {}",
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis()
                            ))),
                        callback,
                    ));
                });
                Ok(())
            },
        )?,
    )?;

    let l = state.event_listeners.clone();
    globals.set(
        "remove_listener",
        lua.create_function(move |_, (event_type, target_id): (String, String)| {
            let l = l.clone();
            tokio::spawn(async move {
                let mut l = l.lock().await;
                let empty = if let Some(listeners) = l.get_mut(&event_type) {
                    listeners.retain(|(id, _)| target_id != *id);
                    listeners.is_empty()
                } else {
                    false
                };
                if empty {
                    l.remove(&event_type);
                }
            });
            Ok(())
        })?,
    )?;

    globals.set(
        "get_listeners",
        lua.create_function(move |lua, (): ()| {
            let l = block_on(state.event_listeners.lock());

            let listeners = lua.create_table()?;
            for (event_type, callbacks) in l.iter() {
                let type_listeners = lua.create_table()?;
                for (id, callback) in callbacks {
                    let listener = lua.create_table()?;
                    let i = callback.info();
                    if let Some(n) = i.name {
                        listener.set("name", n)?;
                    }
                    if let Some(l) = i.line_defined {
                        listener.set("line_defined", l)?;
                    }
                    if let Some(s) = i.source {
                        listener.set("source", s)?;
                    }
                    type_listeners.set(id.to_owned(), listener)?;
                }
                listeners.set(event_type.to_owned(), type_listeners)?;
            }

            Ok(listeners)
        })?,
    )?;

    Ok(())
}
