use super::{Client, Direction, Vec3};
use azalea::{
    BlockPos, BotClientExt, LookAtEvent, SprintDirection, WalkDirection,
    entity::Position,
    interact::HitResultComponent,
    pathfinder::{
        ExecutingPath, GotoEvent, Pathfinder, PathfinderClientExt,
        goals::{BlockPosGoal, Goal, RadiusGoal, ReachBlockPosGoal, XZGoal, YGoal},
    },
    protocol::packets::game::{ServerboundPlayerCommand, s_player_command::Action},
    world::MinecraftEntityId,
};
use log::error;
use mlua::{FromLua, Lua, Result, Table, UserDataRef, Value};

pub fn direction(_lua: &Lua, client: &Client) -> Result<Direction> {
    let d = client.direction();
    Ok(Direction { x: d.0, y: d.1 })
}

pub fn eye_position(_lua: &Lua, client: &Client) -> Result<Vec3> {
    Ok(Vec3::from(client.eye_position()))
}

pub async fn go_to(
    lua: Lua,
    client: UserDataRef<Client>,
    (data, metadata): (Value, Option<Table>),
) -> Result<()> {
    fn g(client: &Client, without_mining: bool, goal: impl Goal + Send + Sync + 'static) {
        if without_mining {
            client.goto_without_mining(goal);
        } else {
            client.goto(goal);
        }
    }

    let error = mlua::Error::FromLuaConversionError {
        from: data.type_name(),
        to: "Table".to_string(),
        message: None,
    };
    let (goal_type, without_mining) = metadata
        .map(|t| {
            (
                t.get("type").unwrap_or_default(),
                t.get("without_mining").unwrap_or_default(),
            )
        })
        .unwrap_or_default();

    #[allow(clippy::cast_possible_truncation)]
    match goal_type {
        1 => {
            let t = data.as_table().ok_or(error)?;
            let p = Vec3::from_lua(t.get("position")?, &lua)?;
            g(
                &client,
                without_mining,
                RadiusGoal {
                    pos: azalea::Vec3::new(p.x, p.y, p.z),
                    radius: t.get("radius")?,
                },
            );
        }
        2 => {
            let p = Vec3::from_lua(data, &lua)?;
            g(
                &client,
                without_mining,
                ReachBlockPosGoal {
                    pos: BlockPos::new(p.x as i32, p.y as i32, p.z as i32),
                    chunk_storage: client.world().read().chunks.clone(),
                },
            );
        }
        3 => {
            let t = data.as_table().ok_or(error)?;
            g(
                &client,
                without_mining,
                XZGoal {
                    x: t.get("x")?,
                    z: t.get("z")?,
                },
            );
        }
        4 => g(
            &client,
            without_mining,
            YGoal {
                y: data.as_table().ok_or(error)?.get("y")?,
            },
        ),
        _ => {
            let p = Vec3::from_lua(data, &lua)?;
            g(
                &client,
                without_mining,
                BlockPosGoal(BlockPos::new(p.x as i32, p.y as i32, p.z as i32)),
            );
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
    let r = client.component::<HitResultComponent>();
    Ok(if r.miss {
        None
    } else {
        let result = lua.create_table()?;
        result.set("position", Vec3::from(r.block_pos))?;
        result.set("inside", r.inside)?;
        result.set("world_border", r.world_border)?;
        Some(result)
    })
}

pub async fn look_at(_lua: Lua, client: UserDataRef<Client>, position: Vec3) -> Result<()> {
    client
        .clone()
        .look_at(azalea::Vec3::new(position.x, position.y, position.z));

    while client.get_tick_broadcaster().recv().await.is_ok() {
        if client
            .ecs
            .lock()
            .get::<LookAtEvent>(client.entity)
            .is_none()
        {
            break;
        }
    }

    Ok(())
}

pub fn pathfinder(lua: &Lua, client: &Client) -> Result<Table> {
    let pathfinder = lua.create_table()?;
    pathfinder.set(
        "is_calculating",
        client.component::<Pathfinder>().is_calculating,
    )?;
    pathfinder.set(
        "is_executing",
        if let Some(p) = client.get_component::<ExecutingPath>() {
            pathfinder.set("last_reached_node", Vec3::from(p.last_reached_node))?;
            pathfinder.set(
                "last_node_reach_elapsed",
                p.last_node_reached_at.elapsed().as_millis(),
            )?;
            pathfinder.set("is_path_partial", p.is_path_partial)?;
            true
        } else {
            false
        },
    )?;
    Ok(pathfinder)
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
