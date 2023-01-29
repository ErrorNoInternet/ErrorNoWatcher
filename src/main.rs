mod bot;
mod logging;
mod matrix;

use azalea::pathfinder::BlockPosGoal;
use azalea::{prelude::*, BlockPos, ClientInformation, Vec3};
use azalea_protocol::packets::game::serverbound_client_command_packet::{
    Action::PerformRespawn, ServerboundClientCommandPacket,
};
use azalea_protocol::packets::game::ClientboundGamePacket;
use azalea_protocol::ServerAddress;
use logging::LogMessageType::*;
use logging::{log_error, log_message};
use matrix::login_and_sync;
use rand::Rng;
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
    alert_radius: u32,
    alert_command: Vec<String>,
    alert_pause_time: u32,
    cleanup_interval: u32,
    mob_expiry_time: u64,
    mob_packet_drop_level: u8,
    matrix: MatrixConfiguration,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct MatrixConfiguration {
    enabled: bool,
    homeserver_url: String,
    username: String,
    password: String,
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
            whitelist: vec![],
            alert_players: vec![],
            alert_location: vec![0, 0],
            alert_radius: 100,
            alert_command: Vec::new(),
            alert_pause_time: 5,
            cleanup_interval: 300,
            mob_expiry_time: 300,
            mob_packet_drop_level: 5,
            matrix: MatrixConfiguration {
                enabled: false,
                homeserver_url: "https://matrix.example.com".to_string(),
                username: "errornowatcher".to_string(),
                password: "MyMatrixPassword".to_string(),
                bot_owners: vec!["@zenderking:envs.net".to_string()],
            },
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
            std::fs::copy("bot_configuration.toml", "bot_configuration.toml.bak")
                .unwrap_or_default();
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

    let original_state = State {
        client: Arc::new(Mutex::new(None)),
        bot_configuration: bot_configuration.clone(),
        whitelist: Arc::new(Mutex::new(bot_configuration.clone().whitelist)),
        bot_status: Arc::new(Mutex::new(BotStatus::default())),
        tick_counter: Arc::new(Mutex::new(0)),
        alert_second_counter: Arc::new(Mutex::new(0)),
        cleanup_second_counter: Arc::new(Mutex::new(0)),
        followed_player: Arc::new(Mutex::new(None)),
        looked_player: Arc::new(Mutex::new(None)),
        player_locations: Arc::new(Mutex::new(HashMap::new())),
        mob_locations: Arc::new(Mutex::new(HashMap::new())),
        player_timestamps: Arc::new(Mutex::new(HashMap::new())),
        alert_players: Arc::new(Mutex::new(bot_configuration.clone().alert_players)),
        alert_queue: Arc::new(Mutex::new(HashMap::new())),
        bot_status_players: Arc::new(Mutex::new(Vec::new())),
    };
    let state = Arc::new(original_state);

    let matrix_configuration = bot_configuration.matrix.to_owned();
    if matrix_configuration.enabled {
        log_message(Matrix, &"Matrix is enabled! Logging in...".to_string());
        tokio::spawn(login_and_sync(matrix_configuration, state.clone()));
    }

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
                        host: segments[0].to_owned(),
                        port: 25565,
                    }
                } else if segments.len() == 2 {
                    ServerAddress {
                        host: segments[0].to_owned(),
                        port: segments[1].to_owned().parse().unwrap_or(25565),
                    }
                } else {
                    log_message(
                        Error,
                        &"Unable to parse server address! Quitting...".to_string(),
                    );
                    return;
                }
            },
            state: state.clone(),
            plugins: plugins![],
            handle,
        })
        .await
        {
            Ok(_) => (),
            Err(error) => log_message(Error, &format!("An error occurred: {}", error)),
        }
        log_message(
            Bot,
            &"ErrorNoWatcher has lost connection, reconnecting in 5 seconds...".to_string(),
        );
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

#[derive(Eq, Hash, PartialEq, PartialOrd, Default, Debug, Clone)]
pub struct Player {
    uuid: String,
    entity_id: u32,
    username: String,
}

#[derive(Eq, Hash, PartialEq, PartialOrd, Default, Debug, Clone)]
pub struct Entity {
    id: u32,
    uuid: String,
    entity_type: String,
}

#[derive(Default, Debug, Clone)]
pub struct PositionTimeData {
    position: Vec<i32>,
    time: u64,
}

#[derive(Eq, Hash, PartialEq, PartialOrd, Default, Debug, Clone)]
pub struct PlayerTimeData {
    join_time: u64,
    chat_message_time: u64,
    leave_time: u64,
}

#[derive(Default, Debug, Clone)]
pub struct BotStatus {
    health: f32,
    food: u32,
    saturation: f32,
}

