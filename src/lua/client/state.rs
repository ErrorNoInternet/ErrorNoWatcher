use super::Client;
use azalea::{
    ClientInformation,
    entity::metadata::{AirSupply, Score},
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
    client_information: Table,
) -> Result<()> {
    if let Err(error) = client
        .set_client_information(ClientInformation {
            view_distance: client_information.get("view_distance")?,
            ..ClientInformation::default()
        })
        .await
    {
        error!("failed to set client client information: {error:?}");
    }
    Ok(())
}
