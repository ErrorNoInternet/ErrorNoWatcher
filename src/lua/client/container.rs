use super::{Client, Container, ContainerRef, ItemStack, Vec3};
use azalea::{
    BlockPos,
    inventory::{Inventory, Menu, Player, SlotList},
    prelude::ContainerClientExt,
    protocol::packets::game::ServerboundSetCarriedItem,
};
use log::error;
use mlua::{Lua, Result, Table, UserDataRef};

pub fn container(_lua: &Lua, client: &Client) -> Result<Option<ContainerRef>> {
    Ok(client
        .get_open_container()
        .map(|c| ContainerRef { inner: c }))
}

pub fn held_item(_lua: &Lua, client: &Client) -> Result<ItemStack> {
    Ok(ItemStack::from(client.component::<Inventory>().held_item()))
}

pub fn held_slot(_lua: &Lua, client: &Client) -> Result<u8> {
    Ok(client.component::<Inventory>().selected_hotbar_slot)
}

pub fn menu(lua: &Lua, client: &Client) -> Result<Table> {
    fn from_slot_list<const N: usize>(s: SlotList<N>) -> Vec<ItemStack> {
        s.iter()
            .map(|i| ItemStack::from(i.to_owned()))
            .collect::<Vec<_>>()
    }

    let table = lua.create_table()?;
    match client.menu() {
        Menu::Player(Player {
            craft_result,
            craft,
            armor,
            inventory,
            offhand,
        }) => {
            table.set("type", 0)?;
            table.set("craft_result", ItemStack::from(craft_result))?;
            table.set("craft", from_slot_list(craft))?;
            table.set("armor", from_slot_list(armor))?;
            table.set("inventory", from_slot_list(inventory))?;
            table.set("offhand", ItemStack::from(offhand))?;
        }
        Menu::Generic9x6 { contents, player } => {
            table.set("type", 6)?;
            table.set("contents", from_slot_list(contents))?;
            table.set("player", from_slot_list(player))?;
        }
        _ => (),
    }
    Ok(table)
}

pub async fn open_container_at(
    _lua: Lua,
    client: UserDataRef<Client>,
    position: Vec3,
) -> Result<Option<Container>> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(client
        .clone()
        .open_container_at(BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        ))
        .await
        .map(|c| Container { inner: c }))
}

pub fn open_inventory(_lua: &Lua, client: &mut Client, _: ()) -> Result<Option<Container>> {
    Ok(client.open_inventory().map(|c| Container { inner: c }))
}

pub fn set_held_slot(_lua: &Lua, client: &Client, slot: u8) -> Result<()> {
    if slot > 8 {
        return Ok(());
    }

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
