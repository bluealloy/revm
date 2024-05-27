.PHONY: sync_tests
sync_tests:
	if [ ! -d tests ]; then \
  		git clone https://github.com/ethereum/tests; \
  	else \
  		cd tests; git pull; git checkout 65994979ae7e318f32d99eaceb4f2180d10145a7; \
  	fi

.PHONY: run_general_state_tests
run_general_state_tests:
	cargo run --color=always --bin revm-rwasm-e2e statetest tests/GeneralStateTests #-s

.PHONY: help
help:
	cargo run --color=always --bin revm-rwasm-e2e -h
