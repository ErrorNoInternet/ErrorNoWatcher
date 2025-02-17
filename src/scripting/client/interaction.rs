use super::{Client, Vec3};
use azalea::{BlockPos, BotClientExt, world::MinecraftEntityId};
use mlua::{Lua, Result, UserDataRefMut};

pub async fn mine(_lua: Lua, mut client: UserDataRefMut<Client>, position: Vec3) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    client
        .inner
        .as_mut()
        .unwrap()
        .mine(BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        ))
        .await;
    Ok(())
}

pub fn start_mining(_lua: &Lua, client: &mut Client, position: Vec3) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    client.inner.as_mut().unwrap().start_mining(BlockPos::new(
        position.x as i32,
        position.y as i32,
        position.z as i32,
    ));
    Ok(())
}

pub fn set_mining(_lua: &Lua, client: &Client, mining: bool) -> Result<()> {
    client.inner.as_ref().unwrap().left_click_mine(mining);
    Ok(())
}

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
