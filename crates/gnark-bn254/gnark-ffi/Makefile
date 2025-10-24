.PHONY: all clean test

LIB_NAME = libgnark_bn254
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Linux)
    EXT = .a
    SOEXT = .so
endif
ifeq ($(UNAME_S),Darwin)
    EXT = .a
    SOEXT = .dylib
endif
ifeq ($(OS),Windows_NT)
    EXT = .lib
    SOEXT = .dll
endif

all: $(LIB_NAME)$(EXT)

$(LIB_NAME)$(EXT): wrapper.go
	go mod download
	go build -buildmode=c-archive -o $(LIB_NAME)$(EXT) wrapper.go
	@echo "Built $(LIB_NAME)$(EXT)"

test:
	go test -v .

clean:
	rm -f $(LIB_NAME)$(EXT) $(LIB_NAME).h $(LIB_NAME)$(SOEXT)

.DEFAULT_GOAL := all
