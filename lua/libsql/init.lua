---@class libsql
---@field connect fun(uri: string, token: string): libsql.Connection
local M = require("libsql.native")

---@class libsql.Rows
---@field next fun(self: libsql.Rows): libsql.Row?
---@field column_count fun(self: libsql.Rows): integer
---@field column_name fun(self: libsql.Rows, index: integer): string
---@field column_type fun(self: libsql.Rows, index: integer): string

---@class libsql.Row
---@field get fun(self: libsql.Row, index: integer): any
---@field column_count fun(self: libsql.Row): integer
---@field column_name fun(self: libsql.Row, index: integer): string
---@field column_type fun(self: libsql.Row, index: integer): string

---@class libsql.Connection
---@field query fun(self: libsql.Connection, query: string, parameters: any[]): libsql.Rows
---@field execute fun(self: libsql.Connection, query: string, parameters: any[]): integer

return M
