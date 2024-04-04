---@class libsql
---@field connect fun(uri: string, token: string): libsql.Connection
local M = require("libsql.native")

---@class libsql.Rows
---@field next fun(): libsql.Row?
---@field column_count fun(self: libsql.Rows): integer
---@field column_name fun(self: libsql.Rows, index: integer): string
---@field column_type fun(self: libsql.Rows, index: integer): string

---@class libsql.Row
---@field get fun(self: libsql.Row, index: integer): any
---@field column_count fun(self: libsql.Row): integer
---@field column_name fun(self: libsql.Row, index: integer): string
---@field column_type fun(self: libsql.Row, index: integer): string

---@class libsql.Connection
---@field query fun(query: string, parameters: string[]): libsql.Rows
---@field execute fun(query: string, parameters: string[]): integer

return M
