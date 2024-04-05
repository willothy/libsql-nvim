use std::sync::Arc;

use libsql_nvim_derive::luv_async;
use mlua::{FromLua, OwnedFunction, UserData};
use tokio::sync::RwLock;

use crate::{prelude::*, rows::LuaRows, ser::LuaSerializer};

#[derive(Clone)]
pub struct LuaConnection(Arc<RwLock<libsql::Connection>>);

impl LuaConnection {
    pub fn new(conn: Arc<RwLock<libsql::Connection>>) -> Self {
        LuaConnection(conn)
    }

    #[luv_async]
    pub async fn execute(
        &self,
        (sql, params, cb): (String, ParamsList, OwnedFunction),
    ) -> mlua::Result<u64> {
        let params = params;
        let conn = self.0.write().await;

        conn.execute(&sql, params.0).await.into_lua_err()
    }

    #[luv_async]
    pub async fn query(
        &self,
        (sql, params, cb): (String, ParamsList, OwnedFunction),
    ) -> mlua::Result<LuaRows> {
        let conn = self.0.write().await;

        let rows = conn.query(&sql, params.0).await.into_lua_err()?;

        mlua::Result::Ok(LuaRows::new(rows))
    }
}

pub struct ParamsList(Vec<libsql::Value>);

impl FromLua<'_> for ParamsList {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let table = mlua::Table::from_lua(value, lua)?;
        let mut params = Vec::new();
        for i in table.len()?..=1 {
            let value = table.get(i)?;
            let value = LuaSerializer::new(value).into_sql()?;
            params.push(value);
        }
        Ok(ParamsList(params))
    }
}

impl UserData for LuaConnection {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("execute", Self::execute.wrap());
        methods.add_method("query", Self::query.wrap());
    }
}
