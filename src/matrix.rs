use crate::log_message;
use crate::logging::LogMessageType::*;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::{
    MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
};

pub async fn login_and_sync(
    homeserver: String,
    username: String,
    password: String,
) -> anyhow::Result<()> {
    let mut client_builder = matrix_sdk::Client::builder().homeserver_url(homeserver);
    client_builder = client_builder.sled_store("matrix-store", None);
    let client = client_builder.build().await.unwrap();
    client
        .login_username(&username, &password)
        .initial_device_display_name("ErrorNoWatcher")
        .await?;
    log_message(Matrix, &format!("Logging in as {}...", username));
    let response = match client.sync_once(SyncSettings::default()).await {
        Ok(response) => response,
        Err(error) => return Err(error.into()),
    };
    client.add_event_handler(room_message_handler);
    let settings = SyncSettings::default().token(response.next_batch);
    client.sync(settings).await?;

    Ok(())
}

async fn room_message_handler(event: OriginalSyncRoomMessageEvent, room: Room) {
    if let Room::Joined(room) = room {
        let MessageType::Text(text_content) = event.content.msgtype else {
            return;
        };

        if text_content.body.contains("!party") {
            let content = RoomMessageEventContent::text_plain("🎉🎊🥳 let's PARTY!! 🥳🎊🎉");
            room.send(content, None).await.unwrap();
        }
    }
}
