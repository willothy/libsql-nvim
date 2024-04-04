# libsql-lua

Unofficial Lua bindings for LibSQL / Turso, mainly focused on the remote API.

> [!Note]
> This is a work in progress. Expect frequent breaking changes.

## Planned Features

- [x] LibSQL remote connections
- [x] Basic query/execute
- [x] Positional placeholders
- [ ] Named placeholders
- [ ] Local DBs
- [ ] Local replica API
- [ ] In-memory DBs
- [ ] DB schema abstraction
- [ ] Query validation
- [ ] JSON or other format support via blobs

## Usage

Currently, only remote connection is supported.

- `libsql.connect(url, token): libsql.Connection`

See [`lua/libsql/_meta.lua`](https://github.com/willothy/libsql-lua/tree/main/lua/libsql/_meta.lua) for API type definitions.

Example:

```lua
local libsql = require("libsql")

local db = libsql.connect("libsql://your-db.turso.io", "your db token")

db:execute(
  [[
  CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT
  );
]],
  {}
)

db:execute("INSERT INTO notes (name) VALUES (?1)", { "One" })
db:execute("INSERT INTO notes (name) VALUES (?1)", { "Two" })
db:execute("INSERT INTO notes (name) VALUES (?1)", { "Three" })

local rows =
  db:query("SELECT * FROM notes WHERE name=?1 OR name=?2", { "One", "Two" })

for row in rows do
  for col = 0, row:column_count() - 1 do
    print(
      string.format(
        "%s: %s = %s",
        row:column_name(col),
        row:column_type(col),
        row:get(col)
      )
    )
  end
end
```
