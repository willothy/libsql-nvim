---@meta

---@class libsql.Connection
local Connection = {}

---@param query string
---@param parameters any[]
---@param cb fun(rows: libsql.Rows)
function Connection:query(query, parameters, cb) end

---@param query string
---@param parameters any[]
---@param cb fun(affected: integer)
function Connection:execute(query, parameters, cb) end

---@class libsql.Rows
---@overload fun():libsql.Row?
local Rows = {}

---@param cb fun(row: libsql.Row?)
function Rows:next(cb) end

---@return libsql.Row?
function Rows:next_sync() end

---@param cb fun(columns: integer)
function Rows:column_count(cb) end

---@return integer
function Rows:column_count_sync() end

---@param index integer
---@param cb fun(name: string)
function Rows:column_name(index, cb) end

---@param index integer
---@return string
function Rows:column_name_sync(index) end

---@param index integer
---@param cb fun(type: string)
function Rows:column_type(index, cb) end

---@param index integer
---@return string
function Rows:column_type_sync(index) end

---@class libsql.Row
local Row = {}

---@param index integer
---@return any
function Row:get(index) end

---@return integer
function Row:column_count() end

---@param index integer
---@return string
function Row:column_name(index) end

---@param index integer
---@return string
function Row:column_type(index) end

---@class libsql.Database
local Database = {}

---@param cb fun(conn: libsql.Connection)
function Database:connect(cb) end

---@return libsql.Connection
function Database:connect_sync() end

---@class libsql
local LibSQL = {}

---@alias libsql.DatabaseKind "remote" | "local" | "memory"

---@class libsql.DatabaseConfig
---@field kind libsql.DatabaseKind
---Url for remote databases.
---@field url string?
---Token for remote databases.
---@field token string?
---Path for local databases.
---@field path string?

---@param config libsql.DatabaseConfig
---@param cb fun(conn: libsql.Database)
function LibSQL.new_db(config, cb) end

---@param config libsql.DatabaseConfig
---@return libsql.Database
function LibSQL.new_db_sync(config) end
