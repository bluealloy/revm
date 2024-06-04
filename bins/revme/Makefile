.PHONY: sync_tests
sync_tests:
	if [ ! -d tests ]; then \
  		git clone https://github.com/ethereum/tests; \
  	else \
  		cd tests; git pull; git checkout faf33b471465d3c6cdc3d04fbd690895f78d33f2; \
  	fi

.PHONY: run_general_state_tests
run_general_state_tests:
	cargo run --features=revm-rwasm --color=always --bin revme statetest tests/GeneralStateTests #-s

.PHONY: help
help:
	cargo run --color=always --bin revme -h
