use std::sync::Arc;

use crate::{logging::log_error, PlayerTimeData, State};
use async_recursion::async_recursion;
use azalea::{
    pathfinder::BlockPosGoal, prelude::*, BlockPos, SprintDirection, Vec3, WalkDirection,
};
use azalea_core::Direction;
use azalea_protocol::packets::game::{
    self, serverbound_interact_packet::InteractionHand, ServerboundGamePacket,
};
use chrono::{Local, TimeZone};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, PartialOrd, EnumIter)]
pub enum Command {
    Help,
    Wait,
    Online,
    Location,
    BotStatus,
    Whitelist,
    WhitelistAdd,
    WhitelistRemove,
    LastLocation,
    LastOnline,
    FollowPlayer,
    StopFollowPlayer,
    LookPlayer,
    StopLookPlayer,
    Goto,
    StopGoto,
    Say,
    Slot,
    UseItem,
    Look,
    Sneak,
    Unsneak,
    PlaceBlock,
    InteractBlock,
    InteractEntity,
    Attack,
    Jump,
    Walk,
    Sprint,
    DropItem,
    DropStack,
    LeaveBed,
    Script,
    Latency,
    MobLocations,
    ToggleBotStatusMessages,
    ToggleAlertMessages,
    Unknown,
}

#[async_recursion]
pub async fn process_command(
    command: &String,
    executor: &String,
    client: &mut Client,
    state: Arc<State>,
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
        "wait" | "sleep" | "pause" => command = Command::Wait,
        "online" | "players" => command = Command::Online,
        "location" | "position" | "coordinates" => command = Command::Location,
        "bot_status" | "health" | "food" | "saturation" => command = Command::BotStatus,
        "whitelist" => command = Command::Whitelist,
        "whitelist_add" => command = Command::WhitelistAdd,
        "whitelist_remove" => command = Command::WhitelistRemove,
        "last_location" | "last_position" => command = Command::LastLocation,
        "last_online" => command = Command::LastOnline,
        "follow_player" => command = Command::FollowPlayer,
        "stop_follow_player" => command = Command::StopFollowPlayer,
        "look_player" => command = Command::LookPlayer,
        "stop_look_player" => command = Command::StopLookPlayer,
        "goto" => command = Command::Goto,
        "stop_goto" => command = Command::StopGoto,
        "say" => command = Command::Say,
        "slot" => command = Command::Slot,
        "use_item" => command = Command::UseItem,
        "look" => command = Command::Look,
        "sneak" | "shift" | "crouch" => command = Command::Sneak,
        "unsneak" | "unshift" | "uncrouch" => command = Command::Unsneak,
        "place_block" | "place" => command = Command::PlaceBlock,
        "interact_block" => command = Command::InteractBlock,
        "interact_entity" => command = Command::InteractEntity,
        "attack" | "hit" => command = Command::Attack,
        "jump" => command = Command::Jump,
        "walk" => command = Command::Walk,
        "sprint" => command = Command::Sprint,
        "drop_item" | "throw_item" => command = Command::DropItem,
        "drop_stack" | "throw_stack" => command = Command::DropStack,
        "leave_bed" => command = Command::LeaveBed,
        "script" | "run" => command = Command::Script,
        "latency" | "ping" => command = Command::Latency,
        "mob_locations" => command = Command::MobLocations,
        "toggle_alert_messages" => command = Command::ToggleAlertMessages,
        "toggle_bot_status_messages" => command = Command::ToggleBotStatusMessages,
        _ => (),
    };
    segments.remove(0);
    let return_value = match command {
        Command::Help => {
            let mut page = 1;
            if segments.len() > 0 {
                page = segments[0].parse().unwrap_or(1)
            }
            if page < 1 {
                page = 1
            }

            let mut commands = Vec::new();
            for command in Command::iter() {
                if command != Command::Unknown {
                    commands.push(format!("{:?}", command));
                }
            }

            let mut start_index = (page - 1) * 10;
            let mut end_index = page * 10;
            while start_index > commands.len() {
                start_index -= 1
            }
            while end_index > commands.len() {
                end_index -= 1
            }
            let paged_commands = &commands[start_index..end_index];
            return format!("Commands (page {}): {}", page, paged_commands.join(", "));
        }
        Command::Wait => {
            if segments.len() < 1 {
                return "Please tell me how long to wait!".to_string();
            }

            let duration = match segments[0].parse() {
                Ok(duration) => duration,
                Err(error) => return format!("Unable to parse duration: {}", error),
            };
            tokio::time::sleep(std::time::Duration::from_millis(duration)).await;
            return format!("I have successfully slept for {} ms!", duration);
        }
        Command::Online => {
            let mut page = 1;
            if segments.len() > 0 {
                page = segments[0].parse().unwrap_or(1)
            }
            if page < 1 {
                page = 1
            }

            let players: Vec<String> = client
                .players
                .read()
                .values()
                .map(|item| item.profile.name.to_owned())
                .collect();

            let mut start_index = (page - 1) * 10;
            let mut end_index = page * 10;
            while start_index > players.len() {
                start_index -= 1
            }
            while end_index > players.len() {
                end_index -= 1
            }
            let paged_players = &players[start_index..end_index];
            return format!(
                "Online players (page {}): {}",
                page,
                paged_players.join(", ")
            );
        }
        Command::Location => {
            let world = client.world.read();
            let entity = match world.entity(client.entity_id.read().to_owned()) {
                Some(entity) => entity,
                None => return "Uh oh! An unknown error occurred!".to_string(),
            };
            return format!(
                "I am currently at {} {} {}!",
                entity.last_pos.x as i32, entity.last_pos.y as i32, entity.last_pos.z as i32
            );
        }
        Command::BotStatus => {
            let bot_status = state.bot_status.lock().unwrap().to_owned();
            let metadata = client.metadata();
            return format!(
                "Health: {:.1}/20.0, Food: {}/20, Saturation: {:.1}/20.0, Score: {}, Air Supply: {}",
                bot_status.health,
                bot_status.food,
                bot_status.saturation,
                metadata.score,
                metadata.air_supply
            );
        }
        Command::Whitelist => {
            let mut page = 1;
            if segments.len() > 0 {
                page = segments[0].parse().unwrap_or(1)
            }
            if page < 1 {
                page = 1
            }

            let whitelist = state.whitelist.lock().unwrap();

            let mut start_index = (page - 1) * 10;
            let mut end_index = page * 10;
            while start_index > whitelist.len() {
                start_index -= 1
            }
            while end_index > whitelist.len() {
                end_index -= 1
            }
            let paged_whitelist = &whitelist[start_index..end_index];
            return format!(
                "Whitelisted players (page {}): {}",
                page,
                paged_whitelist.join(", ")
            );
        }
        Command::WhitelistAdd => {
            if segments.len() < 1 {
                return "Please tell me the name of a player!".to_string();
            }

            let mut whitelist = state.whitelist.lock().unwrap().to_vec();
            if whitelist.contains(&segments[0]) {
                return format!("{} is already whitelisted!", segments[0]);
            }
            whitelist.push(segments[0].to_owned());
            *state.whitelist.lock().unwrap() = whitelist;
            return format!(
                "{} has been successfully added to the whitelist!",
                segments[0]
            );
        }
        Command::WhitelistRemove => {
            if segments.len() < 1 {
                return "Please tell me the name of a player!".to_string();
            }

            let mut whitelist = state.whitelist.lock().unwrap().to_vec();
            if !whitelist.contains(&segments[0]) {
                return format!("{} is not whitelisted!", segments[0]);
            }
            whitelist.remove(
                whitelist
                    .iter()
                    .position(|item| *item == segments[0])
                    .unwrap(),
            );
            *state.whitelist.lock().unwrap() = whitelist;
            return format!(
                "{} has been successfully removed from the whitelist!",
                segments[0]
            );
        }
        Command::LastLocation => {
            if segments.len() < 1 {
                return "Please tell me the name of a player!".to_string();
            }

            let player_locations = state.player_locations.lock().unwrap().to_owned();
            for (player, position_time_data) in player_locations {
                if player.username == segments[0] || player.uuid.to_string() == segments[0] {
                    return format!(
                        "{} was last seen at {} {} {} ({})",
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
            let mut page = 1;
            if segments.len() > 0 {
                if segments[0].parse::<usize>().is_ok() {
                    page = segments[0].parse().unwrap();
                    if page < 1 {
                        page = 1
                    }
                } else {
                    for (player, player_time_data) in state.player_timestamps.lock().unwrap().iter()
                    {
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
                    return format!("I haven't seen {} online yet...", segments[0]);
                }
            }

            let mut sorted_player_time_data: Vec<PlayerTimeData> = state
                .player_timestamps
                .lock()
                .unwrap()
                .to_owned()
                .values()
                .map(|item| item.to_owned())
                .collect();
            sorted_player_time_data.sort_by(|a, b| b.join_time.cmp(&a.join_time));
            let mut player_timestamps = state.player_timestamps.lock().unwrap().to_owned();
            let mut players = Vec::new();
            for player_time_data in sorted_player_time_data {
                for (player, original_player_time_data) in player_timestamps.to_owned().iter() {
                    if player_time_data == original_player_time_data.to_owned() {
                        players.push(player.to_owned());
                        player_timestamps.remove(player);
                        break;
                    }
                }
            }

            let mut start_index = (page - 1) * 10;
            let mut end_index = page * 10;
            while start_index > players.len() {
                start_index -= 1
            }
            while end_index > players.len() {
                end_index -= 1
            }
            let paged_players = &players[start_index..end_index];
            return format!(
                "Sorted by join time (page {}): {}",
                page,
                paged_players.join(", ")
            );
        }
        Command::FollowPlayer => {
            if segments.len() < 1 {
                return "Please tell me the name of a player!".to_string();
            };

            let mut found = true;
            let player_locations = state.player_locations.lock().unwrap().to_owned();
            for (player, _) in player_locations {
                if player.username == segments[0] || player.uuid.to_string() == segments[0] {
                    found = true;
                    *state.followed_player.lock().unwrap() = Some(player.to_owned());
                }
            }
            if found {
                return format!("I am now following {}!", segments[0]);
            } else {
                return format!("I was unable to find {}!", segments[0]);
            }
        }
        Command::StopFollowPlayer => {
            *state.followed_player.lock().unwrap() = None;
            let current_position = client.entity().pos().to_owned();
            client.goto(BlockPosGoal {
                pos: BlockPos {
                    x: current_position.x.round() as i32,
                    y: current_position.y.round() as i32,
                    z: current_position.z.round() as i32,
                },
            });
            "I am no longer following anyone!".to_string()
        }
        Command::LookPlayer => {
            if segments.len() < 1 {
                return "Please tell me the name of a player!".to_string();
            };

            let mut found = true;
            let player_locations = state.player_locations.lock().unwrap().to_owned();
            for (player, _) in player_locations {
                if player.username == segments[0] || player.uuid.to_string() == segments[0] {
                    found = true;
                    *state.looked_player.lock().unwrap() = Some(player.to_owned());
                }
            }
            if found {
                return format!("I am now looking at {}!", segments[0]);
            } else {
                return format!("I was unable to find {}!", segments[0]);
            }
        }
        Command::StopLookPlayer => {
            *state.looked_player.lock().unwrap() = None;
            "I am no longer looking at anyone!".to_string()
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
            let current_position = client.entity().pos().to_owned();
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

            log_error(
                client
                    .write_packet(ServerboundGamePacket::SetCarriedItem(
                        game::serverbound_set_carried_item_packet::ServerboundSetCarriedItemPacket {
                            slot: match segments[0].parse::<i8>() {
                                Ok(number) => (number-1) as u16,
                                Err(error) => return format!("Unable to parse slot: {}", error),
                            },
                        },
                    ))
                    .await
            );
            "I have successfully switched slots!".to_string()
        }
        Command::UseItem => {
            log_error(
                client
                    .write_packet(ServerboundGamePacket::UseItem(
                        game::serverbound_use_item_packet::ServerboundUseItemPacket {
                            hand: InteractionHand::MainHand,
                            sequence: 0,
                        },
                    ))
                    .await,
            );
            "I have successfully used the item!".to_string()
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
        Command::Sneak => {
            let entity_id = client.entity_id.read().to_owned();
            log_error(
                client
                    .write_packet(ServerboundGamePacket::PlayerCommand(
                        game::serverbound_player_command_packet::ServerboundPlayerCommandPacket {
                            id: entity_id,
                            action: game::serverbound_player_command_packet::Action::PressShiftKey,
                            data: 0,
                        },
                    ))
                    .await,
            );
            return "I am now sneaking!".to_string();
        }
        Command::Unsneak => {
            let entity_id = client.entity_id.read().to_owned();
            log_error(
                client
                    .write_packet(ServerboundGamePacket::PlayerCommand(
                        game::serverbound_player_command_packet::ServerboundPlayerCommandPacket {
                            id: entity_id,
                            action:
                                game::serverbound_player_command_packet::Action::ReleaseShiftKey,
                            data: 0,
                        },
                    ))
                    .await,
            );
            return "I am no longer sneaking!".to_string();
        }
        Command::PlaceBlock => {
            if segments.len() < 4 {
                return "Please give me block coordinates, block faces, and optionally cursor positions to interact with!"
                    .to_string();
            }

            let mut coordinates: Vec<i32> = Vec::new();
            for segment in &segments[0..3] {
                coordinates.push(match segment.parse() {
                    Ok(number) => number,
                    Err(error) => return format!("Unable to parse coordinates: {}", error),
                })
            }
            let block_face = match segments[3].to_lowercase().as_str() {
                "up" | "top" => Direction::Up,
                "down" | "bottom" => Direction::Down,
                "north" => Direction::North,
                "east" => Direction::East,
                "south" => Direction::South,
                "west" => Direction::West,
                _ => return "Please give me a valid block face!".to_string(),
            };
            let mut cursor_positions: Vec<f64> = Vec::new();
            if segments.len() >= 7 {
                for segment in &segments[4..7] {
                    cursor_positions.push(match segment.parse() {
                        Ok(number) => number,
                        Err(error) => {
                            return format!("Unable to parse cursor positions: {}", error)
                        }
                    })
                }
            }
            if cursor_positions.len() == 0 {
                cursor_positions = vec![0.5, 1.0, 0.5];
            }
            log_error(
                client
                    .write_packet(ServerboundGamePacket::UseItemOn(
                        game::serverbound_use_item_on_packet::ServerboundUseItemOnPacket {
                            hand: InteractionHand::MainHand,
                            block_hit: game::serverbound_use_item_on_packet::BlockHitResult {
                                block_pos: BlockPos {
                                    x: coordinates[0],
                                    y: coordinates[1],
                                    z: coordinates[2],
                                },
                                direction: block_face,
                                location: Vec3 {
                                    x: cursor_positions[0] + coordinates[0] as f64,
                                    y: cursor_positions[1] + coordinates[1] as f64,
                                    z: cursor_positions[2] + coordinates[2] as f64,
                                },
                                inside: false,
                            },
                            sequence: 0,
                        },
                    ))
                    .await,
            );
            return "I have successfully interacted with the block!".to_string();
        }
        Command::InteractBlock => {
            if segments.len() < 4 {
                return "Please give me block coordinates (and block faces) to interact with!"
                    .to_string();
            }

            let mut coordinates: Vec<i32> = Vec::new();
            for segment in &segments[0..3] {
                coordinates.push(match segment.parse() {
                    Ok(number) => number,
                    Err(error) => return format!("Unable to parse coordinates: {}", error),
                })
            }
            let block_face = match segments[3].to_lowercase().as_str() {
                "up" | "top" => Direction::Up,
                "down" | "bottom" => Direction::Down,
                "north" => Direction::North,
                "east" => Direction::East,
                "south" => Direction::South,
                "west" => Direction::West,
                _ => return "Please give me a valid block face!".to_string(),
            };
            log_error(
                client
                    .write_packet(ServerboundGamePacket::UseItemOn(
                        game::serverbound_use_item_on_packet::ServerboundUseItemOnPacket {
                            hand: InteractionHand::MainHand,
                            block_hit: game::serverbound_use_item_on_packet::BlockHitResult {
                                block_pos: BlockPos {
                                    x: coordinates[0],
                                    y: coordinates[1],
                                    z: coordinates[2],
                                },
                                direction: block_face,
                                location: Vec3 {
                                    x: coordinates[0] as f64,
                                    y: coordinates[1] as f64,
                                    z: coordinates[2] as f64,
                                },
                                inside: false,
                            },
                            sequence: 0,
                        },
                    ))
                    .await,
            );
            return "I have successfully interacted with the block!".to_string();
        }
        Command::InteractEntity => {
            if segments.len() < 1 {
                return "Please give me IDs to interact with!".to_string();
            }

            let mut found = false;
            let mob_locations = state.mob_locations.lock().unwrap().to_owned();
            for (mob, _) in mob_locations {
                if mob.id.to_string() == segments[0]
                    || mob.uuid == segments[0]
                    || mob.entity_type == segments[0]
                {
                    found = true;
                    log_error(
                        client
                            .write_packet(ServerboundGamePacket::Interact(
                                game::serverbound_interact_packet::ServerboundInteractPacket {
                                    action:
                                        game::serverbound_interact_packet::ActionType::Interact {
                                            hand: InteractionHand::MainHand,
                                        },
                                    entity_id: mob.id,
                                    using_secondary_action: false,
                                },
                            ))
                            .await,
                    );
                }
            }
            let player_locations = state.player_locations.lock().unwrap().to_owned();
            for (player, _) in player_locations {
                if player.entity_id.to_string() == segments[0]
                    || player.uuid == segments[0]
                    || player.username == segments[0]
                {
                    found = true;
                    log_error(
                        client
                            .write_packet(ServerboundGamePacket::Interact(
                                game::serverbound_interact_packet::ServerboundInteractPacket {
                                    action:
                                        game::serverbound_interact_packet::ActionType::Interact {
                                            hand: InteractionHand::MainHand,
                                        },
                                    entity_id: player.entity_id,
                                    using_secondary_action: false,
                                },
                            ))
                            .await,
                    );
                }
            }
            if found {
                return "Successfully interacted with entity!".to_string();
            } else {
                return "Unable to find entity!".to_string();
            }
        }
        Command::Attack => {
            if segments.len() < 1 {
                return "Please give me IDs to attack!".to_string();
            }

            let mut found = false;
            let mob_locations = state.mob_locations.lock().unwrap().to_owned();
            for (mob, _) in mob_locations {
                if mob.id.to_string() == segments[0]
                    || mob.uuid == segments[0]
                    || mob.entity_type == segments[0]
                {
                    found = true;
                    log_error(
                        client
                            .write_packet(ServerboundGamePacket::Interact(
                                game::serverbound_interact_packet::ServerboundInteractPacket {
                                    action: game::serverbound_interact_packet::ActionType::Attack,
                                    entity_id: mob.id,
                                    using_secondary_action: false,
                                },
                            ))
                            .await,
                    );
                }
            }
            let player_locations = state.player_locations.lock().unwrap().to_owned();
            for (player, _) in player_locations {
                if player.entity_id.to_string() == segments[0]
                    || player.uuid == segments[0]
                    || player.username == segments[0]
                {
                    found = true;
                    log_error(
                        client
                            .write_packet(ServerboundGamePacket::Interact(
                                game::serverbound_interact_packet::ServerboundInteractPacket {
                                    action: game::serverbound_interact_packet::ActionType::Attack,
                                    entity_id: player.entity_id,
                                    using_secondary_action: false,
                                },
                            ))
                            .await,
                    );
                }
            }
            if found {
                return "Successfully attacked entity!".to_string();
            } else {
                return "Unable to find entity!".to_string();
            }
        }
        Command::Jump => {
            client.jump();
            return "I have successfully jumped!".to_string();
        }
        Command::Walk => {
            if segments.len() < 2 {
                return "Please give me a direction (and duration) to walk in!".to_string();
            }

            let direction = match segments[0].to_lowercase().as_str() {
                "forward" => WalkDirection::Forward,
                "forward_left" => WalkDirection::ForwardLeft,
                "forward_right" => WalkDirection::ForwardRight,
                "backward" => WalkDirection::Backward,
                "backward_left" => WalkDirection::BackwardLeft,
                "backward_right" => WalkDirection::BackwardRight,
                "left" => WalkDirection::Left,
                "right" => WalkDirection::Right,
                _ => WalkDirection::None,
            };
            let duration = match segments[1].parse() {
                Ok(duration) => duration,
                Err(error) => return format!("Unable to parse duration: {}", error),
            };

            client.walk(direction);
            tokio::time::sleep(std::time::Duration::from_millis(duration)).await;
            client.walk(WalkDirection::None);
            return "I have finished walking!".to_string();
        }
        Command::Sprint => {
            if segments.len() < 2 {
                return "Please give me a direction (and duration) to sprint in!".to_string();
            }

            let direction = match segments[0].to_lowercase().as_str() {
                "forward" => SprintDirection::Forward,
                "forward_left" => SprintDirection::ForwardLeft,
                "forward_right" => SprintDirection::ForwardRight,
                _ => return "Please give me a valid direction to sprint in!".to_string(),
            };
            let duration = match segments[1].parse() {
                Ok(duration) => duration,
                Err(error) => return format!("Unable to parse duration: {}", error),
            };

            client.sprint(direction);
            tokio::time::sleep(std::time::Duration::from_millis(duration)).await;
            client.walk(WalkDirection::None);
            return "I have finished sprinting!".to_string();
        }
        Command::DropItem => {
            log_error(
                client
                    .write_packet(ServerboundGamePacket::PlayerAction(
                        game::serverbound_player_action_packet::ServerboundPlayerActionPacket {
                            action: game::serverbound_player_action_packet::Action::DropItem,
                            pos: BlockPos { x: 0, y: 0, z: 0 },
                            direction: Default::default(),
                            sequence: 0,
                        },
                    ))
                    .await,
            );
            return "I have successfully dropped 1 item!".to_string();
        }
        Command::DropStack => {
            log_error(
                client
                    .write_packet(ServerboundGamePacket::PlayerAction(
                        game::serverbound_player_action_packet::ServerboundPlayerActionPacket {
                            action: game::serverbound_player_action_packet::Action::DropAllItems,
                            pos: BlockPos { x: 0, y: 0, z: 0 },
                            direction: Default::default(),
                            sequence: 0,
                        },
                    ))
                    .await,
            );
            return "I have successfully dropped 1 stack!".to_string();
        }
        Command::LeaveBed => {
            let entity_id = client.entity_id.read().to_owned();
            log_error(
                client
                    .write_packet(ServerboundGamePacket::PlayerCommand(
                        game::serverbound_player_command_packet::ServerboundPlayerCommandPacket {
                            id: entity_id,
                            action: game::serverbound_player_command_packet::Action::StopSleeping,
                            data: 0,
                        },
                    ))
                    .await,
            );
            return "I am no longer sleeping!".to_string();
        }
        Command::Script => {
            if segments.len() < 1 {
                return "Please give me a script file to run!".to_string();
            }

            let script = match std::fs::read_to_string(segments[0].to_owned()) {
                Ok(script) => script,
                Err(error) => return format!("Unable to read script: {}", error),
            };
            for line in script.split("\n") {
                process_command(&line.to_string(), &executor, client, state.clone()).await;
            }

            return "Finished executing script!".to_string();
        }
        Command::Latency => {
            let mut player = &state.bot_configuration.username;
            if segments.len() > 0 {
                player = &segments[0]
            }

            let players = client.players.read().to_owned();
            for (uuid, online_player) in players.iter().map(|item| item.to_owned()) {
                if &online_player.profile.name == player
                    || &uuid.as_hyphenated().to_string() == player
                {
                    return format!(
                        "{} has a latency of {} ms!",
                        online_player.profile.name, online_player.latency
                    );
                }
            }

            return format!("{} was not found!", player);
        }
        Command::MobLocations => {
            if segments.len() < 1 {
                return "Please give me the ID or type of a mob!".to_string();
            }
            let mut page = 1;
            if segments.len() > 1 {
                page = segments[1].parse().unwrap_or(1)
            }
            if page < 1 {
                page = 1
            }

            let mut locations = Vec::new();
            for (entity, position_time_data) in state.mob_locations.lock().unwrap().to_owned() {
                if entity.id.to_string() == segments[0]
                    || entity.uuid == segments[0]
                    || entity.entity_type == segments[0]
                {
                    locations.push(format!(
                        "{}: {} {} {}",
                        entity.entity_type,
                        position_time_data.position[0],
                        position_time_data.position[1],
                        position_time_data.position[2],
                    ))
                }
            }

            let mut start_index = (page - 1) * 5;
            let mut end_index = page * 5;
            while start_index > locations.len() {
                start_index -= 1
            }
            while end_index > locations.len() {
                end_index -= 1
            }
            let paged_locations = &locations[start_index..end_index];
            return format!(
                "Mob locations (page {}): {}",
                page,
                paged_locations.join(", ")
            );
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
