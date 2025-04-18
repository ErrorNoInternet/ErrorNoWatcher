mod bot;
mod verification;

use std::{path::Path, sync::Arc, time::Duration};

use anyhow::{Context as _, Result};
use bot::{on_regular_room_message, on_stripped_state_member};
use log::{error, warn};
use matrix_sdk::{
    Client, Error, LoopCtrl, authentication::matrix::MatrixSession, config::SyncSettings,
};
use mlua::Table;
use serde::{Deserialize, Serialize};
use tokio::fs;
use verification::{on_device_key_verification_request, on_room_message_verification_request};

use crate::{State, events::call_listeners, lua::matrix::client::Client as LuaClient};

#[derive(Clone)]
struct Context {
    state: State,
    name: String,
}

impl Context {
    fn is_owner(&self, name: &String) -> bool {
        self.state
            .lua
            .globals()
            .get::<Table>("MatrixOptions")
            .ok()
            .and_then(|options| {
                options
                    .get::<Vec<String>>("owners")
                    .ok()
                    .and_then(|owners| owners.contains(name).then_some(()))
            })
            .is_some()
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Session {
    #[serde(skip_serializing_if = "Option::is_none")]
    sync_token: Option<String>,
    user_session: MatrixSession,
}

async fn persist_sync_token(
    session_file: &Path,
    session: &mut Session,
    sync_token: String,
) -> Result<()> {
    session.sync_token = Some(sync_token);
    fs::write(session_file, serde_json::to_string(&session)?).await?;
    Ok(())
}

pub async fn login(state: &State, options: &Table, globals: &Table, name: String) -> Result<()> {
    let (homeserver_url, username, password, sync_timeout) = (
        options.get::<String>("homeserver_url")?,
        options.get::<String>("username")?,
        &options.get::<String>("password")?,
        options.get::<u64>("sync_timeout"),
    );
    let root_dir = dirs::data_dir()
        .context("no data directory")?
        .join("errornowatcher")
        .join(&name)
        .join("matrix");

    let mut builder = Client::builder().homeserver_url(homeserver_url);
    if !fs::try_exists(&root_dir).await.unwrap_or_default()
        && let Err(error) = fs::create_dir_all(&root_dir).await
    {
        warn!("failed to create directory for matrix sqlite store: {error:?}");
    } else {
        builder = builder.sqlite_store(&root_dir, None);
    }
    let client = builder.build().await?;

    let mut sync_settings = SyncSettings::new();
    if let Ok(seconds) = sync_timeout {
        sync_settings = sync_settings.timeout(Duration::from_secs(seconds));
    }

    let mut new_session;
    let session_file = root_dir.join("session.json");
    if let Some(session) = fs::read_to_string(&session_file)
        .await
        .ok()
        .and_then(|data| serde_json::from_str::<Session>(&data).ok())
    {
        new_session = session.clone();
        if let Some(sync_token) = session.sync_token {
            sync_settings = sync_settings.token(sync_token);
        }
        client.restore_session(session.user_session).await?;
    } else {
        let matrix_auth = client.matrix_auth();
        matrix_auth
            .login_username(username, password)
            .initial_device_display_name(&name)
            .await?;

        new_session = Session {
            user_session: matrix_auth.session().context("should have session")?,
            sync_token: None,
        };
        fs::write(&session_file, serde_json::to_string(&new_session)?).await?;
    }

    client.add_event_handler_context(Context {
        state: state.to_owned(),
        name,
    });
    client.add_event_handler(on_stripped_state_member);
    loop {
        match client.sync_once(sync_settings.clone()).await {
            Ok(response) => {
                sync_settings = sync_settings.token(response.next_batch.clone());
                persist_sync_token(&session_file, &mut new_session, response.next_batch).await?;
                break;
            }
            Err(error) => {
                error!("failed to do initial sync: {error:?}");
            }
        }
    }

    client.add_event_handler(on_device_key_verification_request);
    client.add_event_handler(on_room_message_verification_request);
    client.add_event_handler(on_regular_room_message);

    let client = Arc::new(client);
    globals.set("matrix", LuaClient(client.clone()))?;
    call_listeners(state, "matrix_init", || Ok(())).await?;

    client
        .sync_with_result_callback(sync_settings, |sync_result| async {
            let mut new_session = new_session.clone();
            persist_sync_token(&session_file, &mut new_session, sync_result?.next_batch)
                .await
                .map_err(|err| Error::UnknownError(err.into()))?;
            Ok(LoopCtrl::Continue)
        })
        .await?;
    Ok(())
}
