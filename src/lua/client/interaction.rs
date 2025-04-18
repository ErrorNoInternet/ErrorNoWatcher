use azalea::{
    BlockPos, BotClientExt, interact::StartUseItemEvent,
    protocol::packets::game::s_interact::InteractionHand, world::MinecraftEntityId,
};
use mlua::{Lua, Result, UserDataRef};

use super::{Client, Vec3};

pub fn attack(_lua: &Lua, client: &Client, entity_id: i32) -> Result<()> {
    client.attack(MinecraftEntityId(entity_id));
    Ok(())
}

pub fn block_interact(_lua: &Lua, client: &Client, position: Vec3) -> Result<()> {
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

pub fn set_mining(_lua: &Lua, client: &Client, state: bool) -> Result<()> {
    client.left_click_mine(state);
    Ok(())
}

pub fn start_mining(_lua: &Lua, client: &Client, position: Vec3) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    client.start_mining(BlockPos::new(
        position.x as i32,
        position.y as i32,
        position.z as i32,
    ));
    Ok(())
}

pub fn start_use_item(_lua: &Lua, client: &Client, hand: Option<u8>) -> Result<()> {
    client.ecs.lock().send_event(StartUseItemEvent {
        entity: client.entity,
        hand: match hand {
            Some(1) => InteractionHand::OffHand,
            _ => InteractionHand::MainHand,
        },
        force_block: None,
    });
    Ok(())
}
