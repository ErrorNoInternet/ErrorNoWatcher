use azalea::{BlockPos, entity::Position};
use mlua::{Error, FromLua, IntoLua, Lua, Result, Value};

#[derive(Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl IntoLua for Vec3 {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let table = lua.create_table()?;
        table.set("x", self.x)?;
        table.set("y", self.y)?;
        table.set("z", self.z)?;
        Ok(Value::Table(table))
    }
}

impl From<azalea::Vec3> for Vec3 {
    fn from(v: azalea::Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<&Position> for Vec3 {
    fn from(p: &Position) -> Self {
        Self {
            x: p.x,
            y: p.y,
            z: p.z,
        }
    }
}

impl From<BlockPos> for Vec3 {
    fn from(p: BlockPos) -> Self {
        Self {
            x: f64::from(p.x),
            y: f64::from(p.y),
            z: f64::from(p.z),
        }
    }
}

impl FromLua for Vec3 {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        if let Value::Table(table) = value {
            Ok(
                if let (Ok(x), Ok(y), Ok(z)) = (table.get(1), table.get(2), table.get(3)) {
                    Self { x, y, z }
                } else {
                    Self {
                        x: table.get("x")?,
                        y: table.get("y")?,
                        z: table.get("z")?,
                    }
                },
            )
        } else {
            Err(Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Vec3".to_string(),
                message: None,
            })
        }
    }
}
