use std::sync::Arc;

use tokio::sync::RwLock;

use crate::prelude::*;

use crate::rows::LuaRows;
use crate::ser::LuaSerializer;

#[derive(Clone)]
pub struct LuaDatabase {
    #[allow(unused)]
    db: Arc<RwLock<libsql::Database>>,
    conn: libsql::Connection,
}

impl LuaDatabase {
    #[tokio::main]
    pub async fn connect(url: String, token: String) -> mlua::Result<LuaDatabase> {
        let db = libsql::Builder::new_remote(url, token)
            .build()
            .await
            .into_lua_err()?;

        let conn = db.connect().into_lua_err()?;

        Ok(LuaDatabase {
            db: Arc::new(RwLock::new(db)),
            conn,
        })
    }

    #[tokio::main]
    pub async fn execute<'lua>(
        &self,
        (sql, params): (String, Vec<mlua::Value<'lua>>),
    ) -> mlua::Result<mlua::Integer> {
        let result = self
            .conn
            .execute(
                &sql,
                params.into_iter().try_fold(Vec::new(), |mut acc, val| {
                    let val = LuaSerializer::new(val)
                        .into_sql()
                        .map_err(mlua::Error::external)?;
                    acc.push(val);
                    return mlua::Result::Ok(acc);
                })?,
            )
            .await
            .into_lua_err()?;
        Ok(result as mlua::Integer)
    }

    #[tokio::main]
    pub async fn query<'lua>(
        &self,
        (sql, params): (String, Vec<mlua::Value<'lua>>),
    ) -> mlua::Result<LuaRows> {
        let params = params.into_iter().try_fold(Vec::new(), |mut acc, v| {
            let val = match v {
                mlua::Value::Nil => libsql::Value::Null,
                mlua::Value::Boolean(b) => {
                    if b {
                        libsql::Value::Integer(1)
                    } else {
                        libsql::Value::Integer(0)
                    }
                }
                mlua::Value::Integer(i) => libsql::Value::Integer(i),
                mlua::Value::Number(f) => libsql::Value::Real(f),
                mlua::Value::String(s) => {
                    let s = s.to_str().map_err(|_| {
                        mlua::Error::RuntimeError("string is not valid utf-8".to_string())
                    })?;
                    libsql::Value::Text(s.to_string())
                }
                mlua::Value::Table(_tbl) => {
                    return Err(mlua::Error::RuntimeError(
                        "table is not supported as a parameter".to_string(),
                    ))
                }
                mlua::Value::LightUserData(_)
                | mlua::Value::Function(_)
                | mlua::Value::Thread(_)
                | mlua::Value::UserData(_)
                | mlua::Value::Error(_) => {
                    return Err(mlua::Error::RuntimeError(
                        "unsupported value type".to_string(),
                    ))
                }
            };

            acc.push(val);

            Ok(acc)
        })?;
        let result = self.conn.query(&sql, params).await.into_lua_err()?;
        Ok(LuaRows::new(result))
    }
}

impl UserData for LuaDatabase {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("execute", Self::execute.wrap());
        methods.add_method("query", Self::query.wrap());
    }
}
