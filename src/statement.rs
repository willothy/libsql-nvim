use mlua::Lua;
use std::cell::RefCell;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Statement {
    pub conn: Arc<Mutex<libsql::Connection>>,
    pub stmt: Arc<Mutex<libsql::Statement>>,
    pub raw: RefCell<bool>,
    pub safe_ints: RefCell<bool>,
}

fn lua_value_to_value(_lua: &Lua, v: mlua::Value) -> mlua::Result<libsql::Value> {
    match v {
        mlua::Value::Nil => Ok(libsql::Value::Null),
        mlua::Value::Boolean(_b) => {
            todo!("boolean")
        }
        mlua::Value::Integer(i) => Ok(libsql::Value::Integer(i)),
        mlua::Value::Number(f) => {
            if f.is_finite() {
                Ok(libsql::Value::Real(f))
            } else {
                return Err(mlua::Error::FromLuaConversionError {
                    from: "number",
                    to: "libsql::Value",
                    message: Some("number is not finite".to_string()),
                });
            }
        }
        mlua::Value::String(s) => {
            let s = s.to_str()?;
            Ok(libsql::Value::Text(s.to_string()))
        }
        mlua::Value::Table(_tbl) => {
            todo!("table")
        }
        mlua::Value::Function(_) => {
            todo!("function as bytecode")
        }
        mlua::Value::LightUserData(_) => {
            return Err(mlua::Error::FromLuaConversionError {
                from: "light userdata",
                to: "libsql::Value",
                message: Some("light userdata is not supported".to_string()),
            });
        }
        mlua::Value::Thread(_) => {
            return Err(mlua::Error::FromLuaConversionError {
                from: "thread",
                to: "libsql::Value",
                message: Some("thread is not supported".to_string()),
            });
        }
        mlua::Value::UserData(_) => {
            return Err(mlua::Error::FromLuaConversionError {
                from: "userdata",
                to: "libsql::Value",
                message: Some("userdata is not supported".to_string()),
            });
        }
        mlua::Value::Error(_) => {
            return Err(mlua::Error::FromLuaConversionError {
                from: "error",
                to: "libsql::Value",
                message: Some("error is not supported".to_string()),
            });
        }
    }
}

pub struct Rows {
    rows: RefCell<libsql::Rows>,
    raw: bool,
    safe_ints: bool,
}
