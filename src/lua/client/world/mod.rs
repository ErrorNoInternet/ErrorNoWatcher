#[macro_use]
mod queries;
pub mod find;

use azalea::{BlockPos, auto_tool::AutoToolClientExt, blocks::BlockState, world::InstanceName};
use mlua::{Lua, Result, Table, Value};

use super::{Client, Direction, Vec3};

pub fn best_tool_for_block(lua: &Lua, client: &Client, block_state: u16) -> Result<Value> {
    let Ok(block) = BlockState::try_from(block_state) else {
        return Ok(Value::Nil);
    };
    let result = client.best_tool_in_hotbar_for_block(block);
    let table = lua.create_table()?;
    table.set("index", result.index)?;
    table.set("percentage_per_tick", result.percentage_per_tick)?;
    Ok(Value::Table(table))
}

pub fn dimension(_lua: &Lua, client: &Client) -> Result<String> {
    Ok(client.component::<InstanceName>().to_string())
}

pub fn get_block_state(_lua: &Lua, client: &Client, position: Vec3) -> Result<Option<u16>> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(client
        .world()
        .read()
        .get_block_state(&BlockPos::new(
            position.x as i32,
            position.y as i32,
            position.z as i32,
        ))
        .map(|block| block.id()))
}

#[allow(clippy::cast_possible_truncation)]
pub fn get_fluid_state(lua: &Lua, client: &Client, position: Vec3) -> Result<Option<Table>> {
    let fluid_state = client.world().read().get_fluid_state(&BlockPos::new(
        position.x as i32,
        position.y as i32,
        position.z as i32,
    ));
    Ok(if let Some(state) = fluid_state {
        let table = lua.create_table()?;
        table.set("kind", state.kind as u8)?;
        table.set("amount", state.amount)?;
        table.set("falling", state.falling)?;
        Some(table)
    } else {
        None
    })
}
