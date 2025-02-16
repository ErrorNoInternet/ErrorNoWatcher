use crate::{State, commands::CommandSource, http::serve, scripting};
use azalea::prelude::*;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use mlua::Function;
use log::{debug, error, info, trace};
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
                }
            }
        }
        Event::Init => {
            debug!("client initialized");

            globals.set(
                "client",
                scripting::client::Client {
                    inner: Some(client),
                },
            )?;
            if let Ok(on_init) = globals.get::<Function>("on_init")
                && let Err(error) = on_init.call::<()>(())
            {
                error!("failed to call lua on_init function: {error:?}");
            };

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
