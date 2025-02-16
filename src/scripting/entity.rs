use super::position::Position;
use mlua::{FromLua, IntoLua, Lua, Result, Value};

#[derive(Clone)]
pub struct Entity {
    pub id: u32,
    pub uuid: String,
    pub kind: String,
    pub position: Position,
    pub custom_name: Option<String>,
}

impl IntoLua for Entity {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let entity = lua.create_table()?;
        entity.set("id", self.id)?;
        entity.set("uuid", self.uuid)?;
        entity.set("kind", self.kind)?;
        entity.set("position", self.position)?;
        entity.set("custom_name", self.custom_name)?;
        Ok(Value::Table(entity))
    }
}

impl FromLua for Entity {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        if let Value::Table(table) = value {
            Ok(Self {
                id: table.get("id")?,
                uuid: table.get("uuid")?,
                kind: table.get("kind")?,
                position: table.get("position")?,
                custom_name: table.get("custom_name")?,
            })
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Position".to_string(),
                message: None,
            })
        }
    }
}
