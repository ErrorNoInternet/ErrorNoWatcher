use mlua::{FromLua, IntoLua, Lua, Result, Value};

#[derive(Clone)]
pub struct Direction {
    pub x: f32,
    pub y: f32,
}

impl IntoLua for Direction {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let table = lua.create_table()?;
        table.set("x", self.x)?;
        table.set("y", self.y)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for Direction {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        if let Value::Table(table) = value {
            Ok(if let (Ok(x), Ok(y)) = (table.get(1), table.get(2)) {
                Self { x, y }
            } else {
                Self {
                    x: table.get("x")?,
                    y: table.get("y")?,
                }
            })
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Direction".to_string(),
                message: None,
            })
        }
    }
}
