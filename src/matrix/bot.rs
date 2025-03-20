use super::Context;
use crate::{
    events::call_listeners,
    lua::{eval, exec, matrix::room::Room as LuaRoom, reload},
};
use anyhow::Result;
use log::{debug, error};
use matrix_sdk::{
    Client, Room, RoomState,
    event_handler::Ctx,
    ruma::events::room::{
        member::StrippedRoomMemberEvent,
        message::{MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent},
    },
};
use std::time::Duration;
use tokio::time::sleep;

pub async fn on_regular_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    ctx: Ctx<Context>,
) -> Result<()> {
    if room.state() != RoomState::Joined {
        return Ok(());
    }
    let MessageType::Text(text_content) = event.content.msgtype else {
        return Ok(());
    };

    if ctx.is_owner(&event.sender.to_string()) && text_content.body.starts_with(&ctx.name) {
        let body = text_content.body[ctx.name.len()..]
            .trim_start_matches(':')
            .trim();
        let split = body.split_once(char::is_whitespace).unzip();
        let code = split
            .1
            .map(|body| body.trim_start_matches("```lua").trim_matches(['`', '\n']));

        let mut output = None;
        match split.0.unwrap_or(body).to_lowercase().as_str() {
            "reload" => {
                output = Some(
                    reload(&ctx.state.lua, None)
                        .map_or_else(|error| error.to_string(), |()| String::from("ok")),
                );
            }
            "eval" if let Some(code) = code => {
                output = Some(
                    eval(&ctx.state.lua, code, None)
                        .await
                        .unwrap_or_else(|error| error.to_string()),
                );
            }
            "exec" if let Some(code) = code => {
                output = Some(
                    exec(&ctx.state.lua, code, None)
                        .await
                        .map_or_else(|error| error.to_string(), |()| String::from("ok")),
                );
            }
            "ping" => {
                room.send(RoomMessageEventContent::text_plain("pong!"))
                    .await?;
            }
            _ => (),
        }

        if let Some(output) = output {
            room.send(RoomMessageEventContent::text_html(
                &output,
                format!("<pre><code>{output}</code></pre>"),
            ))
            .await?;
        }
    }

    call_listeners(&ctx.state, "matrix_chat", || {
        let table = ctx.state.lua.create_table()?;
        table.set("room", LuaRoom(room))?;
        table.set("sender_id", event.sender.to_string())?;
        table.set("body", text_content.body)?;
        Ok(table)
    })
    .await
}

pub async fn on_stripped_state_member(
    member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
    ctx: Ctx<Context>,
) -> Result<()> {
    if let Some(user_id) = client.user_id()
        && member.state_key == user_id
        && ctx.is_owner(&member.sender.to_string())
    {
        debug!("joining room {}", room.room_id());
        while let Err(error) = room.join().await {
            error!(
                "failed to join room {}: {error:?}, retrying...",
                room.room_id()
            );
            sleep(Duration::from_secs(10)).await;
        }
        debug!("successfully joined room {}", room.room_id());

        call_listeners(&ctx.state, "matrix_join_room", || {
            let table = ctx.state.lua.create_table()?;
            table.set("room", LuaRoom(room))?;
            table.set("sender", member.sender.to_string())?;
            Ok(table)
        })
        .await?;
    }

    Ok(())
}
