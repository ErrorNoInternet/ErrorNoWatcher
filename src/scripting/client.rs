use super::position::{from_table, to_table};
use azalea::{
    BlockPos, Client as AzaleaClient, ClientInformation,
    ecs::query::Without,
    entity::{Dead, EntityKind, EntityUuid, Position, metadata::CustomName},
    pathfinder::goals::BlockPosGoal,
    prelude::PathfinderClientExt,
    world::MinecraftEntityId,
};
use mlua::{Error, Function, Lua, Result, Table, UserData, UserDataRef};

pub struct Client {
    pub inner: Option<AzaleaClient>,
}

impl UserData for Client {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("pos", |lua, this| {
            let pos = this.inner.as_ref().unwrap().position();
            to_table(lua, pos.x, pos.y, pos.z)
        });
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("set_client_information", set_client_information);
        methods.add_method("get_entity", get_entity);
        methods.add_method_mut("get_entity_position", get_entity_position);
        methods.add_method_mut("goto", goto);
        methods.add_method_mut("stop", stop);
    }
}

async fn set_client_information(
    _lua: Lua,
    client: UserDataRef<Client>,
    client_information: Table,
) -> Result<()> {
    client
        .inner
        .as_ref()
        .unwrap()
        .set_client_information(ClientInformation {
            view_distance: client_information.get("view_distance")?,
            ..ClientInformation::default()
        })
        .await
        .unwrap();
    Ok(())
}

fn get_entity(lua: &Lua, client: &Client, filter_fn: Function) -> Result<u32> {
    let mut ecs = client.inner.as_ref().unwrap().ecs.lock();
    let mut query = ecs.query_filtered::<(
        &MinecraftEntityId,
        &EntityUuid,
        &EntityKind,
        &Position,
        &CustomName,
    ), Without<Dead>>();

    for (&entity_id, entity_uuid, entity_kind, pos, custom_name) in query.iter(&ecs) {
        let entity = lua.create_table()?;

        entity.set("id", entity_id.0)?;
        entity.set("uuid", entity_uuid.to_string())?;
        entity.set("kind", entity_kind.0.to_string())?;
        entity.set("pos", to_table(lua, pos.x, pos.y, pos.z)?)?;
        if let Some(n) = &**custom_name {
            entity.set("custom_name", n.to_string())?;
        }

        if filter_fn.call::<bool>(entity).unwrap() {
            return Ok(entity_id.0);
        };
    }

    Err(Error::RuntimeError("entity not found".to_string()))
}

fn get_entity_position(lua: &Lua, client: &mut Client, entity_id: u32) -> Result<Table> {
    let client = client.inner.as_mut().unwrap();
    let entity = client
        .entity_by::<Without<Dead>, &MinecraftEntityId>(|query_entity_id: &&MinecraftEntityId| {
            query_entity_id.0 == entity_id
        })
        .unwrap();
    let pos = client.entity_component::<Position>(entity);
    to_table(lua, pos.x, pos.y, pos.z)
}

fn goto(_lua: &Lua, client: &mut Client, pos_table: Table) -> Result<()> {
    let pos = from_table(&pos_table)?;
    #[allow(clippy::cast_possible_truncation)]
    client
        .inner
        .as_ref()
        .unwrap()
        .goto(BlockPosGoal(BlockPos::new(
            pos.0 as i32,
            pos.1 as i32,
            pos.2 as i32,
        )));
    Ok(())
}

fn stop(_lua: &Lua, client: &mut Client, _: ()) -> Result<()> {
    client.inner.as_ref().unwrap().stop_pathfinding();
    Ok(())
}
