use azalea::{
    ClientInformation,
    entity::metadata::{AirSupply, Score},
    pathfinder::PathfinderDebugParticles,
    protocol::common::client_information::ModelCustomization,
};
use mlua::{Error, Lua, Result, Table, UserDataRef};

use super::Client;
use crate::hacks::anti_knockback::AntiKnockback;

pub fn air_supply(_lua: &Lua, client: &Client) -> Result<i32> {
    Ok(client.component::<AirSupply>().0)
}

pub fn health(_lua: &Lua, client: &Client) -> Result<f32> {
    Ok(client.health())
}

pub fn hunger(lua: &Lua, client: &Client) -> Result<Table> {
    let hunger = client.hunger();
    let table = lua.create_table()?;
    table.set("food", hunger.food)?;
    table.set("saturation", hunger.saturation)?;
    Ok(table)
}

pub fn score(_lua: &Lua, client: &Client) -> Result<i32> {
    Ok(client.component::<Score>().0)
}

pub async fn set_client_information(
    _lua: Lua,
    client: UserDataRef<Client>,
    info: Table,
) -> Result<()> {
    let get_bool = |table: &Table, name| table.get(name).unwrap_or(true);
    client
        .set_client_information(ClientInformation {
            allows_listing: info.get("allows_listing")?,
            model_customization: info
                .get::<Table>("model_customization")
                .map(|t| ModelCustomization {
                    cape: get_bool(&t, "cape"),
                    jacket: get_bool(&t, "jacket"),
                    left_sleeve: get_bool(&t, "left_sleeve"),
                    right_sleeve: get_bool(&t, "right_sleeve"),
                    left_pants: get_bool(&t, "left_pants"),
                    right_pants: get_bool(&t, "right_pants"),
                    hat: get_bool(&t, "hat"),
                })
                .unwrap_or_default(),
            view_distance: info.get("view_distance").unwrap_or(8),
            ..ClientInformation::default()
        })
        .await
        .map_err(Error::external)
}

pub fn set_component(
    _lua: &Lua,
    client: &Client,
    (name, enabled): (String, Option<bool>),
) -> Result<()> {
    macro_rules! set {
        ($name:ident) => {{
            let mut ecs = client.ecs.lock();
            let mut entity = ecs.entity_mut(client.entity);
            if enabled.unwrap_or(true) {
                entity.insert($name)
            } else {
                entity.remove::<$name>()
            };
            Ok(())
        }};
    }

    match name.as_str() {
        "AntiKnockback" => set!(AntiKnockback),
        "PathfinderDebugParticles" => set!(PathfinderDebugParticles),
        _ => Err(Error::external("invalid component")),
    }
}
