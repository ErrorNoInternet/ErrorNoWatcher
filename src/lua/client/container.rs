use super::{Client, Container, ContainerRef, ItemStack, Vec3};
use azalea::{
    BlockPos, inventory::Inventory, prelude::ContainerClientExt,
    protocol::packets::game::ServerboundSetCarriedItem,
};
use log::error;
use mlua::{Lua, Result, UserDataRef};

pub fn container(_lua: &Lua, client: &Client) -> Result<Option<ContainerRef>> {
    Ok(client
        .inner
        .as_ref()
        .unwrap()
        .get_open_container()
        .map(|c| ContainerRef { inner: c }))
}

pub fn held_item(_lua: &Lua, client: &Client) -> Result<ItemStack> {
    Ok(ItemStack {
        inner: client
            .inner
            .as_ref()
            .unwrap()
            .component::<Inventory>()
            .held_item(),
    })
}

pub fn held_slot(_lua: &Lua, client: &Client) -> Result<u8> {
    Ok(client
        .inner
        .as_ref()
        .unwrap()
        .component::<Inventory>()
        .selected_hotbar_slot)
}

pub async fn open_container_at(
    _lua: Lua,
    client: UserDataRef<Client>,
    position: Vec3,
) -> Result<Option<Container>> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(client
        .inner
        .clone()
        .unwrap()
        .open_container_at(BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        ))
        .await
        .map(|c| Container { inner: c }))
}

pub fn open_inventory(_lua: &Lua, client: &mut Client, _: ()) -> Result<Option<Container>> {
    Ok(client
        .inner
        .as_mut()
        .unwrap()
        .open_inventory()
        .map(|c| Container { inner: c }))
}

pub fn set_held_slot(_lua: &Lua, client: &Client, slot: u8) -> Result<()> {
    if slot > 8 {
        return Ok(());
    }

    let client = client.inner.as_ref().unwrap();
    {
        let mut ecs = client.ecs.lock();
        let mut inventory = client.query::<&mut Inventory>(&mut ecs);
        if inventory.selected_hotbar_slot == slot {
            return Ok(());
        }
        inventory.selected_hotbar_slot = slot;
    };

    if let Err(error) = client.write_packet(ServerboundSetCarriedItem {
        slot: u16::from(slot),
    }) {
        error!("failed to send SetCarriedItem packet: {error:?}");
    }

    Ok(())
}
