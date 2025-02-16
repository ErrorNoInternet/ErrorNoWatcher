use super::Client;
use azalea::ClientInformation;
use mlua::{Lua, Result, Table, UserDataRef};

pub async fn set_client_information(
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
