use mlua::{Lua, Result, Table};

pub fn to_table(lua: &Lua, x: f64, y: f64, z: f64) -> Result<Table> {
    let table = lua.create_table()?;
    table.set("x", x)?;
    table.set("y", y)?;
    table.set("z", z)?;
    Ok(table)
}

pub fn from_table(table: &Table) -> Result<(f64, f64, f64)> {
    Ok((table.get("x")?, table.get("y")?, table.get("z")?))
}
