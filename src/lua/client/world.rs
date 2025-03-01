use super::{Client, Direction, Vec3};
use azalea::{
    BlockPos,
    auto_tool::AutoToolClientExt,
    blocks::{BlockState, BlockStates},
    ecs::query::{With, Without},
    entity::{
        Dead, EntityKind, EntityUuid, LookDirection, Pose, Position as AzaleaPosition,
        metadata::{CustomName, Owneruuid, Player},
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
        ecs.query::<(
            &AzaleaPosition,
            &CustomName,
            &EntityKind,
            &EntityUuid,
            &LookDirection,
            &MinecraftEntityId,
            Option<&Owneruuid>,
            &Pose,
        )>()
        .iter(&ecs)
        .map(
            |(position, custom_name, kind, uuid, direction, id, owner_uuid, pose)| {
                (
                    Vec3::from(position),
                    custom_name.as_ref().map(ToString::to_string),
                    kind.to_string(),
                    uuid.to_string(),
                    Direction::from(direction),
                    id.0,
                    owner_uuid.map(ToOwned::to_owned),
                    *pose as u8,
                )
            },
        )
        .collect::<Vec<_>>()
    };

    let mut matched = Vec::new();
    for (position, custom_name, kind, uuid, direction, id, owner_uuid, pose) in entities {
        let entity = lua.create_table()?;
        entity.set("position", position)?;
        entity.set("custom_name", custom_name)?;
        entity.set("kind", kind)?;
        entity.set("uuid", uuid)?;
        entity.set("direction", direction)?;
        entity.set("id", id)?;
        if let Some(v) = owner_uuid
            && let Some(uuid) = *v
        {
            entity.set("owner_uuid", uuid.to_string())?;
        }
        entity.set("pose", pose)?;
        if filter_fn.call_async::<bool>(&entity).await? {
            matched.push(entity);
        }
    }
    Ok(matched)
}

pub async fn find_players(lua: Lua, client: UserDataRef<Client>, (): ()) -> Result<Vec<Table>> {
    let entities = {
        let mut ecs = client.ecs.lock();
        ecs.query_filtered::<(
            &MinecraftEntityId,
            &EntityUuid,
            &EntityKind,
            &AzaleaPosition,
            &LookDirection,
            &Pose,
        ), (With<Player>, Without<Dead>)>()
            .iter(&ecs)
            .map(|(id, uuid, kind, position, direction, pose)| {
                (
                    id.0,
                    uuid.to_string(),
                    kind.to_string(),
                    Vec3::from(position),
                    Direction::from(direction),
                    *pose as u8,
                )
            })
            .collect::<Vec<_>>()
    };

    let mut players = Vec::new();
    for (id, uuid, kind, position, direction, pose) in entities {
        let entity = lua.create_table()?;
        entity.set("id", id)?;
        entity.set("uuid", uuid)?;
        entity.set("kind", kind)?;
        entity.set("position", position)?;
        entity.set("direction", direction)?;
        entity.set("pose", pose)?;
        players.push(entity);
    }
    Ok(players)
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
