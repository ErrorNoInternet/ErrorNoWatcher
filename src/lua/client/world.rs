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
    let tr = client.best_tool_in_hotbar_for_block(BlockState { id: block_state });

    let tool_result = lua.create_table()?;
    tool_result.set("index", tr.index)?;
    tool_result.set("percentage_per_tick", tr.percentage_per_tick)?;
    Ok(tool_result)
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
    let mut entities = Vec::new();

    {
        let mut ecs = client.ecs.lock();
        let mut query = ecs.query_filtered::<(
            &MinecraftEntityId,
            &EntityUuid,
            &EntityKind,
            &CustomName,
            &AzaleaPosition,
            &LookDirection,
            &Pose,
        ), Without<Dead>>();

        for (id, uuid, kind, custom_name, position, direction, pose) in query.iter(&ecs) {
            let entity = lua.create_table()?;
            entity.set("id", id.0)?;
            entity.set("uuid", uuid.to_string())?;
            entity.set("kind", kind.to_string())?;
            entity.set("custom_name", custom_name.as_ref().map(ToString::to_string))?;
            entity.set("position", Vec3::from(position))?;
            entity.set("direction", Direction::from(direction))?;
            entity.set("pose", *pose as u8)?;
            entities.push(entity);
        }
    }

    let mut matched = Vec::new();
    for entity in entities {
        if filter_fn.call_async::<bool>(&entity).await? {
            matched.push(entity)
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
        if let Some(fs) = client.world().read().get_fluid_state(&BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        )) {
            let fluid_state = lua.create_table()?;
            fluid_state.set("kind", fs.kind as u8)?;
            fluid_state.set("amount", fs.amount)?;
            fluid_state.set("falling", fs.falling)?;
            Some(fluid_state)
        } else {
            None
        },
    )
}
