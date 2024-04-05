use std::sync::{Arc, Weak};

use libsql_nvim_derive::FromLuaSerde;
use mlua::serde::LuaSerdeExt;
use mlua::{ExternalResult, OwnedFunction};
use nvim_oxi::libuv;
use tokio::sync::{Mutex, RwLock};

use crate::conn::LuaConnection;
use crate::prelude::*;

#[derive(Clone)]
pub struct LuaDatabase {
    db: Arc<RwLock<libsql::Database>>,
    #[allow(unused)]
    conn: Weak<RwLock<libsql::Connection>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromLuaSerde)]
pub enum LuaDatabaseKind {
    #[serde(rename = "remote")]
    Remote { url: String, token: String },
    #[serde(rename = "local")]
    Local { path: String },
    #[serde(rename = "memory")]
    Memory,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromLuaSerde)]
pub struct LuaDatabaseConfig {
    pub(crate) kind: LuaDatabaseKind,
}

impl LuaDatabase {
    pub fn new(db: libsql::Database) -> Self {
        LuaDatabase {
            db: Arc::new(RwLock::new(db)),
            conn: Weak::new(),
        }
    }

    pub fn connect(&self, cb: OwnedFunction) -> mlua::Result<()> {
        let data = Arc::new(Mutex::new(None));

        if let Some(conn) = self.conn.upgrade() {
            return cb.call(LuaConnection::new(conn));
        }

        let handle = libuv::AsyncHandle::new({
            let data = Arc::clone(&data);
            move || {
                let Some(rv) = data.blocking_lock().take() else {
                    return mlua::Result::Err(mlua::Error::RuntimeError(
                        "data not set".to_string(),
                    ));
                };
                cb.call(rv)
            }
        })
        .into_lua_err()?;

        std::thread::spawn({
            let db = Arc::clone(&self.db);
            move || {
                let rt = tokio::runtime::Runtime::new().into_lua_err()?;
                rt.block_on(async {
                    let conn = db.read().await.connect().into_lua_err()?;

                    data.lock()
                        .await
                        .replace(LuaConnection::new(Arc::new(RwLock::new(conn))));
                    handle.send().into_lua_err()?;
                    mlua::Result::Ok(())
                })
            }
        });

        Ok(())
    }

    pub fn create((config, cb): (LuaDatabaseConfig, OwnedFunction)) -> mlua::Result<()> {
        let data = Arc::new(Mutex::new(None));

        let handle = libuv::AsyncHandle::new({
            let data = Arc::clone(&data);
            move || {
                let Some(rv) = data.blocking_lock().take() else {
                    return Err(mlua::Error::RuntimeError("data not set".to_string()));
                };
                cb.call(rv)
            }
        })
        .into_lua_err()?;

        std::thread::spawn({
            move || {
                let rt = tokio::runtime::Runtime::new().into_lua_err()?;
                rt.block_on(async {
                    let db = match match config.kind {
                        LuaDatabaseKind::Remote { url, token } => {
                            libsql::Builder::new_remote(url, token)
                                .build()
                                .await
                                .into_lua_err()
                        }
                        LuaDatabaseKind::Local { path } => libsql::Builder::new_local(path)
                            .build()
                            .await
                            .into_lua_err(),
                        LuaDatabaseKind::Memory => libsql::Builder::new_local(":memory:")
                            .build()
                            .await
                            .into_lua_err(),
                    } {
                        Ok(db) => db,
                        Err(e) => {
                            data.lock().await.replace(Err(e));
                            return handle.send().into_lua_err();
                        }
                    };

                    let rv = LuaDatabase::new(db);

                    data.lock().await.replace(Ok(rv));
                    handle.send().into_lua_err()?;
                    mlua::Result::Ok(())
                })
            }
        });

        Ok(())
    }
}

impl UserData for LuaDatabase {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("connect", Self::connect.wrap());
    }
}
