use std::sync::{Arc, RwLock};

use error::IntoLuaResult;
use libsql::{Builder, Connection, Database};
use mlua::{Function, Integer, IntoLua, Lua, Table, UserData};
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
        let result = self.conn.execute(&query, ()).await.into_lua_result()?;
        Ok(result as Integer)
    }

    // pub async fn query<'lua: 'a, 'a>(
    //     &'a self,
    //     lua: &'lua Lua,
    //     (query, params): (String, Vec<String>),
    // ) -> mlua::Result<mlua::Value> {
    //     let result = self.conn.query(&query, params).await.into_lua_result()?;
    //     // let table = result.into_lua_result()?;
    //     Function::wrap_async(|_, _| async { result.next().await.into_lua_result() }).into_lua(lua)
    // }
}

impl UserData for LuaDatabase {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("execute", Self::execute.wrap_async());
    }
}

pub struct LuaRows(libsql::Rows);

impl From<libsql::Rows> for LuaRows {
    fn from(rows: libsql::Rows) -> LuaRows {
        LuaRows(rows)
    }
}

impl LuaRows {
    pub fn new(rows: libsql::Rows) -> LuaRows {
        LuaRows(rows)
    }

    pub fn column_count(&self) -> mlua::Result<i32> {
        Ok(self.0.column_count())
    }

    pub fn column_name(&self, index: Integer) -> mlua::Result<String> {
        if index < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if index >= self.0.column_count() as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }

        self.0
            .column_name(index as i32)
            .ok_or_else(|| mlua::Error::RuntimeError("column name not found".to_string()))
            .map(ToOwned::to_owned)
    }

    pub fn column_type(&self, index: Integer) -> mlua::Result<String> {
        if index < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if index >= self.0.column_count() as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }
        self.0
            .column_type(index as i32)
            .into_lua_result()
            .map(|t| match t {
                libsql::ValueType::Integer => "integer",
                libsql::ValueType::Real => "real",
                libsql::ValueType::Text => "text",
                libsql::ValueType::Blob => "blob",
                libsql::ValueType::Null => "null",
            })
            .map(ToOwned::to_owned)
    }
}

impl UserData for LuaRows {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("column_count", Self::column_count.wrap());
        methods.add_method("column_name", Self::column_name.wrap());
        methods.add_method("column_type", Self::column_type.wrap());
    }
}

pub struct LuaRow(libsql::Row);

impl From<libsql::Row> for LuaRow {
    fn from(row: libsql::Row) -> LuaRow {
        LuaRow(row)
    }
}

impl LuaRow {
    pub fn new(row: libsql::Row) -> LuaRow {
        LuaRow(row)
    }
}

impl UserData for LuaRow {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {}
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
