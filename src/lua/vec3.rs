use mlua::{FromLua, IntoLua, Lua, Result, Value};

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

impl FromLua for Vec3 {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        match value {
            Value::Table(table) => Ok(
                if let (Ok(x), Ok(y), Ok(z)) = (table.get(1), table.get(2), table.get(3)) {
                    Self { x, y, z }
                } else {
                    Self {
                        x: table.get("x")?,
                        y: table.get("y")?,
                        z: table.get("z")?,
                    }
                },
            ),
            Value::Vector(vector) => Ok(Self {
                x: vector.x().into(),
                y: vector.y().into(),
                z: vector.z().into(),
            }),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Vec3".to_string(),
                message: None,
            }),
        }
    }
}
