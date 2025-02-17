use super::{Block, Client, Entity, FluidState, Vec3};
use azalea::{
    BlockPos,
    blocks::{Block as AzaleaBlock, BlockState, BlockStates},
    ecs::query::Without,
    entity::{Dead, EntityKind, EntityUuid, Position as AzaleaPosition, metadata::CustomName},
    world::MinecraftEntityId,
};
use mlua::{Function, Lua, Result};

pub fn find_blocks(
    _lua: &Lua,
    client: &Client,
    (nearest_to, block_names): (Vec3, Vec<String>),
) -> Result<Vec<Vec3>> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(client
        .inner
        .as_ref()
        .unwrap()
        .world()
        .read()
        .find_blocks(
            BlockPos::new(
                nearest_to.x as i32,
                nearest_to.y as i32,
                nearest_to.z as i32,
            ),
            &BlockStates {
                set: block_names
                    .iter()
                    .flat_map(|n| {
                        (u32::MIN..u32::MAX)
                            .map_while(|i| BlockState::try_from(i).ok())
                            .filter(move |&b| n == Into::<Box<dyn AzaleaBlock>>::into(b).id())
                    })
                    .collect(),
            },
        )
        .map(|p| Vec3 {
            x: f64::from(p.x),
            y: f64::from(p.y),
            z: f64::from(p.z),
        })
        .collect())
}

pub fn find_entities(_lua: &Lua, client: &Client, filter_fn: Function) -> Result<Vec<Entity>> {
    let mut matched = Vec::new();

    let mut ecs = client.inner.as_ref().unwrap().ecs.lock();
    let mut query = ecs.query_filtered::<(
        &MinecraftEntityId,
        &EntityUuid,
        &EntityKind,
        &AzaleaPosition,
        &CustomName,
    ), Without<Dead>>();

    for (&id, uuid, kind, position, custom_name) in query.iter(&ecs) {
        let entity = Entity {
            id: id.0,
            uuid: uuid.to_string(),
            kind: kind.to_string(),
            position: Vec3 {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            custom_name: custom_name.as_ref().map(ToString::to_string),
        };

        if filter_fn.call::<bool>(entity.clone()).unwrap() {
            matched.push(entity);
        }
    }

    Ok(matched)
}

pub fn get_block_from_state(_lua: &Lua, _client: &Client, state: u32) -> Result<Option<Block>> {
    let Ok(state) = BlockState::try_from(state) else {
        return Ok(None);
    };
    let block: Box<dyn AzaleaBlock> = state.into();
    let behavior = block.behavior();

    Ok(Some(Block {
        id: block.id().to_string(),
        friction: behavior.friction,
        jump_factor: behavior.jump_factor,
        destroy_time: behavior.destroy_time,
        explosion_resistance: behavior.explosion_resistance,
        requires_correct_tool_for_drops: behavior.requires_correct_tool_for_drops,
    }))
}

pub fn get_block_state(_lua: &Lua, client: &Client, position: Vec3) -> Result<Option<u16>> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(client
        .inner
        .as_ref()
        .unwrap()
        .world()
        .read()
        .get_block_state(&BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        ))
        .map(|b| b.id))
}

pub fn get_fluid_state(_lua: &Lua, client: &Client, position: Vec3) -> Result<Option<FluidState>> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(client
        .inner
        .as_ref()
        .unwrap()
        .world()
        .read()
        .get_fluid_state(&BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        ))
        .map(|f| FluidState {
            kind: f.kind as u8,
            amount: f.amount,
            falling: f.falling,
        }))
}
