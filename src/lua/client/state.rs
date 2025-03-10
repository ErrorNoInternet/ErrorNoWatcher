use super::Client;
use azalea::{
    ClientInformation,
    entity::metadata::{AirSupply, Score},
    protocol::common::client_information::ModelCustomization,
};
use log::error;
use mlua::{Lua, Result, Table, UserDataRef};

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

    if let Err(error) = client
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
    {
        error!("failed to set client client information: {error:?}");
    }
    Ok(())
}
