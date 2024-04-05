use mlua::{ExternalResult, IntoLua, OwnedFunction};
use nvim_oxi::libuv;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::RwLock;

use crate::prelude::*;

pub struct LuaRows {
    inner: Arc<RwLock<libsql::Rows>>,
    n_cols: OnceLock<i32>,
}

impl From<libsql::Rows> for LuaRows {
    fn from(rows: libsql::Rows) -> LuaRows {
        LuaRows {
            inner: Arc::new(RwLock::new(rows)),
            n_cols: OnceLock::new(),
        }
    }
}

impl LuaRows {
    pub fn new(rows: libsql::Rows) -> LuaRows {
        LuaRows {
            inner: Arc::new(RwLock::new(rows)),
            n_cols: OnceLock::new(),
        }
    }

    pub fn column_count(&self) -> mlua::Result<i32> {
        Ok(*self
            .n_cols
            .get_or_init(|| self.inner.blocking_read().column_count()))
    }

    pub fn column_name(&self, (index, cb): (mlua::Integer, OwnedFunction)) -> mlua::Result<()> {
        if index < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if index >= self.column_count()? as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }

        let data = Arc::new(Mutex::new(None));

        let handle = libuv::AsyncHandle::new({
            let data = Arc::clone(&data);
            move || {
                let Ok(mut data) = data.lock() else {
                    cb.call::<Result<String, mlua::Error>, ()>(Err(mlua::Error::RuntimeError(
                        "mutex lock failed".to_string(),
                    )))?;
                    return Ok(());
                };
                let Some(rv) = data.take() else {
                    cb.call::<Result<String, mlua::Error>, ()>(Err(mlua::Error::RuntimeError(
                        "data not set".to_string(),
                    )))?;
                    return Ok(());
                };
                cb.call(rv)
            }
        })
        .into_lua_err()?;

        std::thread::spawn({
            let inner = Arc::clone(&self.inner);
            move || {
                let rt = tokio::runtime::Runtime::new().into_lua_err()?;
                rt.block_on(async {
                    let rv = inner
                        .read()
                        .await
                        .column_name(index as i32)
                        .ok_or_else(|| {
                            mlua::Error::RuntimeError("column name not found".to_string())
                        })
                        .map(ToOwned::to_owned);

                    data.lock()
                        .map_err(|_| mlua::Error::RuntimeError("mutex lock failed".to_string()))?
                        .replace(rv);
                    handle.send().into_lua_err()?;
                    mlua::Result::Ok(())
                })
            }
        });
        Ok(())
    }

    pub fn column_type(&self, (index, cb): (mlua::Integer, OwnedFunction)) -> mlua::Result<()> {
        if index < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if index >= self.column_count()? as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }

        let data = Arc::new(Mutex::new(None));

        let handle = libuv::AsyncHandle::new({
            let data = Arc::clone(&data);
            move || {
                let Ok(mut data) = data.lock() else {
                    cb.call::<Result<String, mlua::Error>, ()>(Err(mlua::Error::RuntimeError(
                        "mutex lock failed".to_string(),
                    )))?;
                    return Ok(());
                };
                let Some(rv) = data.take() else {
                    cb.call::<Result<String, mlua::Error>, ()>(Err(mlua::Error::RuntimeError(
                        "data not set".to_string(),
                    )))?;
                    return Ok(());
                };
                cb.call(rv)
            }
        })
        .into_lua_err()?;

        std::thread::spawn({
            let inner = Arc::clone(&self.inner);
            move || {
                let rt = tokio::runtime::Runtime::new().into_lua_err()?;
                rt.block_on(async {
                    let rv = inner
                        .read()
                        .await
                        .column_type(index as i32)
                        .into_lua_err()
                        .map(|t| match t {
                            libsql::ValueType::Integer => "integer",
                            libsql::ValueType::Real => "real",
                            libsql::ValueType::Text => "text",
                            libsql::ValueType::Blob => "blob",
                            libsql::ValueType::Null => "null",
                        })
                        .map(ToOwned::to_owned);

                    data.lock()
                        .map_err(|_| mlua::Error::RuntimeError("mutex lock failed".to_string()))?
                        .replace(rv);
                    handle.send().into_lua_err()?;
                    mlua::Result::Ok(())
                })
            }
        });
        Ok(())
    }

    pub fn next(&self, cb: OwnedFunction) -> mlua::Result<()> {
        let data = Arc::new(Mutex::new(None));

        let handle = libuv::AsyncHandle::new({
            let data = Arc::clone(&data);
            move || {
                let Ok(mut data) = data.lock() else {
                    cb.call::<Result<Option<LuaRow>, mlua::Error>, ()>(Err(
                        mlua::Error::RuntimeError("mutex lock failed".to_string()),
                    ))?;
                    return Ok(());
                };
                let Some(rv) = data.take() else {
                    cb.call::<Result<Option<LuaRow>, mlua::Error>, ()>(Err(
                        mlua::Error::RuntimeError("data not set".to_string()),
                    ))?;
                    return Ok(());
                };

                cb.call(rv)
            }
        })
        .into_lua_err()?;

        std::thread::spawn({
            let rows = Arc::clone(&self.inner);

            move || {
                tokio::runtime::Runtime::new()
                    .into_lua_err()
                    .and_then(|rt| {
                        let rv = rt
                            .block_on(async {
                                let mut writer = rows.write().await;
                                let rv = match writer.next().await.into_lua_err()? {
                                    Some(row) => Some(LuaRow::new(row, writer.column_count())),
                                    None => None,
                                };
                                mlua::Result::Ok(rv)
                            })
                            .into_lua_err();

                        data.lock()
                            .map_err(|_| mlua::Error::RuntimeError("mutex lock failed".to_string()))
                            .into_lua_err()?
                            .replace(rv);
                        handle.send().into_lua_err()?;
                        Ok(())
                    })
            }
        });
        Ok(())
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

pub struct LuaRow {
    inner: libsql::Row,
    n_cols: i32,
}

impl LuaRow {
    pub fn new(row: libsql::Row, n_cols: i32) -> LuaRow {
        LuaRow { inner: row, n_cols }
    }

    pub fn get<'lua>(&self, lua: &'lua Lua, idx: mlua::Integer) -> mlua::Result<mlua::Value<'lua>> {
        if idx < 0 {
            return Err(mlua::Error::RuntimeError(
                "index must be greater than 0".to_string(),
            ));
        } else if idx >= self.n_cols as i64 {
            return Err(mlua::Error::RuntimeError(
                "column index out of range".to_string(),
            ));
        }
        match self.inner.get_value(idx as i32).into_lua_err()? {
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

    pub fn column_name(&self, idx: mlua::Integer) -> mlua::Result<String> {
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

    pub fn column_type(&self, idx: mlua::Integer) -> mlua::Result<String> {
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
            .into_lua_err()
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
