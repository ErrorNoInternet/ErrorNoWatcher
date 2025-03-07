use super::{Client, Direction, Vec3};
use azalea::{
    BlockPos, BotClientExt, SprintDirection, WalkDirection,
    entity::Position,
    interact::HitResultComponent,
    pathfinder::{
        ExecutingPath, GotoEvent, Pathfinder, PathfinderClientExt,
        goals::{BlockPosGoal, Goal, InverseGoal, RadiusGoal, ReachBlockPosGoal, XZGoal, YGoal},
    },
    protocol::packets::game::{ServerboundPlayerCommand, s_player_command::Action},
    world::MinecraftEntityId,
};
use log::error;
use mlua::{FromLua, Lua, Result, Table, UserDataRef, Value};

pub fn direction(_lua: &Lua, client: &Client) -> Result<Direction> {
    let d = client.direction();
    Ok(Direction { y: d.0, x: d.1 })
}

pub fn eye_position(_lua: &Lua, client: &Client) -> Result<Vec3> {
    Ok(Vec3::from(client.eye_position()))
}

pub async fn go_to(
    lua: Lua,
    client: UserDataRef<Client>,
    (data, metadata): (Table, Option<Table>),
) -> Result<()> {
    fn goto_with_options<G: Goal + Send + Sync + 'static>(
        client: &Client,
        options: &Table,
        goal: G,
    ) {
        if options.get("without_mining").unwrap_or_default() {
            client.goto_without_mining(goal);
        } else {
            client.goto(goal);
        }
    }

    let table = metadata.unwrap_or(lua.create_table()?);
    let goal_type = table.get("type").unwrap_or_default();
    let options = table.get("options").unwrap_or(lua.create_table()?);

    macro_rules! goto {
        ($goal:expr) => {
            if options.get("inverse").unwrap_or_default() {
                goto_with_options(&client, &options, InverseGoal($goal));
            } else {
                goto_with_options(&client, &options, $goal);
            }
        };
    }

    #[allow(clippy::cast_possible_truncation)]
    match goal_type {
        1 => {
            let p = Vec3::from_lua(data.get("position")?, &lua)?;
            goto!(RadiusGoal {
                pos: azalea::Vec3::new(p.x, p.y, p.z),
                radius: data.get("radius")?,
            });
        }
        2 => {
            let p = Vec3::from_lua(Value::Table(data), &lua)?;
            goto!(ReachBlockPosGoal {
                pos: BlockPos::new(p.x as i32, p.y as i32, p.z as i32),
                chunk_storage: client.world().read().chunks.clone(),
            });
        }
        3 => {
            goto!(XZGoal {
                x: data.get("x")?,
                z: data.get("z")?,
            });
        }
        4 => goto!(YGoal { y: data.get("y")? }),
        _ => {
            let p = Vec3::from_lua(Value::Table(data), &lua)?;
            goto!(BlockPosGoal(BlockPos::new(
                p.x as i32, p.y as i32, p.z as i32
            )));
        }
    }

    while client.get_tick_broadcaster().recv().await.is_ok() {
        if client.ecs.lock().get::<GotoEvent>(client.entity).is_none() {
            break;
        }
    }

    Ok(())
}

pub fn jump(_lua: &Lua, client: &mut Client, _: ()) -> Result<()> {
    client.jump();
    Ok(())
}

pub fn looking_at(lua: &Lua, client: &Client) -> Result<Option<Table>> {
    let result = client.component::<HitResultComponent>();
    Ok(if result.miss {
        None
    } else {
        let table = lua.create_table()?;
        table.set("position", Vec3::from(result.block_pos))?;
        table.set("inside", result.inside)?;
        table.set("world_border", result.world_border)?;
        Some(table)
    })
}

pub fn look_at(_lua: &Lua, client: &mut Client, position: Vec3) -> Result<()> {
    client.look_at(azalea::Vec3::new(position.x, position.y, position.z));
    Ok(())
}

pub fn pathfinder(lua: &Lua, client: &Client) -> Result<Table> {
    let table = lua.create_table()?;
    table.set(
        "is_calculating",
        client.component::<Pathfinder>().is_calculating,
    )?;
    table.set(
        "is_executing",
        if let Some(pathfinder) = client.get_component::<ExecutingPath>() {
            table.set(
                "last_reached_node",
                Vec3::from(pathfinder.last_reached_node),
            )?;
            table.set(
                "last_node_reach_elapsed",
                pathfinder.last_node_reached_at.elapsed().as_millis(),
            )?;
            table.set("is_path_partial", pathfinder.is_path_partial)?;
            true
        } else {
            false
        },
    )?;
    Ok(table)
}

pub fn position(_lua: &Lua, client: &Client) -> Result<Vec3> {
    Ok(Vec3::from(&client.component::<Position>()))
}

pub fn set_direction(_lua: &Lua, client: &mut Client, direction: Direction) -> Result<()> {
    client.set_direction(direction.y, direction.x);
    Ok(())
}

pub fn set_jumping(_lua: &Lua, client: &mut Client, jumping: bool) -> Result<()> {
    client.set_jumping(jumping);
    Ok(())
}

pub fn set_position(_lua: &Lua, client: &Client, new_position: Vec3) -> Result<()> {
    let mut ecs = client.ecs.lock();
    let mut position = client.query::<&mut Position>(&mut ecs);
    position.x = new_position.x;
    position.y = new_position.y;
    position.z = new_position.z;
    Ok(())
}

pub fn set_sneaking(_lua: &Lua, client: &Client, sneaking: bool) -> Result<()> {
    if let Err(error) = client.write_packet(ServerboundPlayerCommand {
        id: client.component::<MinecraftEntityId>(),
        action: if sneaking {
            Action::PressShiftKey
        } else {
            Action::ReleaseShiftKey
        },
        data: 0,
    }) {
        error!("failed to send PlayerCommand packet: {error:?}");
    }
    Ok(())
}

pub fn sprint(_lua: &Lua, client: &mut Client, direction: u8) -> Result<()> {
    client.sprint(match direction {
        5 => SprintDirection::ForwardRight,
        6 => SprintDirection::ForwardLeft,
        _ => SprintDirection::Forward,
    });
    Ok(())
}

pub fn stop_pathfinding(_lua: &Lua, client: &Client, _: ()) -> Result<()> {
    client.stop_pathfinding();
    Ok(())
}

pub fn stop_sleeping(_lua: &Lua, client: &Client, _: ()) -> Result<()> {
    if let Err(error) = client.write_packet(ServerboundPlayerCommand {
        id: client.component::<MinecraftEntityId>(),
        action: Action::StopSleeping,
        data: 0,
    }) {
        error!("failed to send PlayerCommand packet: {error:?}");
    }
    Ok(())
}

pub fn walk(_lua: &Lua, client: &mut Client, direction: u8) -> Result<()> {
    client.walk(match direction {
        1 => WalkDirection::Forward,
        2 => WalkDirection::Backward,
        3 => WalkDirection::Left,
        4 => WalkDirection::Right,
        5 => WalkDirection::ForwardRight,
        6 => WalkDirection::ForwardLeft,
        7 => WalkDirection::BackwardRight,
        8 => WalkDirection::BackwardLeft,
        _ => WalkDirection::None,
    });
    Ok(())
}
