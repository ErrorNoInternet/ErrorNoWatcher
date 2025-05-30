#![allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]

mod container;
mod interaction;
mod movement;
mod state;
mod world;

use std::ops::{Deref, DerefMut};

use azalea::{Client as AzaleaClient, world::MinecraftEntityId};
use mlua::{Lua, Result, UserData, UserDataFields, UserDataMethods};

use super::{
    container::{Container, ContainerRef, item_stack::ItemStack},
    direction::Direction,
    player::Player,
    vec3::Vec3,
};

pub struct Client(pub Option<AzaleaClient>);

impl Deref for Client {
    type Target = AzaleaClient;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("should have received init event")
    }
}

impl DerefMut for Client {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("should have received init event")
    }
}

impl UserData for Client {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("air_supply", state::air_supply);
        f.add_field_method_get("container", container::container);
        f.add_field_method_get("dimension", world::dimension);
        f.add_field_method_get("direction", movement::direction);
        f.add_field_method_get("eye_position", movement::eye_position);
        f.add_field_method_get("go_to_reached", movement::go_to_reached);
        f.add_field_method_get("has_attack_cooldown", interaction::has_attack_cooldown);
        f.add_field_method_get("health", state::health);
        f.add_field_method_get("held_item", container::held_item);
        f.add_field_method_get("held_slot", container::held_slot);
        f.add_field_method_get("hunger", state::hunger);
        f.add_field_method_get("id", id);
        f.add_field_method_get("looking_at", movement::looking_at);
        f.add_field_method_get("menu", container::menu);
        f.add_field_method_get("pathfinder", movement::pathfinder);
        f.add_field_method_get("position", movement::position);
        f.add_field_method_get("score", state::score);
        f.add_field_method_get("tab_list", tab_list);
        f.add_field_method_get("username", username);
        f.add_field_method_get("uuid", uuid);
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_async_method("find_all_entities", world::find::all_entities);
        m.add_async_method("find_all_players", world::find::all_players);
        m.add_async_method("find_entities", world::find::entities);
        m.add_async_method("find_players", world::find::players);
        m.add_async_method("go_to", movement::go_to);
        m.add_async_method(
            "go_to_wait_until_reached",
            movement::go_to_wait_until_reached,
        );
        m.add_async_method("mine", interaction::mine);
        m.add_async_method("open_container_at", container::open_container_at);
        m.add_async_method("set_client_information", state::set_client_information);
        m.add_async_method("start_go_to", movement::start_go_to);
        m.add_method("attack", interaction::attack);
        m.add_method("best_tool_for_block", world::best_tool_for_block);
        m.add_method("block_interact", interaction::block_interact);
        m.add_method("chat", chat);
        m.add_method("disconnect", disconnect);
        m.add_method("find_blocks", world::find::blocks);
        m.add_method("get_block_state", world::get_block_state);
        m.add_method("get_fluid_state", world::get_fluid_state);
        m.add_method("jump", movement::jump);
        m.add_method("look_at", movement::look_at);
        m.add_method("open_inventory", container::open_inventory);
        m.add_method("set_component", state::set_component);
        m.add_method("set_direction", movement::set_direction);
        m.add_method("set_held_slot", container::set_held_slot);
        m.add_method("set_jumping", movement::set_jumping);
        m.add_method("set_mining", interaction::set_mining);
        m.add_method("set_position", movement::set_position);
        m.add_method("set_sneaking", movement::set_sneaking);
        m.add_method("sprint", movement::sprint);
        m.add_method("start_mining", interaction::start_mining);
        m.add_method("stop_pathfinding", movement::stop_pathfinding);
        m.add_method("stop_sleeping", movement::stop_sleeping);
        m.add_method("use_item", interaction::use_item);
        m.add_method("walk", movement::walk);
    }
}

fn chat(_lua: &Lua, client: &Client, message: String) -> Result<()> {
    client.chat(&message);
    Ok(())
}

fn disconnect(_lua: &Lua, client: &Client, (): ()) -> Result<()> {
    client.disconnect();
    Ok(())
}

fn id(_lua: &Lua, client: &Client) -> Result<i32> {
    Ok(client.component::<MinecraftEntityId>().0)
}

fn tab_list(_lua: &Lua, client: &Client) -> Result<Vec<Player>> {
    Ok(client.tab_list().into_values().map(Player::from).collect())
}

fn username(_lua: &Lua, client: &Client) -> Result<String> {
    Ok(client.username())
}

fn uuid(_lua: &Lua, client: &Client) -> Result<String> {
    Ok(client.uuid().to_string())
}
