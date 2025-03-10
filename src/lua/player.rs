use azalea::PlayerInfo;
use mlua::{IntoLua, Lua, Result, Value};

#[derive(Clone)]
pub struct Player {
    pub display_name: Option<String>,
    pub gamemode: u8,
    pub latency: i32,
    pub name: String,
    pub uuid: String,
}

impl From<PlayerInfo> for Player {
    fn from(p: PlayerInfo) -> Self {
        Self {
            display_name: p.display_name.map(|text| text.to_string()),
            gamemode: p.gamemode.to_id(),
            latency: p.latency,
            name: p.profile.name,
            uuid: p.uuid.to_string(),
        }
    }
}

impl IntoLua for Player {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let table = lua.create_table()?;
        table.set("display_name", self.display_name)?;
        table.set("gamemode", self.gamemode)?;
        table.set("latency", self.latency)?;
        table.set("name", self.name)?;
        table.set("uuid", self.uuid)?;
        Ok(Value::Table(table))
    }
}
