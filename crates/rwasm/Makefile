.PHONY: all
all: solid_file

TARGET=wasm32-unknown-unknown
PROFILE=release
TMP_FOLDER=tmp

.PHONY: solid_file
solid_file:
	clear
	cargo b --profile ${PROFILE} --target=${TARGET} --no-default-features --features=sdk
	mkdir -p ${TMP_FOLDER}
	cp ../../target/${TARGET}/${PROFILE}/revm_rwasm.wasm ./${TMP_FOLDER}/$@.wasm
	wasm2wat ./${TMP_FOLDER}/$@.wasm > ./${TMP_FOLDER}/$@.wat
	ls -alh ./${TMP_FOLDER}/

.PHONY: tree
tree:
	clear
	cargo tree -e features

.PHONY: clean
clean:
	rm -rf tmp
	cargo clean
