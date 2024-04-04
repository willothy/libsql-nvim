---@meta

---@class libsql.Connection
local Connection = {}

---@param query string
---@param parameters any[]
---@return libsql.Rows
function Connection:query(query, parameters) end

---@param query string
---@param parameters any[]
---@return integer
function Connection:execute(query, parameters) end

---@class libsql.Rows
---@overload fun():libsql.Row?
local Rows = {}

---@return libsql.Row?
function Rows:next() end

---@return integer
function Rows:column_count() end

---@param index integer
---@return string
function Rows:column_name(index) end

---@param index integer
---@return string
function Rows:column_type(index) end

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

---@class libsql
local LibSQL = {}

---@param uri string
---@param token string
---@return libsql.Connection
function LibSQL.connect(uri, token) end
