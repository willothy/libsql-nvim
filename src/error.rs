use mlua::Error as LuaError;

pub trait IntoLuaResult<T, E> {
    fn into_lua_result(self) -> mlua::Result<T>
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>;
}

impl<T, E> IntoLuaResult<T, E> for Result<T, E> {
    fn into_lua_result(self) -> mlua::Result<T>
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        self.map_err(LuaError::external)
    }
}
