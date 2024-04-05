use std::sync::Arc;

use mlua::{OwnedFunction, UserData};
use nvim_oxi::libuv;
use tokio::sync::{Mutex, RwLock};

use crate::{prelude::*, rows::LuaRows, ser::LuaSerializer};

#[derive(Clone)]
pub struct LuaConnection(Arc<RwLock<libsql::Connection>>);

impl LuaConnection {
    pub fn new(conn: Arc<RwLock<libsql::Connection>>) -> Self {
        LuaConnection(conn)
    }

    pub fn execute(
        &self,
        (sql, params, cb): (String, Vec<mlua::Value>, OwnedFunction),
    ) -> mlua::Result<()> {
        let data = Arc::new(Mutex::new(None));

        let handle = libuv::AsyncHandle::new({
            let data = Arc::clone(&data);
            move || {
                let Some(_) = data.blocking_lock().take() else {
                    return cb.call::<Result<String, mlua::Error>, ()>(Err(
                        mlua::Error::RuntimeError("data not set".to_string()),
                    ));
                };
                cb.call(mlua::Result::Ok(()))
            }
        })
        .into_lua_err()?;

        let params = params.into_iter().try_fold(Vec::new(), |mut acc, v| {
            LuaSerializer::new(v)
                .into_sql()
                .map_err(mlua::Error::external)
                .and_then(|val| {
                    acc.push(val);
                    Ok(acc)
                })
        })?;
        std::thread::spawn({
            let conn = LuaConnection::clone(self);
            move || {
                let rt = tokio::runtime::Runtime::new().into_lua_err()?;
                rt.block_on(async {
                    let params = params;
                    let conn = conn.0.write().await;

                    let result = conn.execute(&sql, params).await.into_lua_err();

                    data.lock().await.replace(result);

                    handle.send().into_lua_err()?;
                    mlua::Result::Ok(())
                })
            }
        });

        Ok(())
    }

    fn query(
        &self,
        (sql, params, cb): (String, Vec<mlua::Value>, OwnedFunction),
    ) -> mlua::Result<()> {
        let data = Arc::new(Mutex::new(None));

        let handle = libuv::AsyncHandle::new({
            let data = Arc::clone(&data);
            move || {
                let Some(_) = data.blocking_lock().take() else {
                    return cb.call::<Result<String, mlua::Error>, ()>(Err(
                        mlua::Error::RuntimeError("data not set".to_string()),
                    ));
                };
                cb.call(mlua::Result::Ok(()))
            }
        })
        .into_lua_err()?;

        let params = params.into_iter().try_fold(Vec::new(), |mut acc, v| {
            LuaSerializer::new(v)
                .into_sql()
                .map_err(mlua::Error::external)
                .and_then(|val| {
                    acc.push(val);
                    Ok(acc)
                })
        })?;
        std::thread::spawn({
            let conn = LuaConnection::clone(self);
            move || {
                let rt = tokio::runtime::Runtime::new().into_lua_err()?;
                rt.block_on(async {
                    let params = params;
                    let conn = conn.0.write().await;

                    let result = conn
                        .query(&sql, params)
                        .await
                        .into_lua_err()
                        .map(LuaRows::new);

                    data.lock().await.replace(result);

                    handle.send().into_lua_err()?;
                    mlua::Result::Ok(())
                })
            }
        });

        Ok(())
    }
}

impl UserData for LuaConnection {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("execute", Self::execute.wrap());
        methods.add_method("query", Self::query.wrap());
    }
}
