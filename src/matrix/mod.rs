mod bot;
mod verification;

use crate::{State, lua::matrix::client::Client as LuaClient};
use anyhow::Result;
use bot::{on_regular_room_message, on_stripped_state_member};
use matrix_sdk::{Client, config::SyncSettings};
use std::{fs, sync::Arc};
use verification::{on_device_key_verification_request, on_room_message_verification_request};

const COMMAND_PREFIX: &str = "ErrorNoWatcher";

#[derive(Clone)]
pub struct Context {
    state: State,
}

pub async fn login(
    state: State,
    homeserver_url: String,
    username: String,
    password: &str,
) -> Result<()> {
    let mut client = Client::builder().homeserver_url(homeserver_url);
    if let Some(db_path) = dirs::data_dir().map(|path| path.join("errornowatcher").join("matrix"))
        && fs::create_dir_all(&db_path).is_ok()
    {
        client = client.sqlite_store(db_path, None);
    }

    let client = Arc::new(client.build().await?);
    client
        .matrix_auth()
        .login_username(username, password)
        .device_id("ERRORNOWATCHER")
        .initial_device_display_name("ErrorNoWatcher")
        .await?;

    client.add_event_handler(on_stripped_state_member);
    let response = client.sync_once(SyncSettings::default()).await?;

    client.add_event_handler(on_device_key_verification_request);
    client.add_event_handler(on_room_message_verification_request);
    client.add_event_handler(on_regular_room_message);

    state
        .lua
        .globals()
        .set("matrix", LuaClient(client.clone()))?;

    client.add_event_handler_context(Context { state });
    client
        .sync(SyncSettings::default().token(response.next_batch))
        .await?;

    Ok(())
}
