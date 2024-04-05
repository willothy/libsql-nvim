use std::sync::{atomic::AtomicPtr, Arc};

pub mod conn;
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

    pub use crate::wrap::FunctionNoLua as _;

    pub use mlua::{ExternalResult as _, IntoLua as _};

    pub use mlua::{FromLuaMulti, IntoLuaMulti, Lua, UserData};
}
use prelude::*;
use serde::Serialize as _;

#[mlua::lua_module(name = "libsql_native")]
pub fn libsql(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
    let module = lua.create_table()?;

    unsafe {
        // HACK: This is a workaround to get the main lua state
        // from mlua. It is what it is. Replace ASAP.
        struct LuaInnerStub {
            _state: AtomicPtr<mlua::ffi::lua_State>,
            main_state: *mut mlua::ffi::lua_State,
        }

        let cast = lua as *const mlua::Lua as *const Arc<LuaInnerStub>;
        let arc = Arc::clone(&*cast);
        let state = arc.main_state;

        nvim_oxi::lua::init(state as *mut nvim_oxi::lua::ffi::lua_State);
        nvim_oxi::libuv::init(state as *mut nvim_oxi::lua::ffi::lua_State);
    }

    fn test<'lua: 'a, 'a>(lua: &'lua Lua, _: ()) -> mlua::Result<mlua::Value<'lua>> {
        let ser = mlua::serde::Serializer::new(lua);

        Ok(db::LuaDatabaseKind::Remote {
            url: "akehfukaef".to_owned(),
            token: "akehflahsefeuafh token".to_owned(),
        }
        .serialize(ser)?)
    }
    module.set("test", lua.create_function(|lua, _: ()| test(lua, ()))?)?;

    module.set(
        "new_db",
        lua.create_function(|_lua, args| db::LuaDatabase::create(args))?,
    )?;

    Ok(module)
}
