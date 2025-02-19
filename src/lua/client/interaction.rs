use super::{Client, Vec3};
use azalea::{BlockPos, BotClientExt, attack::AttackEvent, world::MinecraftEntityId};
use mlua::{Lua, Result, UserDataRef};

pub async fn attack(_lua: Lua, client: UserDataRef<Client>, entity_id: u32) -> Result<()> {
    client.clone().attack(MinecraftEntityId(entity_id));

    while client.get_tick_broadcaster().recv().await.is_ok() {
        if client
            .ecs
            .lock()
            .get::<AttackEvent>(client.entity)
            .is_none()
        {
            break;
        }
    }

    Ok(())
}

pub fn block_interact(_lua: &Lua, client: &mut Client, position: Vec3) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    client.block_interact(BlockPos::new(
        position.x as i32,
        position.y as i32,
        position.z as i32,
    ));
    Ok(())
}

pub fn has_attack_cooldown(_lua: &Lua, client: &Client) -> Result<bool> {
    Ok(client.has_attack_cooldown())
}

pub async fn mine(_lua: Lua, client: UserDataRef<Client>, position: Vec3) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    client
        .clone()
        .mine(BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        ))
        .await;
    Ok(())
}

pub fn set_mining(_lua: &Lua, client: &Client, mining: bool) -> Result<()> {
    client.left_click_mine(mining);
    Ok(())
}

pub fn start_mining(_lua: &Lua, client: &mut Client, position: Vec3) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    client.start_mining(BlockPos::new(
        position.x as i32,
        position.y as i32,
        position.z as i32,
    ));
    Ok(())
}
