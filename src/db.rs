use std::sync::{Arc, Weak};

use libsql_nvim_derive::{luv_async, FromLuaSerde};
use mlua::serde::LuaSerdeExt;
use mlua::{ExternalResult, OwnedFunction};
use tokio::sync::RwLock;

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

    #[luv_async]
    async fn connect_impl(&self, cb: OwnedFunction) -> mlua::Result<LuaConnection> {
        let conn = self.db.read().await.connect().into_lua_err()?;

        mlua::Result::Ok(LuaConnection::new(Arc::new(RwLock::new(conn))))
    }

    #[luv_async]
    async fn create_impl(
        (config, cb): (LuaDatabaseConfig, OwnedFunction),
    ) -> mlua::Result<LuaDatabase> {
        let db = match config.kind {
            LuaDatabaseKind::Remote { url, token } => libsql::Builder::new_remote(url, token)
                .build()
                .await
                .into_lua_err(),
            LuaDatabaseKind::Local { path } => libsql::Builder::new_local(path)
                .build()
                .await
                .into_lua_err(),
            LuaDatabaseKind::Memory => libsql::Builder::new_local(":memory:")
                .build()
                .await
                .into_lua_err(),
        }?;

        mlua::Result::Ok(LuaDatabase::new(db))
    }

    pub fn connect(&self, cb: OwnedFunction) -> mlua::Result<()> {
        if let Some(conn) = self.conn.upgrade() {
            return cb.call(LuaConnection::new(conn));
        }

        self.connect_impl(cb)
    }

    pub fn create((config, cb): (LuaDatabaseConfig, OwnedFunction)) -> mlua::Result<()> {
        Self::create_impl((config, cb))
    }
}

impl UserData for LuaDatabase {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("connect", Self::connect.wrap());
    }
}
