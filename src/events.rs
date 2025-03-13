use crate::{
    State,
    commands::CommandSource,
    http::serve,
    lua::{client, direction::Direction, player::Player, vec3::Vec3},
    particle,
    replay::recorder::Recorder,
};
use anyhow::{Context, Result};
use azalea::{
    brigadier::exceptions::BuiltInExceptions::DispatcherUnknownCommand, prelude::*,
    protocol::packets::game::ClientboundGamePacket,
};
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{debug, error, info, trace};
use mlua::{Error, Function, IntoLuaMulti, Table};
use ncr::utils::trim_header;
use std::{net::SocketAddr, process::exit};
use tokio::net::TcpListener;

#[allow(clippy::too_many_lines)]
pub async fn handle_event(client: Client, event: Event, state: State) -> Result<()> {
    match event {
        Event::AddPlayer(player_info) => {
            call_listeners(&state, "add_player", Player::from(player_info)).await;
        }
        Event::Chat(message) => {
            let globals = state.lua.globals();
            let (sender, mut content) = message.split_sender_and_content();
            let uuid = message.uuid().map(|uuid| uuid.to_string());
            let is_whisper = message.is_whisper();
            let text = message.message();
            let ansi_text = text.to_ansi();
            info!("{ansi_text}");

            let mut is_encrypted = false;
            if let Some(ref sender) = sender {
                let mut ncr_options = None;
                if let Ok(options) = globals.get::<Table>("NcrOptions")
                    && let Ok(decrypt) = globals.get::<Function>("ncr_decrypt")
                    && let Some(plaintext) = decrypt
                        .call::<String>((options.clone(), content.clone()))
                        .ok()
                        .as_deref()
                        .and_then(|s| trim_header(s).ok())
                {
                    is_encrypted = true;
                    ncr_options = Some(options);
                    plaintext.clone_into(&mut content);
                    info!("decrypted message from {sender}: {content}");
                }

                if is_whisper
                    && globals
                        .get::<Vec<String>>("Owners")
                        .unwrap_or_default()
                        .contains(sender)
                    && let Err(error) = state.commands.execute(
                        content.clone(),
                        CommandSource {
                            client: client.clone(),
                            message: message.clone(),
                            state: state.clone(),
                            ncr_options: ncr_options.clone(),
                        }
                        .into(),
                    )
                    && error.type_ != DispatcherUnknownCommand
                {
                    CommandSource {
                        client,
                        message,
                        state: state.clone(),
                        ncr_options,
                    }
                    .reply(&format!("{error:?}"));
                }
            }

            let table = state.lua.create_table()?;
            table.set("text", text.to_string())?;
            table.set("ansi_text", ansi_text)?;
            table.set("sender", sender)?;
            table.set("content", content)?;
            table.set("uuid", uuid)?;
            table.set("is_whisper", is_whisper)?;
            table.set("is_encrypted", is_encrypted)?;
            call_listeners(&state, "chat", table).await;
        }
        Event::Death(packet) => {
            if let Some(packet) = packet {
                let message_table = state.lua.create_table()?;
                message_table.set("text", packet.message.to_string())?;
                message_table.set("ansi_text", packet.message.to_ansi())?;
                let table = state.lua.create_table()?;
                table.set("message", message_table)?;
                table.set("player_id", packet.player_id.0)?;
                call_listeners(&state, "death", table).await;
            } else {
                call_listeners(&state, "death", ()).await;
            }
        }
        Event::Disconnect(message) => {
            if let Some(message) = message {
                let table = state.lua.create_table()?;
                table.set("text", message.to_string())?;
                table.set("ansi_text", message.to_ansi())?;
                call_listeners(&state, "disconnect", table).await;
            } else {
                call_listeners(&state, "disconnect", ()).await;
            }
        }
        Event::KeepAlive(id) => call_listeners(&state, "keep_alive", id).await,
        Event::Login => call_listeners(&state, "login", ()).await,
        Event::RemovePlayer(player_info) => {
            call_listeners(&state, "remove_player", Player::from(player_info)).await;
        }
        Event::Tick => call_listeners(&state, "tick", ()).await,
        Event::UpdatePlayer(player_info) => {
            call_listeners(&state, "update_player", Player::from(player_info)).await;
        }
        Event::Packet(packet) => match packet.as_ref() {
            ClientboundGamePacket::AddEntity(packet) => {
                let table = state.lua.create_table()?;
                table.set("id", packet.id.0)?;
                table.set("uuid", packet.uuid.to_string())?;
                table.set("kind", packet.entity_type.to_string())?;
                table.set("position", Vec3::from(packet.position))?;
                table.set(
                    "direction",
                    Direction {
                        y: f32::from(packet.y_rot) / (256.0 / 360.0),
                        x: f32::from(packet.x_rot) / (256.0 / 360.0),
                    },
                )?;
                table.set("data", packet.data)?;
                call_listeners(&state, "add_entity", table).await;
            }
            ClientboundGamePacket::LevelParticles(packet) => {
                let table = state.lua.create_table()?;
                table.set("position", Vec3::from(packet.pos))?;
                table.set("count", packet.count)?;
                table.set("kind", particle::to_kind(&packet.particle) as u8)?;
                call_listeners(&state, "level_particles", table).await;
            }
            ClientboundGamePacket::RemoveEntities(packet) => {
                call_listeners(
                    &state,
                    "remove_entities",
                    packet.entity_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                )
                .await;
            }
            ClientboundGamePacket::SetHealth(packet) => {
                let table = state.lua.create_table()?;
                table.set("food", packet.food)?;
                table.set("health", packet.health)?;
                table.set("saturation", packet.saturation)?;
                call_listeners(&state, "set_health", table).await;
            }
            ClientboundGamePacket::SetPassengers(packet) => {
                let table = state.lua.create_table()?;
                table.set("vehicle", packet.vehicle)?;
                table.set("passengers", &*packet.passengers)?;
                call_listeners(&state, "set_passengers", table).await;
            }
            ClientboundGamePacket::SetTime(packet) => {
                let table = state.lua.create_table()?;
                table.set("day_time", packet.day_time)?;
                table.set("game_time", packet.game_time)?;
                table.set("tick_day_time", packet.tick_day_time)?;
                call_listeners(&state, "set_time", table).await;
            }
            _ => (),
        },
        Event::Init => {
            debug!("received init event");

            let ecs = client.ecs.clone();
            ctrlc::set_handler(move || {
                ecs.lock()
                    .remove_resource::<Recorder>()
                    .map(Recorder::finish);
                exit(0);
            })?;

            let globals = state.lua.globals();
            lua_init(client, &state, &globals).await?;

            let Some(address): Option<SocketAddr> = globals
                .get::<String>("HttpAddress")
                .ok()
                .and_then(|string| string.parse().ok())
            else {
                return Ok(());
            };

            let listener = TcpListener::bind(address).await.inspect_err(|error| {
                error!("failed to listen on {address}: {error:?}");
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

    Ok(())
}

async fn lua_init(client: Client, state: &State, globals: &Table) -> Result<()> {
    let ecs = client.ecs.clone();
    globals.set(
        "finish_replay_recording",
        state.lua.create_function_mut(move |_, (): ()| {
            ecs.lock()
                .remove_resource::<Recorder>()
                .context("recording not active")
                .map_err(Error::external)?
                .finish()
                .map_err(Error::external)
        })?,
    )?;
    globals.set("client", client::Client(Some(client)))?;
    call_listeners(state, "init", ()).await;
    Ok(())
}

async fn call_listeners<T: Clone + IntoLuaMulti + Send + 'static>(
    state: &State,
    event_type: &'static str,
    data: T,
) {
    if let Some(listeners) = state.event_listeners.read().await.get(event_type).cloned() {
        for (id, callback) in listeners {
            let data = data.clone();
            tokio::spawn(async move {
                if let Err(error) = callback.call_async::<()>(data).await {
                    error!("failed to call lua event listener {id} for {event_type}: {error:?}");
                }
            });
        }
    }
}
