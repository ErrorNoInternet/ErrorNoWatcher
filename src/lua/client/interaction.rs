use super::{Client, Vec3};
use azalea::{
    BlockPos, BotClientExt,
    protocol::packets::game::{ServerboundUseItem, s_interact::InteractionHand},
    world::MinecraftEntityId,
};
use log::error;
use mlua::{Lua, Result, UserDataRef};

pub fn attack(_lua: &Lua, client: &mut Client, entity_id: i32) -> Result<()> {
    client.attack(MinecraftEntityId(entity_id));
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

pub fn use_item(_lua: &Lua, client: &Client, hand: Option<u8>) -> Result<()> {
    let d = client.direction();
    if let Err(error) = client.write_packet(ServerboundUseItem {
        hand: match hand {
            Some(1) => InteractionHand::OffHand,
            _ => InteractionHand::MainHand,
        },
        sequence: 0,
        yaw: d.0,
        pitch: d.1,
    }) {
        error!("failed to send UseItem packet: {error:?}");
    }
    Ok(())
}
