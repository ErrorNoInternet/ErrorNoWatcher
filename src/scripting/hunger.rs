use mlua::{FromLua, IntoLua, Lua, Result, Value};

#[derive(Clone)]
pub struct Hunger {
    pub food: u32,
    pub saturation: f32,
}

impl IntoLua for Hunger {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let table = lua.create_table()?;
        table.set("food", self.food)?;
        table.set("saturation", self.saturation)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for Hunger {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        if let Value::Table(table) = value {
            Ok(Self {
                food: table.get("food")?,
                saturation: table.get("saturation")?,
            })
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Hunger".to_string(),
                message: None,
            })
        }
    }
}
