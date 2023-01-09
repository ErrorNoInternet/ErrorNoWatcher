use crate::{logging::log_error, State};
use azalea::{pathfinder::BlockPosGoal, prelude::*, BlockPos};
use azalea_protocol::packets::game::{
    self, serverbound_interact_packet::InteractionHand, ServerboundGamePacket,
};
use chrono::{Local, TimeZone};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, PartialOrd, EnumIter)]
pub enum Command {
    Help,
    BotStatus,
    LastLocation,
    LastOnline,
    FollowPlayer,
    StopFollowPlayer,
    Goto,
    StopGoto,
    Say,
    Slot,
    UseItem,
    Look,
    ToggleBotStatusMessages,
    ToggleAlertMessages,
    Unknown,
}

pub async fn process_command(
    command: &String,
    executor: &String,
    client: &mut Client,
    state: &mut State,
) -> String {
    let mut segments: Vec<String> = command
        .split(" ")
        .map(|segment| segment.to_string())
        .collect();
    if segments.len() <= 0 {
        return "Hmm... I was unable to parse your command!".to_string();
    };

    let mut command = Command::Unknown;
    match segments[0].to_lowercase().as_str() {
        "help" => command = Command::Help,
        "bot_status" => command = Command::BotStatus,
        "last_location" => command = Command::LastLocation,
        "last_online" => command = Command::LastOnline,
        "follow_player" => command = Command::FollowPlayer,
        "stop_follow_player" => command = Command::StopFollowPlayer,
        "goto" => command = Command::Goto,
        "stop_goto" => command = Command::StopGoto,
        "say" => command = Command::Say,
        "slot" => command = Command::Slot,
        "use_item" => command = Command::UseItem,
        "look" => command = Command::Look,
        "toggle_alert_messages" => command = Command::ToggleAlertMessages,
        "toggle_bot_status_messages" => command = Command::ToggleBotStatusMessages,
        _ => (),
    };
    segments.remove(0);
    let return_value = match command {
        Command::Help => {
            let mut commands = Vec::new();
            for command in Command::iter() {
                if command != Command::Unknown {
                    commands.push(format!("{:?}", command));
                }
            }
            return "Commands: ".to_owned() + &commands.join(", ");
        }
        Command::BotStatus => {
            let metadata = client.metadata();
            return format!(
                "Health: {}/20, Score: {}, Air Supply: {}",
                metadata.health, metadata.score, metadata.air_supply
            );
        }
        Command::LastLocation => {
            if segments.len() < 1 {
                return "Please tell me the name of the player!".to_string();
            }

            for (player, position_time_data) in state.player_locations.lock().unwrap().iter() {
                if player.username == segments[0] || player.uuid.to_string() == segments[0] {
                    return format!(
                        "{} was last seen at {}, {}, {} ({})",
                        segments[0],
                        position_time_data.position[0],
                        position_time_data.position[1],
                        position_time_data.position[2],
                        Local
                            .timestamp_opt(position_time_data.time as i64, 0)
                            .unwrap()
                            .format("%Y/%m/%d %H:%M:%S")
                    );
                }
            }
            format!("I haven't seen {} move anywhere near me...", segments[0])
        }
        Command::LastOnline => {
            if segments.len() < 1 {
                return "Please tell me the name of the player!".to_string();
            }

            for (player, player_time_data) in state.player_timestamps.lock().unwrap().iter() {
                if player == &segments[0] {
                    return format!(
                        "{} - last join: {}, last chat message: {}, last leave: {}",
                        segments[0],
                        if player_time_data.join_time != 0 {
                            Local
                                .timestamp_opt(player_time_data.join_time as i64, 0)
                                .unwrap()
                                .format("%Y/%m/%d %H:%M:%S")
                                .to_string()
                        } else {
                            "never".to_string()
                        },
                        if player_time_data.chat_message_time != 0 {
                            Local
                                .timestamp_opt(player_time_data.chat_message_time as i64, 0)
                                .unwrap()
                                .format("%Y/%m/%d %H:%M:%S")
                                .to_string()
                        } else {
                            "never".to_string()
                        },
                        if player_time_data.leave_time != 0 {
                            Local
                                .timestamp_opt(player_time_data.leave_time as i64, 0)
                                .unwrap()
                                .format("%Y/%m/%d %H:%M:%S")
                                .to_string()
                        } else {
                            "never".to_string()
                        },
                    );
                }
            }
            format!("I haven't seen {} online yet...", segments[0])
        }
        Command::FollowPlayer => {
            if segments.len() < 1 {
                return "Please tell me the name of the player!".to_string();
            };

            let mut found = true;
            for (player, _position_time_data) in state.player_locations.lock().unwrap().iter() {
                if player.username == segments[0] || player.uuid.to_string() == segments[0] {
                    found = true;
                    *state.followed_player.lock().unwrap() = Some(player.to_owned());
                }
            }
            if found {
                return format!("I am now following {}...", segments[0]);
            } else {
                return format!("I was unable to find {}...", segments[0]);
            }
        }
        Command::StopFollowPlayer => {
            *state.followed_player.lock().unwrap() = None;
            let current_position = client.entity().pos().clone();
            client.goto(BlockPosGoal {
                pos: BlockPos {
                    x: current_position.x.round() as i32,
                    y: current_position.y.round() as i32,
                    z: current_position.z.round() as i32,
                },
            });
            "I am no longer following anyone!".to_string()
        }
        Command::Goto => {
            if segments.len() < 3 {
                return "Please give me X, Y, and Z coordinates to go to!".to_string();
            }

            let mut coordinates: Vec<i32> = Vec::new();
            for segment in segments {
                coordinates.push(match segment.parse() {
                    Ok(number) => number,
                    Err(error) => return format!("Unable to parse coordinates: {}", error),
                })
            }
            log_error(
                client
                    .send_command_packet(&format!(
                        "msg {} I am now finding a path to {} {} {}...",
                        executor, coordinates[0], coordinates[1], coordinates[2]
                    ))
                    .await,
            );
            client.goto(BlockPosGoal {
                pos: BlockPos {
                    x: coordinates[0],
                    y: coordinates[1],
                    z: coordinates[2],
                },
            });
            format!(
                "I have found the path to {} {} {}!",
                coordinates[0], coordinates[1], coordinates[2]
            )
        }
        Command::StopGoto => {
            let current_position = client.entity().pos().clone();
            client.goto(BlockPosGoal {
                pos: BlockPos {
                    x: current_position.x.round() as i32,
                    y: current_position.y.round() as i32,
                    z: current_position.z.round() as i32,
                },
            });
            "I am no longer going anywhere!".to_string()
        }
        Command::Say => {
            if segments.len() < 1 {
                return "Please give me something to say!".to_string();
            }

            log_error(client.chat(segments.join(" ").as_str()).await);
            "Successfully sent message!".to_string()
        }
        Command::Slot => {
            if segments.len() < 1 {
                return "Please give me a slot to set!".to_string();
            }

            client
                .write_packet(ServerboundGamePacket::SetCarriedItem(
                    game::serverbound_set_carried_item_packet::ServerboundSetCarriedItemPacket {
                        slot: match segments[0].parse() {
                            Ok(number) => number,
                            Err(error) => return format!("Unable to parse slot: {}", error),
                        },
                    },
                ))
                .await
                .unwrap();
            "Successfully sent a `SetCarriedItem` packet to the server".to_string()
        }
        Command::UseItem => {
            client
                .write_packet(ServerboundGamePacket::UseItem(
                    game::serverbound_use_item_packet::ServerboundUseItemPacket {
                        hand: InteractionHand::MainHand,
                        sequence: 0,
                    },
                ))
                .await
                .unwrap();
            "Successfully sent a `UseItem` packet to the server".to_string()
        }
        Command::Look => {
            if segments.len() < 2 {
                return "Please give me rotation vectors to look at!".to_string();
            }

            let mut rotation: Vec<f32> = Vec::new();
            for segment in segments {
                rotation.push(match segment.parse() {
                    Ok(number) => number,
                    Err(error) => return format!("Unable to parse rotation: {}", error),
                })
            }
            client.set_rotation(rotation[0], rotation[1]);
            format!("I am now looking at {} {}!", rotation[0], rotation[1])
        }
        Command::ToggleAlertMessages => {
            if state.alert_players.lock().unwrap().contains(executor) {
                let mut players = state.alert_players.lock().unwrap().to_vec();
                players.remove(
                    players
                        .iter()
                        .position(|item| *item == executor.to_owned())
                        .unwrap(),
                );
                *state.alert_players.lock().unwrap() = players;
                "You will no longer be receiving alert messages!".to_string()
            } else {
                let mut players = state.alert_players.lock().unwrap().to_vec();
                players.push(executor.to_owned());
                *state.alert_players.lock().unwrap() = players;
                "You will now be receiving alert messages!".to_string()
            }
        }
        Command::ToggleBotStatusMessages => {
            if state.bot_status_players.lock().unwrap().contains(executor) {
                let mut players = state.bot_status_players.lock().unwrap().to_vec();
                players.remove(
                    players
                        .iter()
                        .position(|item| *item == executor.to_owned())
                        .unwrap(),
                );
                *state.bot_status_players.lock().unwrap() = players;
                "You will no longer be receiving bot status messages!".to_string()
            } else {
                let mut players = state.bot_status_players.lock().unwrap().to_vec();
                players.push(executor.to_owned());
                *state.bot_status_players.lock().unwrap() = players;
                "You will now be receiving bot status messages!".to_string()
            }
        }
        _ => "".to_string(),
    };
    if !return_value.is_empty() {
        return return_value;
    }

    "Sorry, I don't know what you mean...".to_string()
}
