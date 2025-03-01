use azalea::blocks::{
    Block as AzaleaBlock, BlockState,
    properties::{ChestType, Facing, LightLevel},
};
use mlua::{Function, Lua, Result, Table};

pub fn register_functions(lua: &Lua, globals: &Table) -> Result<()> {
    globals.set(
        "get_block_from_state",
        lua.create_function(get_block_from_state)?,
    )?;
    globals.set("get_block_states", lua.create_async_function(get_block_states)?)?;

    Ok(())
}

pub fn get_block_from_state(lua: &Lua, state: u32) -> Result<Option<Table>> {
    let Ok(state) = BlockState::try_from(state) else {
        return Ok(None);
    };
    let b: Box<dyn AzaleaBlock> = state.into();
    let bh = b.behavior();

    let block = lua.create_table()?;
    block.set("id", b.id())?;
    block.set("friction", bh.friction)?;
    block.set("jump_factor", bh.jump_factor)?;
    block.set("destroy_time", bh.destroy_time)?;
    block.set("explosion_resistance", bh.explosion_resistance)?;
    block.set(
        "requires_correct_tool_for_drops",
        bh.requires_correct_tool_for_drops,
    )?;
    Ok(Some(block))
}

pub async fn get_block_states(
    lua: Lua,
    (block_names, filter_fn): (Vec<String>, Option<Function>),
) -> Result<Vec<u16>> {
    let mut matched = Vec::new();
    for block_name in block_names {
        for b in
            (u32::MIN..u32::MAX).map_while(|possible_id| BlockState::try_from(possible_id).ok())
        {
            if block_name == Into::<Box<dyn AzaleaBlock>>::into(b).id()
                && (if let Some(filter_fn) = &filter_fn {
                    let p = lua.create_table()?;
                    p.set("chest_type", b.property::<ChestType>().map(|v| v as u8))?;
                    p.set("facing", b.property::<Facing>().map(|v| v as u8))?;
                    p.set("light_level", b.property::<LightLevel>().map(|v| v as u8))?;
                    filter_fn.call_async::<bool>(p.clone()).await?
                } else {
                    true
                })
            {
                matched.push(b.id);
            }
        }
    }
    Ok(matched)
}
