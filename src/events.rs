use crate::{State, commands::CommandSource, http::handle, scripting};
use azalea::prelude::*;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{error, info};
use mlua::Function;
use tokio::net::TcpListener;

pub async fn handle_event(client: Client, event: Event, state: State) -> anyhow::Result<()> {
    let globals = state.lua.lock().globals();

    match event {
        Event::Chat(message) => {
            info!("{}", message.message().to_ansi());

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
                };
            }
        }
        Event::Init => {
            globals.set(
                "client",
                scripting::client::Client {
                    inner: Some(client),
                },
            )?;
            globals.get::<Function>("Init")?.call::<()>(())?;

            if let Some(address) = state.address {
                let listener = TcpListener::bind(address).await?;
                loop {
                    let (stream, _) = listener.accept().await?;
                    let io = TokioIo::new(stream);

                    let state = state.clone();
                    let service = service_fn(move |request| {
                        let state = state.clone();
                        async move { handle(request, state).await }
                    });

                    if let Err(error) = http1::Builder::new().serve_connection(io, service).await {
                        error!("failed to serve connection: {error:?}");
                    }
                }
            }
        }
        _ => (),
    };

    Ok(())
}
