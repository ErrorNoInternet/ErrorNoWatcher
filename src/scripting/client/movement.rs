use super::{Client, Vec3};
use azalea::{
    BlockPos, BotClientExt, Client as AzaleaClient, SprintDirection, WalkDirection,
    pathfinder::{
        PathfinderClientExt,
        goals::{BlockPosGoal, Goal, RadiusGoal, ReachBlockPosGoal, XZGoal, YGoal},
    },
};
use mlua::{FromLua, Lua, Result, Table, Value};

pub fn stop_pathfinding(_lua: &Lua, client: &Client, _: ()) -> Result<()> {
    client.inner.as_ref().unwrap().stop_pathfinding();
    Ok(())
}

pub fn goto(
    lua: &Lua,
    client: &mut Client,
    (data, metadata): (Value, Option<Table>),
) -> Result<()> {
    fn g(client: &AzaleaClient, without_mining: bool, goal: impl Goal + Send + Sync + 'static) {
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
    let client = client.inner.as_ref().unwrap();
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
            let p = Vec3::from_lua(t.get("position")?, lua)?;
            g(
                client,
                without_mining,
                RadiusGoal {
                    pos: azalea::Vec3::new(p.x, p.y, p.z),
                    radius: t.get("radius")?,
                },
            );
        }
        2 => {
            let p = Vec3::from_lua(data, lua)?;
            g(
                client,
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
                client,
                without_mining,
                XZGoal {
                    x: t.get("x")?,
                    z: t.get("z")?,
                },
            );
        }
        4 => g(
            client,
            without_mining,
            YGoal {
                y: data.as_integer().ok_or(error)?,
            },
        ),
        _ => {
            let p = Vec3::from_lua(data, lua)?;
            g(
                client,
                without_mining,
                BlockPosGoal(BlockPos::new(p.x as i32, p.y as i32, p.z as i32)),
            );
        }
    }

    Ok(())
}

pub fn jump(_lua: &Lua, client: &mut Client, _: ()) -> Result<()> {
    client.inner.as_mut().unwrap().jump();
    Ok(())
}

pub fn look_at(_lua: &Lua, client: &mut Client, position: Vec3) -> Result<()> {
    client
        .inner
        .as_mut()
        .unwrap()
        .look_at(azalea::Vec3::new(position.x, position.y, position.z));
    Ok(())
}

pub fn set_direction(_lua: &Lua, client: &mut Client, direction: (f32, f32)) -> Result<()> {
    client
        .inner
        .as_mut()
        .unwrap()
        .set_direction(direction.0, direction.1);
    Ok(())
}

pub fn set_jumping(_lua: &Lua, client: &mut Client, jumping: bool) -> Result<()> {
    client.inner.as_mut().unwrap().set_jumping(jumping);
    Ok(())
}

pub fn sprint(_lua: &Lua, client: &mut Client, direction: u8) -> Result<()> {
    client.inner.as_mut().unwrap().sprint(match direction {
        5 => SprintDirection::ForwardRight,
        6 => SprintDirection::ForwardLeft,
        _ => SprintDirection::Forward,
    });
    Ok(())
}

pub fn walk(_lua: &Lua, client: &mut Client, direction: u8) -> Result<()> {
    client.inner.as_mut().unwrap().walk(match direction {
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
