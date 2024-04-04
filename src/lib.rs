pub mod db;
pub mod rows;
pub mod ser;
pub mod wrap;

use db::LuaDatabase;
use mlua::{Lua, Table};

#[mlua::lua_module(name = "libsql_native")]
pub fn libsql(lua: &Lua) -> mlua::Result<Table> {
    let module = lua.create_table()?;

    module.set(
        "connect",
        lua.create_function(|_, (url, token)| LuaDatabase::connect(url, token))?,
    )?;

    Ok(module)
}
