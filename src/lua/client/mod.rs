mod container;
mod interaction;
mod movement;
mod state;
mod world;

use super::{
    container::item_stack::ItemStack,
    container::{Container, ContainerRef},
    direction::Direction,
    entity::Entity,
    fluid_state::FluidState,
    hunger::Hunger,
    vec3::Vec3,
};
use azalea::Client as AzaleaClient;
use mlua::{Lua, Result, Table, UserData, UserDataFields, UserDataMethods};

pub struct Client {
    pub inner: Option<AzaleaClient>,
}

impl UserData for Client {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("air_supply", state::air_supply);
        f.add_field_method_get("container", container::container);
        f.add_field_method_get("direction", movement::direction);
        f.add_field_method_get("eye_position", movement::eye_position);
        f.add_field_method_get("has_attack_cooldown", interaction::has_attack_cooldown);
        f.add_field_method_get("health", state::health);
        f.add_field_method_get("held_item", container::held_item);
        f.add_field_method_get("held_slot", container::held_slot);
        f.add_field_method_get("hunger", state::hunger);
        f.add_field_method_get("looking_at", movement::looking_at);
        f.add_field_method_get("pathfinder", movement::pathfinder);
        f.add_field_method_get("position", movement::position);
        f.add_field_method_get("score", state::score);
        f.add_field_method_get("tab_list", tab_list);
        f.add_field_method_get("uuid", uuid);
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_async_method("goto", movement::goto);
        m.add_async_method("mine", interaction::mine);
        m.add_async_method("open_container_at", container::open_container_at);
        m.add_async_method("set_client_information", state::set_client_information);
        m.add_method("best_tool_for_block", world::best_tool_for_block);
        m.add_method("chat", chat);
        m.add_method("disconnect", disconnect);
        m.add_method("find_blocks", world::find_blocks);
        m.add_method("find_entities", world::find_entities);
        m.add_method("get_block_state", world::get_block_state);
        m.add_method("get_fluid_state", world::get_fluid_state);
        m.add_method("set_held_slot", container::set_held_slot);
        m.add_method("set_mining", interaction::set_mining);
        m.add_method("stop_pathfinding", movement::stop_pathfinding);
        m.add_method_mut("attack", interaction::attack);
        m.add_method_mut("block_interact", interaction::block_interact);
        m.add_method_mut("jump", movement::jump);
        m.add_method_mut("look_at", movement::look_at);
        m.add_method_mut("open_inventory", container::open_inventory);
        m.add_method_mut("set_direction", movement::set_direction);
        m.add_method_mut("set_jumping", movement::set_jumping);
        m.add_method_mut("sprint", movement::sprint);
        m.add_method_mut("start_mining", interaction::start_mining);
        m.add_method_mut("walk", movement::walk);
    }
}

fn chat(_lua: &Lua, client: &Client, message: String) -> Result<()> {
    client.inner.as_ref().unwrap().chat(&message);
    Ok(())
}

fn disconnect(_lua: &Lua, client: &Client, _: ()) -> Result<()> {
    client.inner.as_ref().unwrap().disconnect();
    Ok(())
}

fn tab_list(lua: &Lua, client: &Client) -> Result<Table> {
    let tab_list = lua.create_table()?;
    for (uuid, player_info) in client.inner.as_ref().unwrap().tab_list() {
        let player = lua.create_table()?;
        player.set("gamemode", player_info.gamemode.name())?;
        player.set("latency", player_info.latency)?;
        player.set("name", player_info.profile.name)?;
        player.set(
            "display_name",
            player_info.display_name.map(|n| n.to_string()),
        )?;
        tab_list.set(uuid.to_string(), player)?;
    }
    Ok(tab_list)
}

fn uuid(_lua: &Lua, client: &Client) -> Result<String> {
    Ok(client.inner.as_ref().unwrap().uuid().to_string())
}
