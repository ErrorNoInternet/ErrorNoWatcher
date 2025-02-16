use super::{Client, Vec3};
use azalea::{BlockPos, world::MinecraftEntityId};
use mlua::{Lua, Result};

pub fn attack(_lua: &Lua, client: &mut Client, entity_id: u32) -> Result<()> {
    client
        .inner
        .as_mut()
        .unwrap()
        .attack(MinecraftEntityId(entity_id));
    Ok(())
}

pub fn block_interact(_lua: &Lua, client: &mut Client, position: Vec3) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    client.inner.as_mut().unwrap().block_interact(BlockPos::new(
        position.x as i32,
        position.y as i32,
        position.z as i32,
    ));
    Ok(())
}
