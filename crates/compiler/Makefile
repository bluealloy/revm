.PHONY: build
build:
	du -sch ./bin/revm_compiler.wasm || true
	RUSTFLAGS='-C target-feature=+bulk-memory' cargo b --release --target=wasm32-unknown-unknown
	mkdir -p bin
	cp ../../target/wasm32-unknown-unknown/release/revm_compiler.wasm ./bin/revm_compiler.wasm
	wasm2wat ./bin/revm_compiler.wasm > ./bin/revm_compiler.wat
	du -sch ./bin/revm_compiler.wasm
