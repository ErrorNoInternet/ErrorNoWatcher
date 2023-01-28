use crate::logging::{log_error, LogMessageType::*};
use crate::{log_message, MatrixConfiguration, State};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::{
    MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
};
use std::sync::Arc;

#[derive(Clone)]
struct MatrixState {
    bot_state: Arc<State>,
    matrix_configuration: MatrixConfiguration,
    display_name: String,
}

pub async fn login_and_sync(matrix_configuration: MatrixConfiguration, bot_state: Arc<State>) {
    log_message(Matrix, &"Matrix is enabled! Logging in...".to_string());
    let client_builder =
        matrix_sdk::Client::builder().homeserver_url(&matrix_configuration.homeserver_url);
    let client = match client_builder.build().await {
        Ok(client) => client,
        Err(error) => {
            log_message(MatrixError, &format!("Unable to build client: {}", error));
            return;
        }
    };
    match client
        .login_username(
            &matrix_configuration.username,
            &matrix_configuration.password,
        )
        .device_id("ERRORNOWATCHER")
        .initial_device_display_name("ErrorNoWatcher")
        .send()
        .await
    {
        Ok(_) => (),
        Err(error) => {
            log_message(MatrixError, &format!("Unable to login: {}", error));
            return;
        }
    };
    let response = match client.sync_once(SyncSettings::default()).await {
        Ok(response) => response,
        Err(error) => {
            log_message(MatrixError, &format!("Unable to synchronize: {}", error));
            return;
        }
    };
    let display_name = match client.account().get_display_name().await {
        Ok(display_name) => display_name.unwrap_or(matrix_configuration.username.to_owned()),
        Err(error) => {
            log_message(
                MatrixError,
                &format!("Unable to get display name: {}", error),
            );
            return;
        }
    };
    log_message(
        Matrix,
        &format!("Successfully logged in as {}!", display_name),
    );
    let matrix_state = MatrixState {
        bot_state,
        matrix_configuration: matrix_configuration.clone(),
        display_name,
    };
    client.add_event_handler_context(matrix_state);
    client.add_event_handler(room_message_handler);
    let settings = SyncSettings::default().token(response.next_batch);
    match client.sync(settings).await {
        Ok(_) => (),
        Err(error) => log_message(MatrixError, &format!("Unable to synchronize: {}", error)),
    };
}

async fn room_message_handler(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    state: Ctx<MatrixState>,
) {
    if let Room::Joined(room) = room {
        let MessageType::Text(text_content) = event.content.msgtype else {
            return;
        };

        if state
            .matrix_configuration
            .bot_owners
            .contains(&event.sender.to_string())
            && text_content.body.starts_with(&state.display_name)
        {
            let bot_state = &state.bot_state;
            let client = bot_state.client.lock().unwrap().to_owned();
            let mut client = match client {
                Some(client) => client,
                None => {
                    log_error(
                        room.send(
                            RoomMessageEventContent::text_plain(
                                "I am still joining the Minecraft server!",
                            ),
                            None,
                        )
                        .await,
                    );
                    return;
                }
            };
            log_error(
                room.send(
                    RoomMessageEventContent::text_plain(
                        &crate::bot::process_command(
                            &text_content
                                .body
                                .trim_start_matches(&state.display_name)
                                .trim_start_matches(":")
                                .trim()
                                .to_string(),
                            &event.sender.to_string(),
                            &mut client,
                            bot_state.clone(),
                        )
                        .await,
                    ),
                    None,
                )
                .await,
            );
        }
    }
}
