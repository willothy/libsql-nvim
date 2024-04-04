pub(crate) struct LuaSerializer<'lua>(mlua::Value<'lua>);

impl<'lua> LuaSerializer<'lua> {
    pub fn new(value: mlua::Value<'lua>) -> Self {
        LuaSerializer(value)
    }

    pub fn into_sql(self) -> mlua::Result<libsql::Value> {
        match self.0 {
            mlua::Value::Nil => Ok(libsql::Value::Null),
            mlua::Value::Boolean(b) => {
                if b {
                    Ok(libsql::Value::Integer(1))
                } else {
                    Ok(libsql::Value::Integer(0))
                }
            }
            mlua::Value::Integer(i) => Ok(libsql::Value::Integer(i)),
            mlua::Value::Number(f) => Ok(libsql::Value::Real(f)),
            mlua::Value::String(s) => {
                let s = s.to_str().map_err(|_| {
                    mlua::Error::RuntimeError("string is not valid utf-8".to_string())
                })?;
                Ok(libsql::Value::Text(s.to_string()))
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
        }
    }
}
