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
    let h = client.hunger();

    let hunger = lua.create_table()?;
    hunger.set("food", h.food)?;
    hunger.set("saturation", h.saturation)?;
    Ok(hunger)
}

pub fn score(_lua: &Lua, client: &Client) -> Result<i32> {
    Ok(client.component::<Score>().0)
}

pub async fn set_client_information(
    _lua: Lua,
    client: UserDataRef<Client>,
    ci: Table,
) -> Result<()> {
    let model_customization = if let Some(mc) = ci.get::<Option<Table>>("model_customization")? {
        ModelCustomization {
            cape: mc.get("cape")?,
            jacket: mc.get("jacket")?,
            left_sleeve: mc.get("left_sleeve")?,
            right_sleeve: mc.get("right_sleeve")?,
            left_pants: mc.get("left_pants")?,
            right_pants: mc.get("right_pants")?,
            hat: mc.get("hat")?,
        }
    } else {
        ModelCustomization::default()
    };
    if let Err(error) = client
        .set_client_information(ClientInformation {
            allows_listing: ci.get("allows_listing")?,
            model_customization,
            view_distance: ci.get("view_distance").unwrap_or(8),
            ..ClientInformation::default()
        })
        .await
    {
        error!("failed to set client client information: {error:?}");
    }
    Ok(())
}
