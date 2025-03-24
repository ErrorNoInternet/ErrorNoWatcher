use azalea::{
    BlockPos,
    blocks::{BlockState, BlockStates},
    ecs::query::{With, Without},
    entity::{
        Dead, EntityKind, EntityUuid, LookDirection, Pose, Position as AzaleaPosition,
        metadata::{CustomName, Owneruuid, Player},
    },
    world::MinecraftEntityId,
};
use mlua::{Function, Lua, Result, Table, UserDataRef};

use super::{Client, Direction, Vec3};

pub fn blocks(
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

pub async fn all_entities(lua: Lua, client: UserDataRef<Client>, (): ()) -> Result<Vec<Table>> {
    let mut matched = Vec::with_capacity(256);
    for (position, custom_name, kind, uuid, direction, id, owner_uuid, pose) in
        get_entities!(client)
    {
        let table = lua.create_table()?;
        table.set("position", position)?;
        table.set("custom_name", custom_name)?;
        table.set("kind", kind)?;
        table.set("uuid", uuid)?;
        table.set("direction", direction)?;
        table.set("id", id)?;
        table.set(
            "owner_uuid",
            owner_uuid.and_then(|v| *v).map(|v| v.to_string()),
        )?;
        table.set("pose", pose)?;
        matched.push(table);
    }
    Ok(matched)
}

pub async fn entities(
    lua: Lua,
    client: UserDataRef<Client>,
    filter_fn: Function,
) -> Result<Vec<Table>> {
    let mut matched = Vec::new();
    for (position, custom_name, kind, uuid, direction, id, owner_uuid, pose) in
        get_entities!(client)
    {
        let table = lua.create_table()?;
        table.set("position", position)?;
        table.set("custom_name", custom_name)?;
        table.set("kind", kind)?;
        table.set("uuid", uuid)?;
        table.set("direction", direction)?;
        table.set("id", id)?;
        table.set(
            "owner_uuid",
            owner_uuid.and_then(|v| *v).map(|v| v.to_string()),
        )?;
        table.set("pose", pose)?;
        if filter_fn.call_async::<bool>(&table).await? {
            matched.push(table);
        }
    }
    Ok(matched)
}

pub async fn all_players(lua: Lua, client: UserDataRef<Client>, (): ()) -> Result<Vec<Table>> {
    let mut matched = Vec::new();
    for (id, uuid, kind, position, direction, pose) in get_players!(client) {
        let table = lua.create_table()?;
        table.set("id", id)?;
        table.set("uuid", uuid)?;
        table.set("kind", kind)?;
        table.set("position", position)?;
        table.set("direction", direction)?;
        table.set("pose", pose)?;
        matched.push(table);
    }
    Ok(matched)
}

pub async fn players(
    lua: Lua,
    client: UserDataRef<Client>,
    filter_fn: Function,
) -> Result<Vec<Table>> {
    let mut matched = Vec::new();
    for (id, uuid, kind, position, direction, pose) in get_players!(client) {
        let table = lua.create_table()?;
        table.set("id", id)?;
        table.set("uuid", uuid)?;
        table.set("kind", kind)?;
        table.set("position", position)?;
        table.set("direction", direction)?;
        table.set("pose", pose)?;
        if filter_fn.call_async::<bool>(&table).await? {
            matched.push(table);
        }
    }
    Ok(matched)
}