#[derive(Clone)]
pub struct State {
    client: Arc<Mutex<Option<azalea::Client>>>,
    bot_configuration: BotConfiguration,
    whitelist: Arc<Mutex<Vec<String>>>,
    bot_status: Arc<Mutex<BotStatus>>,
    tick_counter: Arc<Mutex<u8>>,
    alert_second_counter: Arc<Mutex<u16>>,
    cleanup_second_counter: Arc<Mutex<u16>>,
    followed_player: Arc<Mutex<Option<Player>>>,
    looked_player: Arc<Mutex<Option<Player>>>,
    player_locations: Arc<Mutex<HashMap<Player, PositionTimeData>>>,
    mob_locations: Arc<Mutex<HashMap<Entity, PositionTimeData>>>,
    player_timestamps: Arc<Mutex<HashMap<String, PlayerTimeData>>>,
    alert_players: Arc<Mutex<Vec<String>>>,
    alert_queue: Arc<Mutex<HashMap<String, Vec<i32>>>>,
    bot_status_players: Arc<Mutex<Vec<String>>>,
}

async fn handle(mut client: Client, event: Event, state: Arc<State>) -> anyhow::Result<()> {
    match event {
        Event::Login => {
            *state.client.lock().unwrap() = Some(client.clone());
            log_message(
                Bot,
                &"Successfully joined server, receiving initial data...".to_string(),
            );
            log_error(
                client
                    .set_client_information(ClientInformation {
                        view_distance: (state.bot_configuration.alert_radius as f32 / 16.0).ceil()
                            as u8,
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
            let mut player_timestamps = state.player_timestamps.lock().unwrap().to_owned();
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
            let mut player_timestamps = state.player_timestamps.lock().unwrap().to_owned();
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
            *state.tick_counter.lock().unwrap() += 1;
            if *state.tick_counter.lock().unwrap() >= 20 {
                *state.tick_counter.lock().unwrap() = 0;
                *state.alert_second_counter.lock().unwrap() += 1;
                *state.cleanup_second_counter.lock().unwrap() += 1;

                let followed_player = state.followed_player.lock().unwrap().to_owned();
                if followed_player.is_some() {
                    let player_locations = state.player_locations.lock().unwrap().to_owned();
                    match player_locations.get(&followed_player.unwrap()) {
                        Some(position_time_data) => client.goto(BlockPosGoal {
                            pos: BlockPos {
                                x: position_time_data.position[0],
                                y: position_time_data.position[1],
                                z: position_time_data.position[2],
                            },
                        }),
                        None => (),
                    }
                }

                let looked_player = state.looked_player.lock().unwrap().to_owned();
                if looked_player.is_some() {
                    let player_locations = state.player_locations.lock().unwrap().to_owned();
                    match player_locations.get(&looked_player.unwrap()) {
                        Some(position_time_data) => client.look_at(&Vec3 {
                            x: position_time_data.position[0] as f64,
                            y: position_time_data.position[1] as f64,
                            z: position_time_data.position[2] as f64,
                        }),
                        None => (),
                    }
                }
            }

            if *state.alert_second_counter.lock().unwrap() as u32
                >= state.bot_configuration.alert_pause_time
            {
                *state.alert_second_counter.lock().unwrap() = 0;

                let alert_queue = state.alert_queue.lock().unwrap().to_owned();
                for (intruder, position) in alert_queue {
                    log_message(
                        Bot,
                        &format!(
                            "{} is in the specified alert radius at {} {} {}!",
                            intruder, position[0], position[1], position[2]
                        ),
                    );
                    let alert_players = state.alert_players.lock().unwrap().to_vec();
                    for alert_player in alert_players {
                        log_error(
                            client
                                .send_command_packet(&format!(
                                    "msg {} {}",
                                    alert_player,
                                    format!(
                                        "{} is near our base at {} {} {}!",
                                        intruder, position[0], position[1], position[2],
                                    )
                                ))
                                .await,
                        );
                    }
                    let mut alert_command = state.bot_configuration.alert_command.to_vec();
                    for argument in alert_command.iter_mut() {
                        *argument = argument.replace("{player_name}", &intruder);
                        *argument = argument.replace("{x}", &(position[0]).to_string());
                        *argument = argument.replace("{y}", &(position[1]).to_string());
                        *argument = argument.replace("{z}", &(position[2]).to_string());
                    }
                    if alert_command.len() >= 1 {
                        log_message(Bot, &"Executing alert shell command...".to_string());
                        let command_name = alert_command[0].to_owned();
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
                *state.alert_queue.lock().unwrap() = HashMap::new();
            }

            if *state.cleanup_second_counter.lock().unwrap() as u32
                >= state.bot_configuration.cleanup_interval
            {
                *state.cleanup_second_counter.lock().unwrap() = 0;

                log_message(Bot, &"Cleaning up mob locations...".to_string());
                let mut mob_locations = state.mob_locations.lock().unwrap().to_owned();
                let before_count = mob_locations.len();
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                for (mob, position_time_data) in mob_locations.to_owned() {
                    if current_time - position_time_data.time
                        > state.bot_configuration.mob_expiry_time
                    {
                        mob_locations.remove(&mob);
                    }
                }
                let after_count = mob_locations.len();
                *state.mob_locations.lock().unwrap() = mob_locations;

                let removed_count = before_count - after_count;
                let mut label = "mobs";
                if removed_count == 1 {
                    label = "mob";
                }
                log_message(
                    Bot,
                    &format!(
                        "Successfully removed {} {} ({} -> {})",
                        removed_count, label, before_count, after_count
                    ),
                );
            }
        }
        Event::Packet(packet) => match packet.as_ref() {
            ClientboundGamePacket::AddEntity(packet) => {
                if packet.entity_type.to_string() != "Player" {
                    let entity = Entity {
                        id: packet.id,
                        uuid: packet.uuid.as_hyphenated().to_string(),
                        entity_type: packet.entity_type.to_string().to_lowercase(),
                    };

                    let mut mob_locations = state.mob_locations.lock().unwrap().to_owned();
                    mob_locations.insert(
                        entity,
                        PositionTimeData {
                            position: vec![packet.x as i32, packet.y as i32, packet.z as i32],
                            time: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        },
                    );
                    *state.mob_locations.lock().unwrap() = mob_locations;
                }
            }
            ClientboundGamePacket::MoveEntityPos(packet) => {
                let world = client.world.read();
                let raw_entity = match world.entity(packet.entity_id) {
                    Some(raw_entity) => raw_entity,
                    None => return Ok(()),
                };
                let entity_type = format!("{:?}", raw_entity.metadata)
                    .split("(")
                    .map(|item| item.to_owned())
                    .collect::<Vec<String>>()[0]
                    .to_lowercase();
                if entity_type != "player" {
                    if rand::thread_rng().gen_range(0..10) + 1
                        > 10 - state.bot_configuration.mob_packet_drop_level
                    {
                        return Ok(());
                    }
                }
                let entity = Entity {
                    id: raw_entity.id,
                    uuid: raw_entity.uuid.as_hyphenated().to_string(),
                    entity_type: entity_type.to_owned(),
                };
                let entity_position = raw_entity.pos();

                if entity_type == "player" {
                    let players = client.players.read().to_owned();
                    for (uuid, player) in players.iter().map(|item| item.to_owned()) {
                        if uuid.as_hyphenated().to_string() == entity.uuid {
                            let mut player_locations =
                                state.player_locations.lock().unwrap().to_owned();
                            let username = player.profile.name.to_owned();
                            for (player, _) in player_locations.to_owned() {
                                if player.username == username {
                                    player_locations.remove(&player);
                                }
                            }
                            player_locations.insert(
                                Player {
                                    uuid: uuid.as_hyphenated().to_string(),
                                    entity_id: entity.id,
                                    username,
                                },
                                PositionTimeData {
                                    position: vec![
                                        entity_position.x as i32,
                                        entity_position.y as i32,
                                        entity_position.z as i32,
                                    ],
                                    time: SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs(),
                                },
                            );
                            *state.player_locations.lock().unwrap() = player_locations;

                            if ((state.bot_configuration.alert_location[0]
                                - state.bot_configuration.alert_radius as i32)
                                ..(state.bot_configuration.alert_location[0]
                                    + state.bot_configuration.alert_radius as i32))
                                .contains(&(entity_position.x as i32))
                                && ((state.bot_configuration.alert_location[1]
                                    - state.bot_configuration.alert_radius as i32)
                                    ..(state.bot_configuration.alert_location[1]
                                        + state.bot_configuration.alert_radius as i32))
                                    .contains(&(entity_position.z as i32))
                            {
                                if !state
                                    .whitelist
                                    .lock()
                                    .unwrap()
                                    .contains(&player.profile.name)
                                {
                                    let mut alert_queue =
                                        state.alert_queue.lock().unwrap().to_owned();
                                    alert_queue.insert(
                                        player.profile.name.to_owned(),
                                        vec![
                                            entity_position.x as i32,
                                            entity_position.y as i32,
                                            entity_position.z as i32,
                                        ],
                                    );
                                    *state.alert_queue.lock().unwrap() = alert_queue;
                                }
                            }
                        }
                    }
                } else {
                    let mut mob_locations = state.mob_locations.lock().unwrap().to_owned();
                    mob_locations.insert(
                        entity,
                        PositionTimeData {
                            position: vec![
                                entity_position.x as i32,
                                entity_position.y as i32,
                                entity_position.z as i32,
                            ],
                            time: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        },
                    );
                    *state.mob_locations.lock().unwrap() = mob_locations;
                }
            }
            ClientboundGamePacket::SetHealth(packet) => {
                *state.bot_status.lock().unwrap() = BotStatus {
                    health: packet.health,
                    food: packet.food,
                    saturation: packet.saturation,
                };
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
                                    "Health: {:.1}/20, Food: {}/20, Saturation: {:.1}/20",
                                    packet.health, packet.food, packet.saturation
                                ),
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

            for bot_owner in state.bot_configuration.bot_owners.to_owned() {
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

                    let return_value =
                        &bot::process_command(&command, &bot_owner, &mut client, state.clone())
                            .await;
                    log_error(
                        client
                            .send_command_packet(&format!("msg {} {}", bot_owner, return_value))
                            .await,
                    );
                }

                let mut player_timestamps = state.player_timestamps.lock().unwrap().to_owned();
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
