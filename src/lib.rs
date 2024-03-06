mod error;
mod statement;

use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use error::throw_libsql_error;
use mlua::{Lua, Table, UserData, UserDataMethods};

#[derive(Clone)]
pub struct Database {
    db: Arc<Mutex<libsql::Database>>,
    conn: RefCell<Option<Arc<Mutex<libsql::Connection>>>>,
    default_safe_integers: RefCell<bool>,
}

impl Database {
    pub fn new(db: libsql::Database, conn: libsql::Connection) -> Self {
        Database {
            db: Arc::new(Mutex::new(db)),
            conn: RefCell::new(Some(Arc::new(Mutex::new(conn)))),
            default_safe_integers: RefCell::new(false),
        }
    }

    pub fn set_default_safe_integers(&self, toggle: bool) {
        self.default_safe_integers.replace(toggle);
    }

    fn get_conn(&self) -> Arc<Mutex<libsql::Connection>> {
        let conn = self.conn.borrow();
        conn.as_ref().unwrap().clone()
    }
}

impl UserData for Database {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(fields: &mut F) {}

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method("__tostring", |_, _, _: ()| Ok(format!("libsql.Database")));

        methods.add_method("is_in_transaction", |_, this, _: ()| {
            Ok(this
                .get_conn()
                .lock()
                .map_err(|_| mlua::Error::external("Failed to lock connection"))?
                .is_autocommit())
        });

        methods.add_method("close", |_, this, _: ()| {
            this.conn.replace(None);
            Ok(())
        });

        methods.add_method("sync_sync", |lua, this, _: ()| {
            let db = this.db.clone();

            let rt = tokio::runtime::Runtime::new()?;

            rt.block_on(async move {
                let db = db
                    .lock()
                    .map_err(|_| mlua::Error::external("Failed to lock database"))?;
                if let Err(e) = db.sync().await {
                    throw_libsql_error(lua, e)?;
                }
                mlua::Result::Ok(())
            })?;

            Ok(())
        });

        methods.add_method("exec_sync", |_lua, this, (sql,): (mlua::String,)| {
            let rt = tokio::runtime::Runtime::new()?;

            let conn = this.get_conn();
            rt.block_on(async move {
                let sql = sql.to_str().map_err(|_| {
                    mlua::Error::external("Failed to convert lua string to rust string")
                })?;
                conn.lock()
                    .map_err(|_| mlua::Error::external("Failed to lock connection"))?
                    .execute_batch(sql)
                    .await
                    .map_err(|e| mlua::Error::external(format!("Failed to execute sql: {}", e)))?;
                mlua::Result::Ok(())
            })?;

            mlua::Result::Ok(())
        });

        // methods.add_method("prepare_sync", |_lua, this, (sql,): (mlua::String,)| {
        //     let sql = sql.to_str().map_err(|_| {
        //         mlua::Error::external("Failed to convert lua string to rust string")
        //     })?;
        //
        //     let conn = this.get_conn();
        //     let rt = tokio::runtime::Runtime::new()?;
        //     let stmt = rt.block_on(async move {
        //         conn.lock()
        //             .map_err(|_| mlua::Error::external("Failed to lock connection"))?
        //             .prepare(sql)
        //             .await
        //             .map_err(|e| mlua::Error::external(format!("Failed to prepare sql: {}", e)))
        //     })?;
        //     let stmt = Arc::new(Mutex::new(stmt));
        //     // let stmt =
        //     //
        //     todo!()
        // });
    }
}

pub fn lua_db_open(
    lua: &Lua,
    (db_path, auth_token): (mlua::String, mlua::String),
) -> mlua::Result<Database> {
    let db_path = db_path.to_str()?;
    let auth_token = auth_token.to_str()?;
    let db = if is_remote_path(&db_path) {
        let version = version("remote");
        // trace!("Opening remote database: {}", db_path);
        libsql::Database::open_remote_internal(db_path, auth_token, version)
    } else {
        panic!()
        // let cipher = libsql::Cipher::from_str(&encryption_cipher).or_else(|err| {
        //     throw_libsql_error(
        //         &mut cx,
        //         libsql::Error::SqliteFailure(err.extended_code, "".into()),
        //     )
        // })?;
        // let mut builder = libsql::Builder::new_local(&db_path);
        // if !encryption_key.is_empty() {
        //     let encryption_config =
        //         libsql::EncryptionConfig::new(cipher, encryption_key.into());
        //     builder = builder.encryption_config(encryption_config);
        // }
        // rt.block_on(builder.build())
    }
    .or_else(|err| throw_libsql_error(lua, err))?;
    let conn = db.connect().or_else(|err| throw_libsql_error(lua, err))?;
    let db = Database::new(db, conn);
    Ok(db)
}

fn is_remote_path(path: &str) -> bool {
    path.starts_with("libsql://") || path.starts_with("http://") || path.starts_with("https://")
}

fn version(protocol: &str) -> String {
    let ver = env!("CARGO_PKG_VERSION");
    format!("libsql-js-{protocol}-{ver}")
}

#[mlua::lua_module(name = "libsql")]
pub fn entry(lua: &Lua) -> mlua::Result<Table> {
    let t = lua.create_table()?;

    t.set("version", env!("CARGO_PKG_VERSION"))?;

    t.set("open", lua.create_function(lua_db_open)?)?;

    Ok(t)
}
