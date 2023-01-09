mod bot;
mod logging;

use azalea::pathfinder::BlockPosGoal;
use azalea::{prelude::*, BlockPos, ClientInformation};
use azalea_protocol::packets::game::serverbound_client_command_packet::{
    Action::PerformRespawn, ServerboundClientCommandPacket,
};
use azalea_protocol::packets::game::ClientboundGamePacket;
use azalea_protocol::ServerAddress;
use logging::LogMessageType::*;
use logging::{log_error, log_message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BotConfiguration {
    username: String,
    server_address: String,
    register_keyword: String,
    register_command: String,
    login_keyword: String,
    login_command: String,
    bot_owners: Vec<String>,
    whitelist: Vec<String>,
    alert_players: Vec<String>,
    alert_location: Vec<i32>,
    alert_radius: i32,
    alert_command: Vec<String>,
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
            whitelist: vec![],
            alert_players: vec![],
            alert_location: vec![0, 0],
            alert_radius: 100,
            alert_command: Vec::new(),
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

    loop {
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
                bot_configuration: bot_configuration.clone(),
                logged_in: Arc::new(Mutex::new(false)),
                tick_counter: Arc::new(Mutex::new(0)),
                alert_second_counter: Arc::new(Mutex::new(0)),
                followed_player: Arc::new(Mutex::new(None)),
                player_locations: Arc::new(Mutex::new(HashMap::new())),
                player_timestamps: Arc::new(Mutex::new(HashMap::new())),
                alert_players: Arc::new(Mutex::new(bot_configuration.clone().alert_players)),
                bot_status_players: Arc::new(Mutex::new(Vec::new())),
            },
            plugins: plugins![],
            handle,
        })
        .await
        {
            Ok(_) => (),
            Err(error) => log_message(Error, &format!("An error occurred: {}", error)),
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

#[derive(Eq, Hash, PartialEq, PartialOrd, Debug, Clone)]
pub struct Player {
    uuid: u128,
    entity_id: u32,
    username: String,
}

#[derive(Default, Debug, Clone)]
pub struct PositionTimeData {
    position: Vec<i32>,
    time: u64,
}

#[derive(Default, Debug, Clone)]
pub struct PlayerTimeData {
    join_time: u64,
    chat_message_time: u64,
    leave_time: u64,
}

#[derive(Default, Debug, Clone)]
pub struct State {
    bot_configuration: BotConfiguration,
    logged_in: Arc<Mutex<bool>>,
    tick_counter: Arc<Mutex<u8>>,
    alert_second_counter: Arc<Mutex<u8>>,
    followed_player: Arc<Mutex<Option<Player>>>,
    player_locations: Arc<Mutex<HashMap<Player, PositionTimeData>>>,
    player_timestamps: Arc<Mutex<HashMap<String, PlayerTimeData>>>,
    alert_players: Arc<Mutex<Vec<String>>>,
    bot_status_players: Arc<Mutex<Vec<String>>>,
}

async fn handle(client: Client, event: Event, mut state: State) -> anyhow::Result<()> {
    match event {
        Event::Login => {
            log_message(
                Bot,
                &"Successfully joined server, receiving initial data...".to_string(),
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
        Event::AddPlayer(player) => {
            let mut player_timestamps = state.player_timestamps.lock().unwrap().clone();
            let mut current_player = player_timestamps
                .get(&player.profile.name)
                .unwrap_or(&PlayerTimeData {
                    join_time: 0,
                    chat_message_time: 0,
                    leave_time: 0,
                })
                .to_owned();
            current_player.join_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            player_timestamps.insert(player.profile.name, current_player);
            *state.player_timestamps.lock().unwrap() = player_timestamps;
        }
        Event::RemovePlayer(player) => {
            let mut player_timestamps = state.player_timestamps.lock().unwrap().clone();
            let mut current_player = player_timestamps
                .get(&player.profile.name)
                .unwrap_or(&PlayerTimeData {
                    join_time: 0,
                    chat_message_time: 0,
                    leave_time: 0,
                })
                .to_owned();
            current_player.leave_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            player_timestamps.insert(player.profile.name, current_player);
            *state.player_timestamps.lock().unwrap() = player_timestamps;
        }
        Event::Tick => {
            if !*state.logged_in.lock().unwrap() {
                *state.logged_in.lock().unwrap() = true;
                log_message(
                    Bot,
                    &"ErrorNoWatcher has finished initializing!".to_string(),
                );
            }

            *state.tick_counter.lock().unwrap() += 1;
            if *state.tick_counter.lock().unwrap() >= 20 {
                *state.tick_counter.lock().unwrap() = 0;
                *state.alert_second_counter.lock().unwrap() += 1;

                let followed_player = state.followed_player.lock().unwrap().to_owned();
                if followed_player.is_some() {
                    let player_locations = state.player_locations.lock().unwrap().clone();
                    match player_locations.get(&followed_player.unwrap()) {
                        Some(position_time_data) => client.goto(BlockPosGoal {
                            pos: BlockPos {
                                x: position_time_data.position[0],
                                y: position_time_data.position[1],
                                z: position_time_data.position[2],
                            },
                        }),
                        None => *state.followed_player.lock().unwrap() = None,
                    }
                }
            }

            if *state.alert_second_counter.lock().unwrap() >= 5 {
                *state.alert_second_counter.lock().unwrap() = 0;

                let player_locations = state.player_locations.lock().unwrap().clone();
                for (player, position_time_data) in player_locations {
                    if ((state.bot_configuration.alert_location[0]
                        - state.bot_configuration.alert_radius)
                        ..(state.bot_configuration.alert_location[0]
                            + state.bot_configuration.alert_radius))
                        .contains(&position_time_data.position[0])
                        || ((state.bot_configuration.alert_location[1]
                            - state.bot_configuration.alert_radius)
                            ..(state.bot_configuration.alert_location[1]
                                + state.bot_configuration.alert_radius))
                            .contains(&position_time_data.position[2])
                    {
                        if !state.bot_configuration.whitelist.contains(&player.username) {
                            let alert_players = state.alert_players.lock().unwrap().clone();
                            for alert_player in alert_players {
                                log_error(
                                    client
                                        .send_command_packet(&format!(
                                            "msg {} {}",
                                            alert_player,
                                            format!(
                                                "{} is near our base at {} {} {}!",
                                                player.username,
                                                position_time_data.position[0],
                                                position_time_data.position[1],
                                                position_time_data.position[2],
                                            )
                                        ))
                                        .await,
                                );
                            }
                            let mut alert_command = state.bot_configuration.alert_command.to_vec();
                            for argument in alert_command.iter_mut() {
                                *argument = argument.replace("{player_name}", &player.username);
                                *argument = argument
                                    .replace("{x}", &position_time_data.position[0].to_string());
                                *argument = argument
                                    .replace("{y}", &position_time_data.position[1].to_string());
                                *argument = argument
                                    .replace("{z}", &position_time_data.position[2].to_string());
                            }
                            if alert_command.len() >= 1 {
                                log_message(Bot, &"Executing alert shell command...".to_string());
                                let command_name = alert_command[0].clone();
                                alert_command.remove(0);
                                log_error(
                                    std::process::Command::new(command_name)
                                        .args(alert_command)
                                        .stdin(std::process::Stdio::null())
                                        .stdout(std::process::Stdio::null())
                                        .stderr(std::process::Stdio::null())
                                        .spawn(),
                                );
                            }
                        }
                    }
                }
            }
        }
        Event::Packet(packet) => match packet.as_ref() {
            ClientboundGamePacket::MoveEntityPos(packet) => {
                let world = client.world.read();
                let entity = world.entity(packet.entity_id).unwrap();
                for (uuid, player) in client.players.read().iter() {
                    if uuid.as_u128() == entity.uuid.as_u128() {
                        let position = entity.pos();
                        let mut player_locations = state.player_locations.lock().unwrap().clone();
                        player_locations.insert(
                            Player {
                                uuid: uuid.as_u128(),
                                entity_id: entity.id,
                                username: player.profile.name.clone(),
                            },
                            PositionTimeData {
                                position: vec![
                                    position.x as i32,
                                    position.y as i32,
                                    position.z as i32,
                                ],
                                time: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            },
                        );
                        *state.player_locations.lock().unwrap() = player_locations;
                    }
                }
            }
            ClientboundGamePacket::SetHealth(packet) => {
                let bot_status_players: Vec<String> = state
                    .bot_status_players
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|item| item.to_owned())
                    .collect();
                for player in bot_status_players {
                    log_error(
                        client
                            .send_command_packet(&format!(
                                "msg {} {}",
                                player,
                                format!(
                                    "Health: {}/20, Food: {}/20, Saturation: {}/20",
                                    packet.health, packet.food, packet.saturation
                                )
                            ))
                            .await,
                    );
                }
            }
            _ => (),
        },
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
                                &bot::process_command(&command, &bot_owner, &client, &mut state)
                                    .await,
                            ))
                            .await,
                    );
                }

                let mut player_timestamps = state.player_timestamps.lock().unwrap().clone();
                let mut current_player = player_timestamps
                    .get(&message.username().unwrap_or("Someone".to_string()))
                    .unwrap_or(&PlayerTimeData {
                        join_time: 0,
                        chat_message_time: 0,
                        leave_time: 0,
                    })
                    .to_owned();
                current_player.chat_message_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                player_timestamps.insert(
                    message.username().unwrap_or("Someone".to_string()),
                    current_player,
                );
                *state.player_timestamps.lock().unwrap() = player_timestamps;
            }
        }
        _ => {}
    }

    Ok(())
}
