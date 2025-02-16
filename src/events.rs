use crate::{State, commands::CommandSource, scripting};
use azalea::prelude::*;
use log::info;
use mlua::Function;

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
            globals.get::<Function>("Init")?.call::<()>(())?
        }
        _ => (),
    };

    Ok(())
}
