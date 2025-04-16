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

use super::{Client, Direction, Vec3};

#[derive(Debug)]
struct AnyGoal(Box<dyn Goal>);

impl Goal for AnyGoal {
    fn success(&self, n: BlockPos) -> bool {
        self.0.success(n)
    }

    fn heuristic(&self, n: BlockPos) -> f32 {
        self.0.heuristic(n)
    }
}

#[allow(clippy::cast_possible_truncation)]
fn to_goal(lua: &Lua, client: &Client, data: Table, options: &Table, kind: u8) -> Result<AnyGoal> {
    let goal: Box<dyn Goal> = match kind {
        1 => {
            let pos = Vec3::from_lua(data.get("position")?, lua)?;
            Box::new(RadiusGoal {
                pos: azalea::Vec3::new(pos.x, pos.y, pos.z),
                radius: data.get("radius")?,
            })
        }
        2 => {
            let pos = Vec3::from_lua(Value::Table(data), lua)?;
            Box::new(ReachBlockPosGoal {
                pos: BlockPos::new(pos.x as i32, pos.y as i32, pos.z as i32),
                chunk_storage: client.world().read().chunks.clone(),
            })
        }
        3 => Box::new(XZGoal {
            x: data.get("x")?,
            z: data.get("z")?,
        }),
        4 => Box::new(YGoal { y: data.get("y")? }),
        _ => {
            let pos = Vec3::from_lua(Value::Table(data), lua)?;
            Box::new(BlockPosGoal(BlockPos::new(
                pos.x as i32,
                pos.y as i32,
                pos.z as i32,
            )))
        }
    };

    Ok(AnyGoal(if options.get("inverse").unwrap_or_default() {
        Box::new(InverseGoal(AnyGoal(goal)))
    } else {
        goal
    }))
}

pub fn go_to_reached(_lua: &Lua, client: &Client) -> Result<bool> {
    Ok(client.is_goto_target_reached())
}

pub async fn go_to_wait_until_reached(
    _lua: Lua,
    client: UserDataRef<Client>,
    (): (),
) -> Result<()> {
    client.wait_until_goto_target_reached().await;
    Ok(())
}

pub async fn go_to(
    lua: Lua,
    client: UserDataRef<Client>,
    (data, metadata): (Table, Option<Table>),
) -> Result<()> {
    let metadata = metadata.unwrap_or(lua.create_table()?);
    let options = metadata.get("options").unwrap_or(lua.create_table()?);
    let goal = to_goal(
        &lua,
        &client,
        data,
        &options,
        metadata.get("type").unwrap_or_default(),
    )?;
    if options.get("without_mining").unwrap_or_default() {
        client.start_goto_without_mining(goal);
        client.wait_until_goto_target_reached().await;
    } else {
        client.goto(goal).await;
    }
    Ok(())
}

pub async fn start_go_to(
    lua: Lua,
    client: UserDataRef<Client>,
    (data, metadata): (Table, Option<Table>),
) -> Result<()> {
    let metadata = metadata.unwrap_or(lua.create_table()?);
    let options = metadata.get("options").unwrap_or(lua.create_table()?);
    let goal = to_goal(
        &lua,
        &client,
        data,
        &options,
        metadata.get("type").unwrap_or_default(),
    )?;
    if options.get("without_mining").unwrap_or_default() {
        client.start_goto_without_mining(goal);
    } else {
        client.start_goto(goal);
    }
    while client.get_tick_broadcaster().recv().await.is_ok() {
        if client.ecs.lock().get::<GotoEvent>(client.entity).is_none() {
            break;
        }
    }

    Ok(())
}

pub fn direction(_lua: &Lua, client: &Client) -> Result<Direction> {
    let direction = client.direction();
    Ok(Direction {
        y: direction.0,
        x: direction.1,
    })
}

pub fn eye_position(_lua: &Lua, client: &Client) -> Result<Vec3> {
    Ok(Vec3::from(client.eye_position()))
}

pub fn jump(_lua: &Lua, client: &Client, (): ()) -> Result<()> {
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

pub fn look_at(_lua: &Lua, client: &Client, position: Vec3) -> Result<()> {
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

pub fn set_direction(_lua: &Lua, client: &Client, direction: Direction) -> Result<()> {
    client.set_direction(direction.y, direction.x);
    Ok(())
}

pub fn set_jumping(_lua: &Lua, client: &Client, jumping: bool) -> Result<()> {
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

pub fn sprint(_lua: &Lua, client: &Client, direction: u8) -> Result<()> {
    client.sprint(match direction {
        5 => SprintDirection::ForwardRight,
        6 => SprintDirection::ForwardLeft,
        _ => SprintDirection::Forward,
    });
    Ok(())
}

pub fn stop_pathfinding(_lua: &Lua, client: &Client, (): ()) -> Result<()> {
    client.stop_pathfinding();
    Ok(())
}

pub fn stop_sleeping(_lua: &Lua, client: &Client, (): ()) -> Result<()> {
    if let Err(error) = client.write_packet(ServerboundPlayerCommand {
        id: client.component::<MinecraftEntityId>(),
        action: Action::StopSleeping,
        data: 0,
    }) {
        error!("failed to send PlayerCommand packet: {error:?}");
    }
    Ok(())
}

pub fn walk(_lua: &Lua, client: &Client, direction: u8) -> Result<()> {
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
