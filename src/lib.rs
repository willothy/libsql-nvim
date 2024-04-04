pub mod db;
pub mod rows;
pub mod ser;
pub mod wrap;

use db::LuaDatabase;
use mlua::{Lua, Table};

#[doc(hidden)]
#[allow(unused)]
pub(crate) mod prelude {
    pub use crate::wrap::{
        AsyncMethodMutNoLua as _, AsyncMethodMutWithLua as _, AsyncMethodNoLua as _,
        AsyncMethodWithLua as _,
    };

    pub use crate::wrap::{
        MethodMutNoLua as _, MethodMutWithLua as _, MethodNoLua as _, MethodWithLua as _,
    };

    pub use crate::wrap::{FieldNoLua as _, FieldWithLua as _};

    pub use crate::wrap::MethodNoLuaNoArgs as _;

    pub use mlua::{ExternalResult as _, IntoLua as _};

    pub use mlua::{FromLuaMulti, IntoLuaMulti, Lua, UserData};
}

#[mlua::lua_module(name = "libsql_native")]
pub fn libsql(lua: &Lua) -> mlua::Result<Table> {
    let module = lua.create_table()?;

    module.set(
        "connect",
        lua.create_function(|_, (url, token)| LuaDatabase::connect(url, token))?,
    )?;

    Ok(module)
}
