use mlua::{FromLua, IntoLua, Lua, Result, Value};

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
