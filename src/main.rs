mod logging;

use azalea_protocol::packets::game::serverbound_client_command_packet::{
    Action::PerformRespawn, ServerboundClientCommandPacket,
};
use logging::LogMessageType::*;
use logging::{log_error, log_message};

use azalea::prelude::*;
use azalea_protocol::ServerAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BotConfiguration {
    username: String,
    server_address: String,
    register_keyword: String,
    register_command: String,
    login_keyword: String,
    login_command: String,
}

impl Default for BotConfiguration {
    fn default() -> BotConfiguration {
        BotConfiguration {
            username: "ErrorNoWatcher".to_string(),
            server_address: "localhost".to_string(),
            register_keyword: "/register".to_string(),
            register_command: "register 1VerySafePassword!!! 1VerySafePassword!!!".to_string(),
            login_keyword: "/login".to_string(),
            login_command: "login 1VerySafePassword!!!".to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    let bot_configuration: BotConfiguration = match toml::from_str(
        &std::fs::read_to_string("bot_configuration.toml").unwrap_or_default(),
    ) {
        Ok(bot_configuration) => bot_configuration,
        Err(_) => {
            let default_configuration = BotConfiguration::default();
            match std::fs::write(
                "bot_configuration.toml",
                toml::to_string(&default_configuration).unwrap(),
            ) {
                Ok(_) => (),
                Err(error) => {
                    log_message(
                        Error,
                        &format!("Unable to save configuration file: {}", error),
                    );
                    return;
                }
            };
            default_configuration
        }
    };

    match azalea::start(azalea::Options {
        account: Account::offline(&bot_configuration.username),
        address: {
            let segments: Vec<String> = bot_configuration
                .server_address
                .split(":")
                .map(|item| item.to_string())
                .collect();
            if segments.len() == 1 {
                ServerAddress {
                    host: segments[0].clone(),
                    port: 25565,
                }
            } else if segments.len() == 2 {
                ServerAddress {
                    host: segments[0].clone(),
                    port: segments[1].clone().parse().unwrap_or(25565),
                }
            } else {
                log_message(
                    Error,
                    &"Unable to parse server address! Quitting...".to_string(),
                );
                return;
            }
        },
        state: State { bot_configuration },
        plugins: plugins![],
        handle,
    })
    .await
    {
        Ok(_) => (),
        Err(error) => log_message(Error, &format!("Unable to start ErrorNoWatcher: {}", error)),
    }
}

#[derive(Default, Clone)]
pub struct State {
    bot_configuration: BotConfiguration,
}

async fn handle(client: Client, event: Event, state: State) -> anyhow::Result<()> {
    match event {
        Event::Login => log_message(
            Bot,
            &"ErrorNoWatcher has successfully joined the server".to_string(),
        ),
        Event::Death(_) => {
            log_message(Bot, "Player has died! Automatically respawning...");
            client
                .write_packet(
                    ServerboundClientCommandPacket {
                        action: PerformRespawn,
                    }
                    .get(),
                )
                .await?
        }
        Event::Chat(message) => {
            let message_text = message.message().to_ansi();
            log_message(Chat, &message_text);

            if message.username().is_none() {
                if message_text.contains(&state.bot_configuration.register_keyword) {
                    log_message(
                        Bot,
                        &"Detected register keyword! Registering...".to_string(),
                    );
                    log_error(
                        client
                            .send_command_packet(&state.bot_configuration.register_command)
                            .await,
                    )
                } else if message_text.contains(&state.bot_configuration.login_keyword) {
                    log_message(Bot, &"Detected login keyword! Logging in...".to_string());
                    log_error(
                        client
                            .send_command_packet(&state.bot_configuration.login_command)
                            .await,
                    )
                }
            }
        }
        _ => {}
    }

    Ok(())
}
