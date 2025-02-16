mod pathfinding;
mod state;

use super::{entity::Entity, position::Position};
use azalea::{
    Client as AzaleaClient,
    ecs::query::Without,
    entity::{Dead, EntityKind, EntityUuid, Position as AzaleaPosition, metadata::CustomName},
    world::MinecraftEntityId,
};
use mlua::{Function, Lua, Result, UserData};

pub struct Client {
    pub inner: Option<AzaleaClient>,
}

impl UserData for Client {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("position", |_, this| {
            let position = this.inner.as_ref().unwrap().position();
            Ok(Position {
                x: position.x,
                y: position.y,
                z: position.z,
            })
        });
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("set_client_information", state::set_client_information);
        methods.add_method("find_entities", find_entities);
        methods.add_method("stop_pathfinding", pathfinding::stop_pathfinding);
        methods.add_method_mut("goto", pathfinding::goto);
    }
}

fn find_entities(_lua: &Lua, client: &Client, filter_fn: Function) -> Result<Vec<Entity>> {
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
            position: Position {
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
