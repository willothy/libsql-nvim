

lua/libsql/native.so: src/*.rs
	cargo build --release
	cp target/release/libsql_lua.so lua/libsql/native.so
