use super::{Client, Vec3};
use azalea::{
    BlockPos, SprintDirection, WalkDirection, pathfinder::goals::BlockPosGoal, prelude::*,
};
use mlua::{Lua, Result};

pub fn stop_pathfinding(_lua: &Lua, client: &Client, _: ()) -> Result<()> {
    client.inner.as_ref().unwrap().stop_pathfinding();
    Ok(())
}

pub fn goto(_lua: &Lua, client: &mut Client, position: Vec3) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    client
        .inner
        .as_ref()
        .unwrap()
        .goto(BlockPosGoal(BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        )));
    Ok(())
}

pub fn goto_without_mining(_lua: &Lua, client: &mut Client, position: Vec3) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    client
        .inner
        .as_ref()
        .unwrap()
        .goto_without_mining(BlockPosGoal(BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        )));
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
