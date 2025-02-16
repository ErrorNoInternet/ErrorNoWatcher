use super::{Client, Position};
use azalea::{BlockPos, pathfinder::goals::BlockPosGoal, prelude::*};
use mlua::{Lua, Result};

pub fn goto(_lua: &Lua, client: &mut Client, position: Position) -> Result<()> {
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

pub fn stop_pathfinding(_lua: &Lua, client: &Client, _: ()) -> Result<()> {
    client.inner.as_ref().unwrap().stop_pathfinding();
    Ok(())
}
