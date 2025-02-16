use crate::{State, commands::CommandSource, http::serve, scripting};
use azalea::prelude::*;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{debug, error, info, trace};
use mlua::{Function, IntoLuaMulti, Table};
use tokio::net::TcpListener;

pub async fn handle_event(client: Client, event: Event, state: State) -> anyhow::Result<()> {
    let globals = state.lua.globals();

    match event {
        Event::Chat(message) => {
            let formatted_message = message.message();
            info!("{}", formatted_message.to_ansi());

            let owners = globals.get::<Vec<String>>("OWNERS")?;
            if message.is_whisper()
                && let (Some(sender), content) = message.split_sender_and_content()
                && owners.contains(&sender)
            {
                if let Err(error) = state.commands.execute(
                    content,
                    CommandSource {
                        client: client.clone(),
                        message: message.clone(),
                        state: state.clone(),
                    }
                    .into(),
                ) {
                    CommandSource {
                        client,
                        message,
                        state,
                    }
                    .reply(&format!("{error:?}"));
                }
            }

            call_lua_handler(&globals, "on_chat", ());
        }
        Event::Death(Some(packet)) => {
            let death_data = state.lua.create_table()?;
            death_data.set("message", packet.message.to_string())?;
            death_data.set("player_id", packet.player_id)?;

            call_lua_handler(&globals, "on_death", death_data);
        }
        Event::Tick => call_lua_handler(&globals, "on_tick", ()),
        Event::Login => call_lua_handler(&globals, "on_login", ()),
        Event::Init => {
            debug!("client initialized");

            globals.set(
                "client",
                scripting::client::Client {
                    inner: Some(client),
                },
            )?;
            call_lua_handler(&globals, "on_init", ());

            if let Some(address) = state.http_address {
                let listener = TcpListener::bind(address).await.map_err(|error| {
                    error!("failed to listen on {address}: {error:?}");
                    error
                })?;
                debug!("http server listening on {address}");

                loop {
                    let (stream, peer) = listener.accept().await?;
                    trace!("http server got connection from {peer}");

                    let conn_state = state.clone();
                    let service = service_fn(move |request| {
                        let request_state = conn_state.clone();
                        async move { serve(request, request_state).await }
                    });

                    tokio::task::spawn(async move {
                        if let Err(error) = http1::Builder::new()
                            .serve_connection(TokioIo::new(stream), service)
                            .await
                        {
                            error!("failed to serve connection: {error:?}");
                        }
                    });
                }
            }
        }
        _ => (),
    }

    Ok(())
}

fn call_lua_handler<T: IntoLuaMulti>(globals: &Table, name: &str, data: T) {
    if let Ok(handler) = globals.get::<Function>(name)
        && let Err(error) = handler.call::<()>(data)
    {
        error!("failed to call lua {name} function: {error:?}");
    }
}
