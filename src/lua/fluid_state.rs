use mlua::{FromLua, IntoLua, Lua, Result, Value};

#[derive(Clone)]
pub struct FluidState {
    pub kind: u8,
    pub amount: u8,
    pub falling: bool,
}

impl IntoLua for FluidState {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let table = lua.create_table()?;
        table.set("kind", self.kind)?;
        table.set("amount", self.amount)?;
        table.set("falling", self.falling)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for FluidState {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        if let Value::Table(table) = value {
            Ok(Self {
                kind: table.get("kind")?,
                amount: table.get("amount")?,
                falling: table.get("falling")?,
            })
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "FluidState".to_string(),
                message: None,
            })
        }
    }
}
