pub mod db;
pub mod rows;
pub mod ser;
pub mod wrap;

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
pub fn libsql(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
    let module = lua.create_table()?;

    module.set(
        "connect",
        lua.create_function(|_, (url, token)| db::LuaDatabase::connect(url, token))?,
    )?;

    Ok(module)
}
