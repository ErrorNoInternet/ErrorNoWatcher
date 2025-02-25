use azalea::entity::LookDirection;
use mlua::{FromLua, IntoLua, Lua, Result, Value};

#[derive(Clone)]
pub struct Direction {
    pub y: f32,
    pub x: f32,
}

impl From<&LookDirection> for Direction {
    fn from(d: &LookDirection) -> Self {
        Self {
            y: d.y_rot,
            x: d.x_rot,
        }
    }
}

impl IntoLua for Direction {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let table = lua.create_table()?;
        table.set("y", self.y)?;
        table.set("x", self.x)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for Direction {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        if let Value::Table(table) = value {
            Ok(if let (Ok(y), Ok(x)) = (table.get(1), table.get(2)) {
                Self { y, x }
            } else {
                Self {
                    y: table.get("y")?,
                    x: table.get("x")?,
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
