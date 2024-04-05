use libsql_nvim_derive::luv_async;
use mlua::{ExternalResult, IntoLua, OwnedFunction};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

use crate::prelude::*;

#[derive(Clone)]
pub struct LuaRows {
    inner: Arc<RwLock<libsql::Rows>>,
    n_cols: Arc<OnceLock<i32>>,
}

impl From<libsql::Rows> for LuaRows {
    fn from(rows: libsql::Rows) -> LuaRows {
        LuaRows {
            inner: Arc::new(RwLock::new(rows)),
            n_cols: Arc::new(OnceLock::new()),
        }
    }
}

impl LuaRows {
    pub fn new(rows: libsql::Rows) -> LuaRows {
        LuaRows {
            inner: Arc::new(RwLock::new(rows)),
            n_cols: Arc::new(OnceLock::new()),
        }
    }

    #[luv_async]
    pub async fn column_count(&self, cb: OwnedFunction) -> mlua::Result<i32> {
        match self.n_cols.get().copied() {
            Some(n_cols) => Ok(n_cols),
            None => {
                let n_cols = self.inner.read().await.column_count();
                self.n_cols.set(n_cols).ok();
                Ok(n_cols)
            }
        }
    }

    fn column_count_internal(&self) -> i32 {
        if let Some(n_cols) = self.n_cols.get() {
            return *n_cols;
        }
        let n_cols = self.inner.blocking_read().column_count();
        self.n_cols.set(n_cols).ok();
        n_cols
    }

    #[luv_async]
    pub fn column_name(&self, (index, cb): (mlua::Integer, OwnedFunction)) -> mlua::Result<String> {
        match index {
            i64::MIN..=-1 => Err(mlua::Error::RuntimeError(
                "index must be greater than or equal to 0".to_string(),
            )),
            i if i >= self.column_count_internal() as i64 => Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            )),
            i => self
                .inner
                .read()
                .await
                .column_name(i as i32)
                .ok_or_else(|| mlua::Error::RuntimeError("column name not found".to_string()))
                .map(ToOwned::to_owned),
        }
    }

    #[luv_async]
    pub fn column_type(&self, (index, cb): (mlua::Integer, OwnedFunction)) -> mlua::Result<String> {
        match index {
            i64::MIN..=-1 => Err(mlua::Error::RuntimeError(
                "index must be greater than or equal to 0".to_string(),
            )),
            i if i >= self.column_count_internal() as i64 => Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            )),
            i => self
                .inner
                .read()
                .await
                .column_type(i as i32)
                .into_lua_err()
                .map(|t| match t {
                    libsql::ValueType::Integer => "integer",
                    libsql::ValueType::Real => "real",
                    libsql::ValueType::Text => "text",
                    libsql::ValueType::Blob => "blob",
                    libsql::ValueType::Null => "null",
                })
                .map(ToOwned::to_owned),
        }
    }

    #[luv_async]
    pub fn next(&self, cb: OwnedFunction) -> mlua::Result<Option<LuaRow>> {
        let mut writer = self.inner.write().await;
        let rv = match writer.next().await.into_lua_err()? {
            Some(row) => Some(LuaRow::new(row, writer.column_count())),
            None => None,
        };
        mlua::Result::Ok(rv)
    }
}

impl UserData for LuaRows {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("column_count", Self::column_count.wrap());
        methods.add_method("column_name", Self::column_name.wrap());
        methods.add_method("column_type", Self::column_type.wrap());

        methods.add_method("next", Self::next.wrap());

        methods.add_meta_method("__call", Self::next.wrap());
    }
}

#[derive(Clone)]
pub struct LuaRow(Arc<LuaRowInner>);

struct LuaRowInner {
    row: libsql::Row,
    n_cols: i32,
}

struct FieldValue(libsql::Value);

impl IntoLua<'_> for FieldValue {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        match self.0 {
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
}

impl LuaRow {
    pub fn new(row: libsql::Row, n_cols: i32) -> LuaRow {
        LuaRow(Arc::new(LuaRowInner { row, n_cols }))
    }

    #[luv_async]
    pub fn get(&self, (idx, cb): (mlua::Integer, OwnedFunction)) -> mlua::Result<FieldValue> {
        match idx {
            i if i < 0 => Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            )),
            i if i >= self.0.n_cols as i64 => Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            )),
            i => self
                .0
                .row
                .get_value(i as i32)
                .into_lua_err()
                .map(FieldValue),
        }
    }

    #[luv_async]
    pub fn column_count(&self, cb: OwnedFunction) -> mlua::Result<i32> {
        Ok(self.0.n_cols)
    }

    #[luv_async]
    pub fn column_name(&self, (idx, cb): (mlua::Integer, OwnedFunction)) -> mlua::Result<String> {
        match idx {
            i64::MIN..=0 => Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            )),
            i if i >= self.0.n_cols as i64 => Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            )),
            i => self
                .0
                .row
                .column_name(i as i32)
                .ok_or_else(|| mlua::Error::RuntimeError("column name not found".to_string()))
                .map(ToOwned::to_owned),
        }
    }

    #[luv_async]
    pub fn column_type(&self, (idx, cb): (mlua::Integer, OwnedFunction)) -> mlua::Result<String> {
        match idx {
            i64::MIN..=0 => Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            )),
            i if i >= self.0.n_cols as i64 => Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            )),
            i => self
                .0
                .row
                .column_type(i as i32)
                .into_lua_err()
                .map(|v| match v {
                    libsql::ValueType::Null => "null",
                    libsql::ValueType::Integer => "integer",
                    libsql::ValueType::Real => "real",
                    libsql::ValueType::Text => "text",
                    libsql::ValueType::Blob => "blob",
                })
                .map(ToOwned::to_owned),
        }
    }
}

impl UserData for LuaRow {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get", Self::get.wrap());
        methods.add_method("column_count", Self::column_count.wrap());
        methods.add_method("column_name", Self::column_name.wrap());
        methods.add_method("column_type", Self::column_type.wrap());
    }
}
