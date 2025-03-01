use super::{Client, Direction, Vec3};
use azalea::{
    BlockPos,
    auto_tool::AutoToolClientExt,
    blocks::{BlockState, BlockStates},
    ecs::query::Without,
    entity::{
        Dead, EntityKind, EntityUuid, LookDirection, Pose, Position as AzaleaPosition,
        metadata::CustomName,
    },
    world::{InstanceName, MinecraftEntityId},
};
use mlua::{Function, Lua, Result, Table, UserDataRef};

pub fn best_tool_for_block(lua: &Lua, client: &Client, block_state: u16) -> Result<Table> {
    let result = client.best_tool_in_hotbar_for_block(BlockState { id: block_state });

    let table = lua.create_table()?;
    table.set("index", result.index)?;
    table.set("percentage_per_tick", result.percentage_per_tick)?;
    Ok(table)
}

pub fn dimension(_lua: &Lua, client: &Client) -> Result<String> {
    Ok(client.component::<InstanceName>().to_string())
}

pub fn find_blocks(
    _lua: &Lua,
    client: &Client,
    (nearest_to, block_states): (Vec3, Vec<u16>),
) -> Result<Vec<Vec3>> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(client
        .world()
        .read()
        .find_blocks(
            BlockPos::new(
                nearest_to.x as i32,
                nearest_to.y as i32,
                nearest_to.z as i32,
            ),
            &BlockStates {
                set: block_states.iter().map(|&id| BlockState { id }).collect(),
            },
        )
        .map(Vec3::from)
        .collect())
}

pub async fn find_entities(
    lua: Lua,
    client: UserDataRef<Client>,
    filter_fn: Function,
) -> Result<Vec<Table>> {
    let entities = {
        let mut ecs = client.ecs.lock();
        ecs.query_filtered::<(
            &MinecraftEntityId,
            &EntityUuid,
            &EntityKind,
            &CustomName,
            &AzaleaPosition,
            &LookDirection,
            &Pose,
        ), Without<Dead>>()
            .iter(&ecs)
            .map(|(id, uuid, kind, custom_name, position, direction, pose)| {
                (
                    id.0,
                    uuid.to_string(),
                    kind.to_string(),
                    custom_name.as_ref().map(ToString::to_string),
                    Vec3::from(position),
                    Direction::from(direction),
                    *pose as u8,
                )
            })
            .collect::<Vec<_>>()
    };

    let mut matched = Vec::new();
    for (id, uuid, kind, custom_name, position, direction, pose) in entities {
        let entity = lua.create_table()?;
        entity.set("id", id)?;
        entity.set("uuid", uuid)?;
        entity.set("kind", kind)?;
        entity.set("custom_name", custom_name)?;
        entity.set("position", position)?;
        entity.set("direction", direction)?;
        entity.set("pose", pose)?;
        if filter_fn.call_async::<bool>(&entity).await? {
            matched.push(entity);
        }
    }
    Ok(matched)
}

pub fn get_block_state(_lua: &Lua, client: &Client, position: Vec3) -> Result<Option<u16>> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(client
        .world()
        .read()
        .get_block_state(&BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        ))
        .map(|b| b.id))
}

pub fn get_fluid_state(lua: &Lua, client: &Client, position: Vec3) -> Result<Option<Table>> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(
        if let Some(state) = client.world().read().get_fluid_state(&BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        )) {
            let table = lua.create_table()?;
            table.set("kind", state.kind as u8)?;
            table.set("amount", state.amount)?;
            table.set("falling", state.falling)?;
            Some(table)
        } else {
            None
        },
    )
}
