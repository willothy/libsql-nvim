use std::sync::{Arc, OnceLock, RwLock};

use error::IntoLuaResult;
use libsql::{Builder, Connection, Database};
use mlua::{Integer, IntoLua, Lua, Table, UserData};
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

    pub async fn query<'lua: 'a, 'a>(
        &'a self,
        (query, params): (String, Vec<String>),
    ) -> mlua::Result<LuaRows> {
        let result = self.conn.query(&query, params).await.into_lua_result()?;
        Ok(LuaRows::new(result))
    }
}

impl UserData for LuaDatabase {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("execute", Self::execute.wrap_async());
        methods.add_async_method("query", Self::query.wrap_async());
    }
}

pub struct LuaRows(RwLock<libsql::Rows>, OnceLock<i32>);

impl From<libsql::Rows> for LuaRows {
    fn from(rows: libsql::Rows) -> LuaRows {
        LuaRows(RwLock::new(rows), OnceLock::new())
    }
}

impl LuaRows {
    pub fn new(rows: libsql::Rows) -> LuaRows {
        LuaRows(RwLock::new(rows), OnceLock::new())
    }

    pub fn column_count(&self) -> mlua::Result<i32> {
        if let Some(&count) = self.1.get() {
            return Ok(count);
        }
        let count = self
            .0
            .read()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
            .column_count();
        Ok(*self.1.get_or_init(|| count))
    }

    pub fn column_name(&self, index: Integer) -> mlua::Result<String> {
        if index < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if index >= self.column_count()? as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }

        self.0
            .read()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
            .column_name(index as i32)
            .ok_or_else(|| mlua::Error::RuntimeError("column name not found".to_string()))
            .map(ToOwned::to_owned)
    }

    pub fn column_type(&self, index: Integer) -> mlua::Result<String> {
        if index < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if index >= self.column_count()? as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }
        self.0
            .read()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
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

    pub async fn next(&self, _: ()) -> mlua::Result<Option<LuaRow>> {
        let mut writer = self
            .0
            .write()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        match writer.next().await {
            Ok(Some(row)) => Ok(Some(LuaRow::new(row, self.column_count()?))),
            Ok(None) => Ok(None),
            Err(e) => Err(e).into_lua_result(),
        }
    }
}

impl UserData for LuaRows {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("column_count", Self::column_count.wrap());
        methods.add_method("column_name", Self::column_name.wrap());
        methods.add_method("column_type", Self::column_type.wrap());

        methods.add_async_method("next", Self::next.wrap_async());
    }
}

pub struct LuaRow {
    inner: libsql::Row,
    n_cols: i32,
}

impl LuaRow {
    pub fn new(row: libsql::Row, n_cols: i32) -> LuaRow {
        LuaRow { inner: row, n_cols }
    }

    pub fn get<'lua>(&self, lua: &'lua Lua, idx: Integer) -> mlua::Result<mlua::Value<'lua>> {
        if idx < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if idx >= self.n_cols as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }
        match self.inner.get_value(idx as i32).into_lua_result()? {
            libsql::Value::Null => Ok(mlua::Value::Nil),
            libsql::Value::Integer(i) => i.into_lua(lua),
            libsql::Value::Real(f) => f.into_lua(lua),
            libsql::Value::Text(str) => str.into_lua(lua),
            libsql::Value::Blob(_bytes) => {
                return Err(mlua::Error::RuntimeError(
                    "blob not yet implemented".to_string(),
                ))
            }
        }
    }

    pub fn column_count(&self) -> mlua::Result<i32> {
        Ok(self.n_cols)
    }

    pub fn column_name(&self, idx: Integer) -> mlua::Result<String> {
        if idx < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if idx >= self.n_cols as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }
        self.inner
            .column_name(idx as i32)
            .ok_or_else(|| mlua::Error::RuntimeError("column name not found".to_string()))
            .map(ToOwned::to_owned)
    }

    pub fn column_type(&self, idx: Integer) -> mlua::Result<String> {
        if idx < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if idx >= self.n_cols as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }
        self.inner
            .column_type(idx as i32)
            .into_lua_result()
            .map(|v| match v {
                libsql::ValueType::Null => "null",
                libsql::ValueType::Integer => "integer",
                libsql::ValueType::Real => "real",
                libsql::ValueType::Text => "text",
                libsql::ValueType::Blob => "blob",
            })
            .map(ToOwned::to_owned)
    }
}

impl UserData for LuaRow {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get", Self::get.wrap());
        methods.add_method("column_count", Self::column_count.wrap());
        methods.add_method("column_name", Self::column_name.wrap());
        methods.add_method("column_type", Self::column_type.wrap());
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
