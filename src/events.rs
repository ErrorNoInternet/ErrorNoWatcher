use std::process::exit;

use crate::{
    State,
    commands::CommandSource,
    http::serve,
    lua::{self, events::register_functions, player::Player},
};
use azalea::prelude::*;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{debug, error, info, trace};
use mlua::{Function, IntoLuaMulti};
use tokio::net::TcpListener;

pub async fn handle_event(client: Client, event: Event, state: State) -> anyhow::Result<()> {
    state.lua.gc_stop();
    let globals = state.lua.globals();

    match event {
        Event::AddPlayer(player_info) => {
            call_listeners(&state, "add_player", Player::from(player_info)).await;
        }
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
                        state: state.clone(),
                    }
                    .reply(&format!("{error:?}"));
                }
            }

            call_listeners(&state, "chat", formatted_message.to_string()).await;
        }
        Event::Death(Some(packet)) => {
            let death_data = state.lua.create_table()?;
            death_data.set("message", packet.message.to_string())?;
            death_data.set("player_id", packet.player_id.0)?;
            call_listeners(&state, "death", death_data).await;
        }
        Event::Disconnect(message) => {
            call_listeners(&state, "disconnect", message.map(|m| m.to_string())).await;
            exit(1)
        }
        Event::Login => call_listeners(&state, "login", ()).await,
        Event::RemovePlayer(player_info) => {
            call_listeners(&state, "remove_player", Player::from(player_info)).await;
        }
        Event::Tick => call_listeners(&state, "tick", ()).await,
        Event::UpdatePlayer(player_info) => {
            call_listeners(&state, "update_player", Player::from(player_info)).await;
        }
        Event::Init => {
            debug!("client initialized");

            globals.set(
                "client",
                lua::client::Client {
                    inner: Some(client),
                },
            )?;
            register_functions(&state.lua, &globals, state.clone()).await?;
            if let Ok(on_init) = globals.get::<Function>("on_init")
                && let Err(error) = on_init.call::<()>(())
            {
                error!("failed to call lua on_init function: {error:?}");
            }

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

                    tokio::spawn(async move {
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

async fn call_listeners<T: Clone + IntoLuaMulti>(state: &State, event_type: &str, data: T) {
    if let Some(listeners) = state.event_listeners.lock().await.get(event_type) {
        for (_, listener) in listeners {
            if let Err(error) = listener.call_async::<()>(data.clone()).await {
                error!("failed to call lua event listener for {event_type}: {error:?}");
            }
        }
    }
}
