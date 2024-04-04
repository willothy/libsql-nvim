use std::sync::{Arc, RwLock};

use error::IntoLuaResult as _;
use libsql::{Builder, Connection, Database};
use mlua::{Integer, Lua, Table, UserData};
use wrap::prelude::*;

pub mod error;
pub mod object;
pub mod wrap;

#[derive(Clone)]
pub struct LuaDatabase {
    #[allow(unused)]
    db: Arc<RwLock<Database>>,
    conn: Connection,
}

impl LuaDatabase {
    pub async fn connect(url: String, token: String) -> mlua::Result<LuaDatabase> {
        let db = Builder::new_remote(url, token)
            .build()
            .await
            .into_lua_result()?;

        let conn = db.connect().into_lua_result()?;

        Ok(LuaDatabase {
            db: Arc::new(RwLock::new(db)),
            conn,
        })
    }

    pub async fn execute(&self, query: String) -> mlua::Result<Integer> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let result = self.conn.execute(&query, ()).await.into_lua_result()?;
            Ok(result as Integer)
        })
    }
}

impl UserData for LuaDatabase {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("execute", Self::execute.wrap_async());
    }
}

#[mlua::lua_module(name = "libsql")]
pub fn libsql(lua: &Lua) -> mlua::Result<Table> {
    let module = lua.create_table()?;

    module.set(
        "connect",
        lua.create_async_function(|_, (url, token)| LuaDatabase::connect(url, token))?,
    )?;

    Ok(module)
}
