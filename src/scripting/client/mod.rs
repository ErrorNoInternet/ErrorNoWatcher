mod interaction;
mod movement;
mod state;
mod world;

use super::{
    block::Block, direction::Direction, entity::Entity, fluid_state::FluidState, hunger::Hunger,
    vec3::Vec3,
};
use azalea::{
    Client as AzaleaClient,
    entity::metadata::{AirSupply, Score},
    interact::HitResultComponent,
};
use mlua::{Lua, Result, UserData, UserDataFields, UserDataMethods};

pub struct Client {
    pub inner: Option<AzaleaClient>,
}

impl UserData for Client {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("air_supply", |_, this| {
            Ok(this.inner.as_ref().unwrap().component::<AirSupply>().0)
        });

        f.add_field_method_get("direction", |_, this| {
            let d = this.inner.as_ref().unwrap().direction();
            Ok(Direction { x: d.0, y: d.1 })
        });

        f.add_field_method_get("eye_position", |_, this| {
            let p = this.inner.as_ref().unwrap().eye_position();
            Ok(Vec3 {
                x: p.x,
                y: p.y,
                z: p.z,
            })
        });

        f.add_field_method_get("health", |_, this| {
            Ok(this.inner.as_ref().unwrap().health())
        });

        f.add_field_method_get("hunger", |_, this| {
            let h = this.inner.as_ref().unwrap().hunger();
            Ok(Hunger {
                food: h.food,
                saturation: h.saturation,
            })
        });

        f.add_field_method_get("looking_at", |lua, this| {
            let hr = this
                .inner
                .as_ref()
                .unwrap()
                .component::<HitResultComponent>();
            Ok(if hr.miss {
                None
            } else {
                let result = lua.create_table()?;
                result.set(
                    "position",
                    Vec3 {
                        x: f64::from(hr.block_pos.x),
                        y: f64::from(hr.block_pos.y),
                        z: f64::from(hr.block_pos.z),
                    },
                )?;
                result.set("inside", hr.inside)?;
                result.set("world_border", hr.world_border)?;
                Some(result)
            })
        });

        f.add_field_method_get("position", |_, this| {
            let p = this.inner.as_ref().unwrap().position();
            Ok(Vec3 {
                x: p.x,
                y: p.y,
                z: p.z,
            })
        });

        f.add_field_method_get("score", |_, this| {
            Ok(this.inner.as_ref().unwrap().component::<Score>().0)
        });

        f.add_field_method_get("tab_list", |lua, this| {
            let tab_list = lua.create_table()?;
            for (uuid, player_info) in this.inner.as_ref().unwrap().tab_list() {
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
        });

        f.add_field_method_get("uuid", |_, this| {
            Ok(this.inner.as_ref().unwrap().uuid().to_string())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_async_method("set_client_information", state::set_client_information);
        m.add_async_method_mut("mine", interaction::mine);
        m.add_method("chat", chat);
        m.add_method("find_blocks", world::find_blocks);
        m.add_method("find_entities", world::find_entities);
        m.add_method("get_block_from_state", world::get_block_from_state);
        m.add_method("get_block_state", world::get_block_state);
        m.add_method("get_fluid_state", world::get_fluid_state);
        m.add_method("set_mining", interaction::set_mining);
        m.add_method("stop_pathfinding", movement::stop_pathfinding);
        m.add_method_mut("attack", interaction::attack);
        m.add_method_mut("block_interact", interaction::block_interact);
        m.add_method_mut("goto", movement::goto);
        m.add_method_mut("jump", movement::jump);
        m.add_method_mut("look_at", movement::look_at);
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
