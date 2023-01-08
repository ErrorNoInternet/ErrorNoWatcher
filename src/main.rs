mod bot;
mod logging;

use azalea::pathfinder::BlockPosGoal;
use azalea::{prelude::*, BlockPos, ClientInformation};
use azalea_block::BlockState;
use azalea_protocol::packets::game::serverbound_client_command_packet::{
    Action::PerformRespawn, ServerboundClientCommandPacket,
};
use azalea_protocol::ServerAddress;
use logging::LogMessageType::*;
use logging::{log_error, log_message};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

static NON_SOLID_BLOCKS: &[BlockState] = &[
    BlockState::Air,
    BlockState::Lava__0,
    BlockState::Water__0,
    BlockState::Cobweb,
    BlockState::Grass,
    BlockState::Fern,
    BlockState::DeadBush,
];

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BotConfiguration {
    username: String,
    server_address: String,
    register_keyword: String,
    register_command: String,
    login_keyword: String,
    login_command: String,
    bot_owners: Vec<String>,
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
            bot_owners: vec![],
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
        state: State {
            bot_configuration,
            tick_counter: Arc::new(Mutex::new(0)),
            pathfind_tick_counter: Arc::new(Mutex::new(0)),
            final_target: Arc::new(Mutex::new(None)),
            current_target: Arc::new(Mutex::new(None)),
        },
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
    tick_counter: Arc<Mutex<u8>>,
    pathfind_tick_counter: Arc<Mutex<u8>>,
    final_target: Arc<Mutex<Option<Vec<i32>>>>,
    current_target: Arc<Mutex<Option<Vec<i32>>>>,
}

async fn handle(client: Client, event: Event, mut state: State) -> anyhow::Result<()> {
    match event {
        Event::Login => {
            log_message(
                Bot,
                &"ErrorNoWatcher has successfully joined the server".to_string(),
            );
            log_error(
                client
                    .set_client_information(ClientInformation {
                        view_distance: 2,
                        ..Default::default()
                    })
                    .await,
            );
        }
        Event::Death(_) => {
            log_message(
                Bot,
                &"Player has died! Automatically respawning...".to_string(),
            );
            client
                .write_packet(
                    ServerboundClientCommandPacket {
                        action: PerformRespawn,
                    }
                    .get(),
                )
                .await?
        }
        Event::Tick => {
            *state.tick_counter.lock().unwrap() += 1;
            *state.pathfind_tick_counter.lock().unwrap() += 1;

            if *state.tick_counter.lock().unwrap() >= 20 {
                *state.tick_counter.lock().unwrap() = 0;

                if state.current_target.lock().unwrap().is_some() {
                    let coordinates =
                        (*state.current_target.lock().unwrap().clone().unwrap()).to_vec();
                    println!("{:?}", coordinates);
                    client.goto(BlockPosGoal {
                        pos: BlockPos {
                            x: coordinates[0],
                            y: coordinates[1],
                            z: coordinates[2],
                        },
                    });
                }
            }

            if *state.pathfind_tick_counter.lock().unwrap() >= 10 {
                *state.pathfind_tick_counter.lock().unwrap() = 0;

                if state.final_target.lock().unwrap().is_some() {
                    let current_position = client.entity().pos().clone();
                    let target_position =
                        state.final_target.lock().unwrap().clone().unwrap().to_vec();
                    let mut new_position = Vec::new();

                    if (current_position.x as i32) < target_position[0] {
                        new_position.push(current_position.x as i32 + 2);
                    } else {
                        new_position.push(current_position.x as i32 - 2);
                    }
                    new_position.push(current_position.y as i32 + 2);
                    if (current_position.z as i32) < target_position[2] {
                        new_position.push(current_position.z as i32 + 2);
                    } else {
                        new_position.push(current_position.z as i32 - 2);
                    }

                    while NON_SOLID_BLOCKS.to_vec().contains(
                        &client
                            .world
                            .read()
                            .get_block_state(&BlockPos {
                                x: new_position[0],
                                y: new_position[1] - 1,
                                z: new_position[2],
                            })
                            .unwrap(),
                    ) {
                        new_position[1] -= 1;
                    }

                    while !NON_SOLID_BLOCKS.to_vec().contains(
                        &client
                            .world
                            .read()
                            .get_block_state(&BlockPos {
                                x: new_position[0],
                                y: new_position[1],
                                z: new_position[2],
                            })
                            .unwrap(),
                    ) {
                        if new_position[0] < target_position[0] {
                            new_position[0] += 1
                        } else {
                            new_position[0] -= 1
                        }
                    }

                    while !NON_SOLID_BLOCKS.to_vec().contains(
                        &client
                            .world
                            .read()
                            .get_block_state(&BlockPos {
                                x: new_position[0],
                                y: new_position[1],
                                z: new_position[2],
                            })
                            .unwrap(),
                    ) {
                        if new_position[2] < target_position[2] {
                            new_position[2] += 1
                        } else {
                            new_position[2] -= 1
                        }
                    }

                    while NON_SOLID_BLOCKS.to_vec().contains(
                        &client
                            .world
                            .read()
                            .get_block_state(&BlockPos {
                                x: new_position[0],
                                y: new_position[1] - 1,
                                z: new_position[2],
                            })
                            .unwrap(),
                    ) {
                        if new_position[0] < target_position[0] {
                            new_position[0] += 1
                        } else {
                            new_position[0] -= 1
                        }
                    }

                    while NON_SOLID_BLOCKS.to_vec().contains(
                        &client
                            .world
                            .read()
                            .get_block_state(&BlockPos {
                                x: new_position[0],
                                y: new_position[1] - 1,
                                z: new_position[2],
                            })
                            .unwrap(),
                    ) {
                        if new_position[2] < target_position[2] {
                            new_position[2] += 1
                        } else {
                            new_position[2] -= 1
                        }
                    }
                    *state.current_target.lock().unwrap() = Some(new_position);
                }
            }
        }
        Event::Chat(message) => {
            log_message(Chat, &message.message().to_ansi());

            if message.username().is_none() {
                if message
                    .content()
                    .contains(&state.bot_configuration.register_keyword)
                {
                    log_message(
                        Bot,
                        &"Detected register keyword! Registering...".to_string(),
                    );
                    log_error(
                        client
                            .send_command_packet(&state.bot_configuration.register_command)
                            .await,
                    )
                } else if message
                    .content()
                    .contains(&state.bot_configuration.login_keyword)
                {
                    log_message(Bot, &"Detected login keyword! Logging in...".to_string());
                    log_error(
                        client
                            .send_command_packet(&state.bot_configuration.login_command)
                            .await,
                    )
                }
                return Ok(());
            }

            for bot_owner in state.bot_configuration.bot_owners.clone() {
                if message
                    .message()
                    .to_string()
                    .starts_with(&format!("{} whispers to you: ", bot_owner))
                {
                    let command = message
                        .message()
                        .to_string()
                        .split("whispers to you: ")
                        .nth(1)
                        .unwrap_or("")
                        .to_string();
                    log_error(
                        client
                            .send_command_packet(&format!(
                                "msg {} Processing command...",
                                bot_owner
                            ))
                            .await,
                    );
                    log_error(
                        client
                            .send_command_packet(&format!(
                                "msg {} {}",
                                bot_owner,
                                &bot::process_command(&command, &client, &mut state),
                            ))
                            .await,
                    );
                }
            }
        }
        _ => {}
    }

    Ok(())
}
