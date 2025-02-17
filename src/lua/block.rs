use azalea::blocks::{
    Block as AzaleaBlock, BlockState,
    properties::{ChestType, Facing, LightLevel},
};
use mlua::{FromLua, Function, IntoLua, Lua, Result, Table, Value};

#[derive(Clone)]
pub struct Block {
    pub id: String,
    pub friction: f32,
    pub jump_factor: f32,
    pub destroy_time: f32,
    pub explosion_resistance: f32,
    pub requires_correct_tool_for_drops: bool,
}

impl IntoLua for Block {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let table = lua.create_table()?;
        table.set("id", self.id)?;
        table.set("friction", self.friction)?;
        table.set("jump_factor", self.jump_factor)?;
        table.set("destroy_time", self.destroy_time)?;
        table.set("explosion_resistance", self.explosion_resistance)?;
        table.set(
            "requires_correct_tool_for_drops",
            self.requires_correct_tool_for_drops,
        )?;
        Ok(Value::Table(table))
    }
}

impl FromLua for Block {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        if let Value::Table(table) = value {
            Ok(Self {
                id: table.get("id")?,
                friction: table.get("friction")?,
                jump_factor: table.get("jump_factor")?,
                destroy_time: table.get("destroy_time")?,
                explosion_resistance: table.get("explosion_resistance")?,
                requires_correct_tool_for_drops: table.get("requires_correct_tool_for_drops")?,
            })
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Block".to_string(),
                message: None,
            })
        }
    }
}

pub fn register_functions(lua: &Lua, globals: &Table) -> Result<()> {
    globals.set(
        "get_block_from_state",
        lua.create_function(get_block_from_state)?,
    )?;
    globals.set("get_block_states", lua.create_function(get_block_states)?)?;

    Ok(())
}

pub fn get_block_from_state(_lua: &Lua, state: u32) -> Result<Option<Block>> {
    let Ok(state) = BlockState::try_from(state) else {
        return Ok(None);
    };
    let block: Box<dyn AzaleaBlock> = state.into();
    let behavior = block.behavior();

    Ok(Some(Block {
        id: block.id().to_string(),
        friction: behavior.friction,
        jump_factor: behavior.jump_factor,
        destroy_time: behavior.destroy_time,
        explosion_resistance: behavior.explosion_resistance,
        requires_correct_tool_for_drops: behavior.requires_correct_tool_for_drops,
    }))
}

pub fn get_block_states(
    lua: &Lua,
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
                    filter_fn.call::<bool>(p.clone())?
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
