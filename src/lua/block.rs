use azalea::blocks::{
    Block as AzaleaBlock, BlockState,
    properties::{ChestType, Facing, LightLevel},
};
use mlua::{Function, Lua, Result, Table};

pub fn register_globals(lua: &Lua, globals: &Table) -> Result<()> {
    globals.set(
        "get_block_from_state",
        lua.create_function(get_block_from_state)?,
    )?;
    globals.set(
        "get_block_states",
        lua.create_async_function(get_block_states)?,
    )?;

    Ok(())
}

pub fn get_block_from_state(lua: &Lua, state: u32) -> Result<Option<Table>> {
    let Ok(state) = BlockState::try_from(state) else {
        return Ok(None);
    };
    let block: Box<dyn AzaleaBlock> = state.into();
    let behavior = block.behavior();

    let table = lua.create_table()?;
    table.set("id", block.id())?;
    table.set("friction", behavior.friction)?;
    table.set("jump_factor", behavior.jump_factor)?;
    table.set("destroy_time", behavior.destroy_time)?;
    table.set("explosion_resistance", behavior.explosion_resistance)?;
    table.set(
        "requires_correct_tool_for_drops",
        behavior.requires_correct_tool_for_drops,
    )?;
    Ok(Some(table))
}

pub async fn get_block_states(
    lua: Lua,
    (block_names, filter_fn): (Vec<String>, Option<Function>),
) -> Result<Vec<u16>> {
    let mut matched = Vec::with_capacity(16);
    for block_name in block_names {
        for block in
            (u32::MIN..u32::MAX).map_while(|possible_id| BlockState::try_from(possible_id).ok())
        {
            if block_name == Into::<Box<dyn AzaleaBlock>>::into(block).id()
                && (if let Some(filter_fn) = &filter_fn {
                    let table = lua.create_table()?;
                    table.set("chest_type", block.property::<ChestType>().map(|v| v as u8))?;
                    table.set("facing", block.property::<Facing>().map(|v| v as u8))?;
                    table.set(
                        "light_level",
                        block.property::<LightLevel>().map(|v| v as u8),
                    )?;
                    filter_fn.call_async::<bool>(table).await?
                } else {
                    true
                })
            {
                matched.push(block.id());
            }
        }
    }
    Ok(matched)
}
