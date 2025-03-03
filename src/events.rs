use crate::{
    State,
    commands::CommandSource,
    http::serve,
    lua::{self, direction::Direction, player::Player, vec3::Vec3},
    particle,
};
use azalea::{prelude::*, protocol::packets::game::ClientboundGamePacket};
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{debug, error, info, trace};
use mlua::IntoLuaMulti;
use std::process::exit;
use tokio::net::TcpListener;

#[allow(clippy::too_many_lines)]
pub async fn handle_event(client: Client, event: Event, state: State) -> anyhow::Result<()> {
    match event {
        Event::AddPlayer(player_info) => {
            call_listeners(&state, "add_player", Player::from(player_info)).await;
        }
        Event::Chat(message) => {
            let formatted_message = message.message();
            info!("{}", formatted_message.to_ansi());

            if message.is_whisper()
                && let (Some(sender), content) = message.split_sender_and_content()
                && state
                    .lua
                    .globals()
                    .get::<Vec<String>>("Owners")?
                    .contains(&sender)
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
            let table = state.lua.create_table()?;
            table.set("message", packet.message.to_string())?;
            table.set("player_id", packet.player_id.0)?;
            call_listeners(&state, "death", table).await;
        }
        Event::Disconnect(message) => {
            call_listeners(&state, "disconnect", message.map(|m| m.to_string())).await;
            exit(0)
        }
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
            _ => (),
        },
        Event::Init => {
            debug!("received initialize event");

            state.lua.globals().set(
                "client",
                lua::client::Client {
                    inner: Some(client),
                },
            )?;
            call_listeners(&state, "init", ()).await;

            let Some(address) = state.http_address else {
                return Ok(());
            };

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
        _ => (),
    }

    Ok(())
}

async fn call_listeners<T: Clone + IntoLuaMulti>(state: &State, event_type: &str, data: T) {
    if let Some(listeners) = state.event_listeners.read().await.get(event_type) {
        for (id, callback) in listeners {
            if let Err(error) = callback.call_async::<()>(data.clone()).await {
                error!("failed to call lua event listener {id} for {event_type}: {error:?}");
            }
        }
    }
}
